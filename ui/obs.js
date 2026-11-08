// The page OBS shows as a browser source. It is served by YuiClock on 127.0.0.1 and picks up
// setting changes by polling; if YuiClock quits, it keeps ticking with the last settings.
let lastSettings = "";

async function pollSettings() {
  try {
    const response = await fetch("/settings", { cache: "no-store" });
    if (!response.ok) return;
    const text = await response.text();
    if (text !== lastSettings) {
      lastSettings = text;
      applySettings(JSON.parse(text));
    }
  } catch {
    // YuiClock isn't running right now; try again on the next poll.
  }
}

pollSettings().finally(startClock);
setInterval(pollSettings, 1000);
