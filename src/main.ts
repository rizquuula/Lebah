import "./app.css";
import App from "./App.svelte";
import { mount } from "svelte";

async function start() {
  if (
    import.meta.env.DEV &&
    typeof window !== "undefined" &&
    new URLSearchParams(window.location.search).has("e2e")
  ) {
    const { installMockIpc } = await import("./lib/test/mock-ipc");
    installMockIpc();
  }
  return mount(App, {
    target: document.getElementById("app")!,
  });
}

const app = await start();

export default app;
