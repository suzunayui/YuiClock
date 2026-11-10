// The page OBS shows as a browser source. YuiClock writes it, with settings.json beside it,
// into its config folder, and OBS opens it as a local file. The page re-reads settings.json
// every second, so it follows changes while YuiClock runs and keeps the last ones otherwise.
let lastSettings = "";

async function pollSettings() {
  try {
    // OBS serves local files by path alone, so the query only keeps the browser from caching.
    const response = await fetch(`settings.json?t=${Date.now()}`, { cache: "no-store" });
    if (!response.ok) return;
    const text = await response.text();
    if (text !== lastSettings) {
      lastSettings = text;
      applySettings(JSON.parse(text));
    }
  } catch {
    // Mid-write or unreadable for a moment; the next poll will pick it up.
  }
}

pollSettings().finally(startClock);
setInterval(pollSettings, 1000);
