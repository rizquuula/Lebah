<script lang="ts">
  import Board from "./lib/components/Board.svelte";
</script>

<main>
  <header>
    <div class="logo-group">
      <div class="logo-icon">
        <svg width="28" height="28" viewBox="0 0 24 24" fill="none">
          <path d="M12 2L2 7l10 5 10-5-10-5z" fill="url(#g1)" opacity="0.9"/>
          <path d="M2 17l10 5 10-5" stroke="url(#g2)" stroke-width="1.5" fill="none" opacity="0.7"/>
          <path d="M2 12l10 5 10-5" stroke="url(#g2)" stroke-width="1.5" fill="none" opacity="0.85"/>
          <defs>
            <linearGradient id="g1" x1="2" y1="2" x2="22" y2="12">
              <stop offset="0%" stop-color="#89b4fa"/>
              <stop offset="100%" stop-color="#b4befe"/>
            </linearGradient>
            <linearGradient id="g2" x1="2" y1="12" x2="22" y2="22">
              <stop offset="0%" stop-color="#89b4fa"/>
              <stop offset="100%" stop-color="#cba6f7"/>
            </linearGradient>
          </defs>
        </svg>
      </div>
      <h1>LebahTempa</h1>
      <span class="subtitle">Claude Code Orchestrator</span>
    </div>
    <div class="header-glow"></div>
  </header>
  <Board />
</main>

<style>
  :global(*) {
    box-sizing: border-box;
  }
  :global(body) {
    margin: 0;
    background: #0a0a1a;
    font-family: 'Inter', -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif;
    color: #cdd6f4;
    overflow: hidden;
  }
  :global(body::before) {
    content: '';
    position: fixed;
    inset: 0;
    background:
      radial-gradient(ellipse 800px 600px at 20% 20%, rgba(137, 180, 250, 0.08) 0%, transparent 70%),
      radial-gradient(ellipse 600px 400px at 80% 80%, rgba(203, 166, 247, 0.06) 0%, transparent 70%),
      radial-gradient(ellipse 500px 500px at 50% 50%, rgba(166, 227, 161, 0.04) 0%, transparent 70%);
    pointer-events: none;
    z-index: 0;
    animation: bgShift 20s ease-in-out infinite alternate;
  }
  @keyframes bgShift {
    0% { opacity: 1; }
    50% { opacity: 0.7; }
    100% { opacity: 1; }
  }
  :global(::-webkit-scrollbar) {
    width: 6px;
    height: 6px;
  }
  :global(::-webkit-scrollbar-track) {
    background: transparent;
  }
  :global(::-webkit-scrollbar-thumb) {
    background: rgba(137, 180, 250, 0.2);
    border-radius: 3px;
  }
  :global(::-webkit-scrollbar-thumb:hover) {
    background: rgba(137, 180, 250, 0.35);
  }
  main {
    height: 100vh;
    display: flex;
    flex-direction: column;
    position: relative;
    z-index: 1;
  }
  header {
    display: flex;
    align-items: center;
    padding: 14px 20px;
    background: rgba(24, 24, 37, 0.6);
    backdrop-filter: blur(20px) saturate(1.5);
    -webkit-backdrop-filter: blur(20px) saturate(1.5);
    border-bottom: 1px solid rgba(137, 180, 250, 0.1);
    position: relative;
    overflow: hidden;
    animation: headerSlide 0.6s ease-out;
  }
  @keyframes headerSlide {
    from { transform: translateY(-100%); opacity: 0; }
    to { transform: translateY(0); opacity: 1; }
  }
  .header-glow {
    position: absolute;
    bottom: -1px;
    left: 0;
    right: 0;
    height: 1px;
    background: linear-gradient(90deg, transparent, rgba(137, 180, 250, 0.4), rgba(203, 166, 247, 0.3), transparent);
    animation: glowPulse 3s ease-in-out infinite;
  }
  @keyframes glowPulse {
    0%, 100% { opacity: 0.5; }
    50% { opacity: 1; }
  }
  .logo-group {
    display: flex;
    align-items: center;
    gap: 12px;
  }
  .logo-icon {
    display: flex;
    align-items: center;
    animation: logoFloat 4s ease-in-out infinite;
  }
  @keyframes logoFloat {
    0%, 100% { transform: translateY(0); }
    50% { transform: translateY(-2px); }
  }
  h1 {
    margin: 0;
    font-size: 20px;
    font-weight: 700;
    background: linear-gradient(135deg, #89b4fa, #b4befe, #cba6f7);
    -webkit-background-clip: text;
    -webkit-text-fill-color: transparent;
    background-clip: text;
  }
  .subtitle {
    font-size: 13px;
    color: rgba(108, 112, 134, 0.8);
    padding-left: 4px;
    border-left: 1px solid rgba(108, 112, 134, 0.3);
  }
</style>
