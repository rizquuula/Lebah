use std::sync::mpsc::{Receiver, RecvTimeoutError, TryRecvError};
use std::time::{Duration, Instant};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StreamKind {
    Stdout,
    Stderr,
}

#[derive(Clone, Copy, Debug)]
pub struct PumpConfig {
    pub poll_interval: Duration,
    pub post_exit_grace: Duration,
}

impl Default for PumpConfig {
    fn default() -> Self {
        Self {
            poll_interval: Duration::from_millis(200),
            post_exit_grace: Duration::from_secs(2),
        }
    }
}

/// Drain `stdout_rx`/`stderr_rx` to `on_output`, then invoke `on_end(success)` exactly once.
///
/// Why: the runner's reader threads can stay blocked in `read_line` forever when the agent
/// spawns a grandchild that inherits the stdout/stderr pipes — EOF never arrives even after
/// the direct child exits. A plain `for line in rx` would leak the wiring thread in that case.
/// After `exit_rx` fires we give readers a short grace period to deliver trailing output, then
/// return regardless of whether the output channels have closed.
pub fn pump_handle<FOut, FEnd>(
    stdout_rx: Receiver<String>,
    stderr_rx: Receiver<String>,
    exit_rx: Receiver<bool>,
    config: PumpConfig,
    mut on_output: FOut,
    on_end: FEnd,
) where
    FOut: FnMut(StreamKind, String),
    FEnd: FnOnce(bool),
{
    let mut stdout_done = false;
    let mut stderr_done = false;
    let mut exit: Option<(bool, Instant)> = None;

    loop {
        let mut made_progress = false;

        if !stdout_done {
            match stdout_rx.try_recv() {
                Ok(line) => {
                    on_output(StreamKind::Stdout, line);
                    made_progress = true;
                }
                Err(TryRecvError::Empty) => {}
                Err(TryRecvError::Disconnected) => stdout_done = true,
            }
        }

        if !stderr_done {
            match stderr_rx.try_recv() {
                Ok(line) => {
                    on_output(StreamKind::Stderr, line);
                    made_progress = true;
                }
                Err(TryRecvError::Empty) => {}
                Err(TryRecvError::Disconnected) => stderr_done = true,
            }
        }

        if exit.is_none() {
            match exit_rx.try_recv() {
                Ok(success) => exit = Some((success, Instant::now())),
                Err(TryRecvError::Empty) => {}
                Err(TryRecvError::Disconnected) => exit = Some((false, Instant::now())),
            }
        }

        if let Some((success, at)) = exit {
            if (stdout_done && stderr_done) || at.elapsed() >= config.post_exit_grace {
                on_end(success);
                return;
            }
        } else if stdout_done && stderr_done {
            // Both streams closed before any exit signal — wait briefly for exit_rx,
            // then treat disconnect as failure.
            match exit_rx.recv_timeout(config.post_exit_grace) {
                Ok(success) => {
                    on_end(success);
                    return;
                }
                Err(RecvTimeoutError::Timeout) | Err(RecvTimeoutError::Disconnected) => {
                    on_end(false);
                    return;
                }
            }
        }

        if !made_progress {
            std::thread::sleep(config.poll_interval);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::mpsc;
    use std::sync::{Arc, Mutex};
    use std::thread;

    fn short_config() -> PumpConfig {
        PumpConfig {
            poll_interval: Duration::from_millis(5),
            post_exit_grace: Duration::from_millis(80),
        }
    }

    #[derive(Default)]
    struct Sink {
        outputs: Vec<(StreamKind, String)>,
        ended: Option<bool>,
    }

    fn run_pump(
        stdout_rx: Receiver<String>,
        stderr_rx: Receiver<String>,
        exit_rx: Receiver<bool>,
        config: PumpConfig,
    ) -> Arc<Mutex<Sink>> {
        let sink = Arc::new(Mutex::new(Sink::default()));
        let sink_c = Arc::clone(&sink);
        let sink_e = Arc::clone(&sink);
        thread::spawn(move || {
            pump_handle(
                stdout_rx,
                stderr_rx,
                exit_rx,
                config,
                move |k, l| sink_c.lock().unwrap().outputs.push((k, l)),
                move |s| sink_e.lock().unwrap().ended = Some(s),
            );
        });
        sink
    }

    fn wait_for_end(sink: &Arc<Mutex<Sink>>, timeout: Duration) -> Option<bool> {
        let start = Instant::now();
        while start.elapsed() < timeout {
            if let Some(v) = sink.lock().unwrap().ended {
                return Some(v);
            }
            thread::sleep(Duration::from_millis(5));
        }
        None
    }

    #[test]
    fn delivers_output_then_ends_on_clean_exit() {
        let (so_tx, so_rx) = mpsc::channel();
        let (se_tx, se_rx) = mpsc::channel();
        let (ex_tx, ex_rx) = mpsc::channel();

        let sink = run_pump(so_rx, se_rx, ex_rx, short_config());

        so_tx.send("hello".to_string()).unwrap();
        se_tx.send("warn".to_string()).unwrap();
        drop(so_tx);
        drop(se_tx);
        ex_tx.send(true).unwrap();

        let ended = wait_for_end(&sink, Duration::from_secs(1)).expect("pump did not end");
        assert!(ended);
        let outs = &sink.lock().unwrap().outputs;
        assert!(outs.contains(&(StreamKind::Stdout, "hello".to_string())));
        assert!(outs.contains(&(StreamKind::Stderr, "warn".to_string())));
    }

    #[test]
    fn ends_within_grace_when_streams_never_close_after_exit() {
        // Simulates grandchild holding stdout/stderr pipes open after the agent exits.
        let (_so_tx, so_rx) = mpsc::channel::<String>();
        let (_se_tx, se_rx) = mpsc::channel::<String>();
        let (ex_tx, ex_rx) = mpsc::channel();

        let config = short_config();
        let sink = run_pump(so_rx, se_rx, ex_rx, config);

        ex_tx.send(true).unwrap();

        let deadline = config.post_exit_grace + Duration::from_millis(400);
        let ended = wait_for_end(&sink, deadline).expect("pump hung past grace period");
        assert!(ended);
        // Keep senders alive past the assertion to mimic the leaked-pipe scenario.
        drop(_so_tx);
        drop(_se_tx);
    }

    #[test]
    fn treats_exit_disconnect_as_failure() {
        let (so_tx, so_rx) = mpsc::channel::<String>();
        let (se_tx, se_rx) = mpsc::channel::<String>();
        let (ex_tx, ex_rx) = mpsc::channel::<bool>();

        let sink = run_pump(so_rx, se_rx, ex_rx, short_config());

        drop(so_tx);
        drop(se_tx);
        drop(ex_tx);

        let ended = wait_for_end(&sink, Duration::from_secs(1)).expect("pump did not end");
        assert!(!ended);
    }

    #[test]
    fn drains_pending_output_delivered_before_exit_fires() {
        let (so_tx, so_rx) = mpsc::channel();
        let (_se_tx, se_rx) = mpsc::channel::<String>();
        let (ex_tx, ex_rx) = mpsc::channel();

        let sink = run_pump(so_rx, se_rx, ex_rx, short_config());

        for i in 0..5 {
            so_tx.send(format!("line-{i}")).unwrap();
        }
        drop(so_tx);
        ex_tx.send(true).unwrap();

        let ended = wait_for_end(&sink, Duration::from_secs(1)).expect("pump did not end");
        assert!(ended);
        let outs = &sink.lock().unwrap().outputs;
        let stdout_lines: Vec<_> = outs
            .iter()
            .filter(|(k, _)| *k == StreamKind::Stdout)
            .map(|(_, l)| l.clone())
            .collect();
        assert_eq!(
            stdout_lines,
            (0..5).map(|i| format!("line-{i}")).collect::<Vec<_>>()
        );
    }
}
