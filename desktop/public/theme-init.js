// Runs before first paint so the window never flashes the wrong theme.
// Keep the storage key and the resolution rule in sync with src/lib/theme.svelte.ts.
(() => {
  let preference = "system";
  try {
    preference = localStorage.getItem("voice-studio-theme") || "system";
  } catch {
    // Storage can be unavailable; fall back to following the system.
  }
  const systemDark = matchMedia("(prefers-color-scheme: dark)").matches;
  const dark = preference === "dark" || (preference === "system" && systemDark);
  document.documentElement.dataset.theme = dark ? "dark" : "light";
})();
