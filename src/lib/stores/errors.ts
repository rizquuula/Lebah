import { writable } from "svelte/store";

export const lastError = writable<string | null>(null);
export const setError = (msg: string) => lastError.set(msg);
export const clearError = () => lastError.set(null);
