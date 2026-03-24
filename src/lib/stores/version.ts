import { writable } from "svelte/store";
import { invoke } from "@tauri-apps/api/core";

export const appVersion = writable<string>("");

export async function loadAppVersion(): Promise<void> {
  try {
    const version = await invoke<string>("get_app_version");
    appVersion.set(version);
  } catch {
    appVersion.set("");
  }
}
