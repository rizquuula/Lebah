use std::sync::mpsc;

/// Normalizes OpenCode JSON streaming output into Claude-compatible JSON format.
///
/// OpenCode emits: step_start, text, tool_use, step_finish
/// Claude emits: system/init, assistant, result
///
/// This normalizer transforms each opencode line so the frontend parser
/// (TerminalModal.svelte) can handle both agents without changes.
pub fn spawn_normalizer(raw_rx: mpsc::Receiver<String>) -> mpsc::Receiver<String> {
    let (normalized_tx, normalized_rx) = mpsc::channel();

    std::thread::spawn(move || {
        let mut is_first_step = true;
        let mut total_input_tokens: i64 = 0;
        let mut total_output_tokens: i64 = 0;
        let mut total_cost: f64 = 0.0;
        let mut last_reason = String::new();

        for line in raw_rx {
            let trimmed = line.trim();
            if trimmed.is_empty() {
                continue;
            }

            let obj: serde_json::Value = match serde_json::from_str(trimmed) {
                Ok(v) => v,
                Err(_) => {
                    // Non-JSON line — pass through as-is (stderr, etc.)
                    let _ = normalized_tx.send(line);
                    continue;
                }
            };

            let event_type = obj.get("type").and_then(|v| v.as_str()).unwrap_or("");

            match event_type {
                "step_start" => {
                    if is_first_step {
                        is_first_step = false;
                        let init = serde_json::json!({
                            "type": "system",
                            "subtype": "init",
                            "model": "opencode"
                        });
                        let _ = normalized_tx.send(init.to_string());
                    }
                }
                "text" => {
                    let text = obj
                        .pointer("/part/text")
                        .and_then(|v| v.as_str())
                        .unwrap_or("");
                    if !text.is_empty() {
                        let msg = serde_json::json!({
                            "type": "assistant",
                            "message": {
                                "content": [{ "type": "text", "text": text }],
                                "usage": {}
                            }
                        });
                        let _ = normalized_tx.send(msg.to_string());
                    }
                }
                "tool_use" => {
                    let tool_name = obj
                        .pointer("/part/tool")
                        .and_then(|v| v.as_str())
                        .unwrap_or("unknown");
                    let input = obj
                        .pointer("/part/state/input")
                        .cloned()
                        .unwrap_or(serde_json::Value::Object(serde_json::Map::new()));
                    let title = obj
                        .pointer("/part/state/title")
                        .and_then(|v| v.as_str())
                        .or_else(|| {
                            obj.pointer("/part/state/input/description")
                                .and_then(|v| v.as_str())
                        })
                        .unwrap_or(tool_name);

                    // Emit tool_use
                    let tool_msg = serde_json::json!({
                        "type": "assistant",
                        "message": {
                            "content": [{
                                "type": "tool_use",
                                "name": title,
                                "input": input
                            }],
                            "usage": {}
                        }
                    });
                    let _ = normalized_tx.send(tool_msg.to_string());
                }
                "step_finish" => {
                    let reason = obj
                        .pointer("/part/reason")
                        .and_then(|v| v.as_str())
                        .unwrap_or("");
                    last_reason = reason.to_string();

                    // Accumulate tokens and cost
                    if let Some(tokens) = obj.pointer("/part/tokens") {
                        total_input_tokens +=
                            tokens.get("input").and_then(|v| v.as_i64()).unwrap_or(0);
                        total_output_tokens +=
                            tokens.get("output").and_then(|v| v.as_i64()).unwrap_or(0);
                    }
                    total_cost += obj
                        .pointer("/part/cost")
                        .and_then(|v| v.as_f64())
                        .unwrap_or(0.0);

                    // Only emit result on final stop (not intermediate tool-calls)
                    if reason == "stop" || reason == "end_turn" {
                        let result = serde_json::json!({
                            "type": "result",
                            "is_error": false,
                            "total_cost_usd": total_cost,
                            "duration_ms": 0,
                            "usage": {
                                "input_tokens": total_input_tokens,
                                "output_tokens": total_output_tokens,
                                "cache_read_input_tokens": 0,
                                "cache_creation_input_tokens": 0
                            }
                        });
                        let _ = normalized_tx.send(result.to_string());
                    }
                }
                _ => {
                    // Unknown event type — pass through
                    let _ = normalized_tx.send(line);
                }
            }
        }

        // If the process ended without a stop event, emit a result based on last state
        if last_reason != "stop" && last_reason != "end_turn" {
            let result = serde_json::json!({
                "type": "result",
                "is_error": last_reason != "tool-calls",
                "total_cost_usd": total_cost,
                "duration_ms": 0,
                "usage": {
                    "input_tokens": total_input_tokens,
                    "output_tokens": total_output_tokens,
                    "cache_read_input_tokens": 0,
                    "cache_creation_input_tokens": 0
                }
            });
            let _ = normalized_tx.send(result.to_string());
        }
    });

    normalized_rx
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalizes_step_start_to_init() {
        let (tx, rx) = mpsc::channel();
        let normalized = spawn_normalizer(rx);

        tx.send(r#"{"type":"step_start","timestamp":123,"sessionID":"s1","part":{"type":"step-start"}}"#.to_string()).unwrap();
        drop(tx);

        let line = normalized.recv().unwrap();
        let obj: serde_json::Value = serde_json::from_str(&line).unwrap();
        assert_eq!(obj["type"], "system");
        assert_eq!(obj["subtype"], "init");
    }

    #[test]
    fn normalizes_text_to_assistant() {
        let (tx, rx) = mpsc::channel();
        let normalized = spawn_normalizer(rx);

        tx.send(r#"{"type":"text","part":{"type":"text","text":"hello world"}}"#.to_string())
            .unwrap();
        drop(tx);

        // Skip the init event if first step isn't sent
        let line = normalized.recv().unwrap();
        let obj: serde_json::Value = serde_json::from_str(&line).unwrap();
        assert_eq!(obj["type"], "assistant");
        assert_eq!(obj["message"]["content"][0]["text"], "hello world");
    }

    #[test]
    fn normalizes_step_finish_stop_to_result() {
        let (tx, rx) = mpsc::channel();
        let normalized = spawn_normalizer(rx);

        tx.send(r#"{"type":"step_finish","part":{"reason":"stop","tokens":{"input":100,"output":50},"cost":0.01}}"#.to_string()).unwrap();
        drop(tx);

        let line = normalized.recv().unwrap();
        let obj: serde_json::Value = serde_json::from_str(&line).unwrap();
        assert_eq!(obj["type"], "result");
        assert_eq!(obj["is_error"], false);
        assert_eq!(obj["usage"]["input_tokens"], 100);
    }

    #[test]
    fn intermediate_tool_calls_no_result() {
        let (tx, rx) = mpsc::channel();
        let normalized = spawn_normalizer(rx);

        tx.send(r#"{"type":"step_finish","part":{"reason":"tool-calls","tokens":{"input":50,"output":10},"cost":0.001}}"#.to_string()).unwrap();
        tx.send(r#"{"type":"step_finish","part":{"reason":"stop","tokens":{"input":100,"output":50},"cost":0.01}}"#.to_string()).unwrap();
        drop(tx);

        // Should only get one result (the stop one), with accumulated tokens
        let line = normalized.recv().unwrap();
        let obj: serde_json::Value = serde_json::from_str(&line).unwrap();
        assert_eq!(obj["type"], "result");
        assert_eq!(obj["usage"]["input_tokens"], 150);
        assert_eq!(obj["usage"]["output_tokens"], 60);
    }
}
