<script lang="ts">
  import Board from "./lib/components/Board.svelte";
</script>

<main>
  <header>
    <h1>LebahTempa</h1>
    <span class="subtitle">Claude Code Orchestrator</span>
  </header>
  <Board />
</main>

<style>
  :global(body) {
    margin: 0;
    background: #11111b;
    font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif;
    color: #cdd6f4;
  }
  main {
    height: 100vh;
    display: flex;
    flex-direction: column;
  }
  header {
    display: flex;
    align-items: baseline;
    gap: 12px;
    padding: 12px 16px;
    background: #181825;
    border-bottom: 1px solid #313244;
  }
  h1 {
    margin: 0;
    font-size: 20px;
    font-weight: 700;
    color: #89b4fa;
  }
  .subtitle {
    font-size: 13px;
    color: #6c7086;
  }
</style>
