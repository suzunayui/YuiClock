// Draws the clock face. Shared by the desktop window (main.js) and the OBS page (obs.js),
// which only differ in how they receive settings.
const root = document.documentElement;
const face = document.getElementById("face");
const date = document.getElementById("date");
const clock = document.getElementById("clock");

// An OBS source added for one piece (e.g. "?part=year") shows only that piece.
const part = CLOCK_PARTS[new URLSearchParams(location.search).get("part")];

let showSeconds = true;
let showDate = true;
let alignWidths = true;
let weekdayFormat = "short-ja";
let cornerRadius = 0;

function render() {
  const now = new Date();
  if (part) {
    clock.textContent = part.text(now, weekdayFormat);
  } else {
    date.textContent = formatDate(now, weekdayFormat);
    clock.textContent = formatTime(now, showSeconds);
  }
  date.hidden = part ? true : !showDate;
  fit();
}

// Scale the face to fill the window, whatever font and lines are showing.
function fit() {
  // 100% rounds the ends into full semicircles: radius is half the shorter side.
  const radius = (Math.min(innerWidth, innerHeight) / 2) * (cornerRadius / 100);
  root.style.setProperty("--radius", `${radius}px`);

  const base = 100;
  face.style.fontSize = `${base}px`;
  // Sized in em, so the date keeps its match with the time when the face scales below.
  alignDateWidth(date, clock, "0.5em", alignWidths && !part);
  const { width, height } = face.getBoundingClientRect();
  if (!width || !height) return;
  const scale = Math.min((innerWidth * 0.92) / width, (innerHeight * 0.88) / height);
  face.style.fontSize = `${base * scale}px`;
}

function applySettings(settings) {
  const name = settings.fontFamily.replace(/["\\]/g, "");
  root.style.setProperty("--fg", withTransparency(settings.textColor, settings.textTransparency));
  root.style.setProperty("--bg", withTransparency(settings.backgroundColor, settings.backgroundTransparency));
  root.style.setProperty("--font", `${name ? `"${name}", ` : ""}"Yu Gothic UI", monospace`);
  root.style.setProperty("--weight", settings.bold ? "700" : "400");
  showDate = settings.showDate;
  showSeconds = settings.showSeconds;
  alignWidths = settings.alignWidths ?? true;
  weekdayFormat = settings.weekdayFormat;
  cornerRadius = settings.cornerRadius;
  render();
}

function tick() {
  render();
  // Aim for the next whole second so the display never drifts.
  setTimeout(tick, 1000 - new Date().getMilliseconds());
}

function startClock() {
  window.addEventListener("resize", fit);
  tick();
}
