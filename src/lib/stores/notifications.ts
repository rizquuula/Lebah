import { writable } from "svelte/store";

export const notification = writable<string | null>(null);

let _timer: ReturnType<typeof setTimeout> | null = null;

export function showNotification(msg: string, duration = 3000): void {
  if (_timer) {
    clearTimeout(_timer);
    _timer = null;
  }
  notification.set(msg);
  _timer = setTimeout(() => {
    notification.set(null);
    _timer = null;
  }, duration);
}

export function clearNotification(): void {
  if (_timer) {
    clearTimeout(_timer);
    _timer = null;
  }
  notification.set(null);
}
