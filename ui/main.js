const { invoke } = window.__TAURI__.core;
const { listen } = window.__TAURI__.event;

const root = document.documentElement;
const face = document.getElementById("face");
const date = document.getElementById("date");
const clock = document.getElementById("clock");
const pad = (n) => String(n).padStart(2, "0");
const WEEKDAYS = ["日", "月", "火", "水", "木", "金", "土"];

let showSeconds = true;
let cornerRadius = 0;

function render() {
  const now = new Date();
  date.textContent = `${now.getFullYear()}/${pad(now.getMonth() + 1)}/${pad(now.getDate())} (${WEEKDAYS[now.getDay()]})`;
  const time = `${pad(now.getHours())}:${pad(now.getMinutes())}`;
  clock.textContent = showSeconds ? `${time}:${pad(now.getSeconds())}` : time;
  fit();
}

// Scale the face to fill the window, whatever font and lines are showing.
function fit() {
  // 100% rounds the ends into full semicircles: radius is half the shorter side.
  const radius = (Math.min(innerWidth, innerHeight) / 2) * (cornerRadius / 100);
  root.style.setProperty("--radius", `${radius}px`);

  const base = 100;
  face.style.fontSize = `${base}px`;
  const { width, height } = face.getBoundingClientRect();
  if (!width || !height) return;
  const scale = Math.min((innerWidth * 0.92) / width, (innerHeight * 0.88) / height);
  face.style.fontSize = `${base * scale}px`;
}

function apply(settings) {
  const name = settings.fontFamily.replace(/["\\]/g, "");
  root.style.setProperty("--fg", settings.textColor);
  root.style.setProperty("--bg", settings.backgroundColor);
  root.style.setProperty("--font", `${name ? `"${name}", ` : ""}"Yu Gothic UI", monospace`);
  root.style.setProperty("--weight", settings.bold ? "700" : "400");
  date.hidden = !settings.showDate;
  showSeconds = settings.showSeconds;
  cornerRadius = settings.cornerRadius;
  render();
}

function tick() {
  render();
  // Aim for the next whole second so the display never drifts.
  setTimeout(tick, 1000 - new Date().getMilliseconds());
}

window.addEventListener("resize", fit);
window.addEventListener("contextmenu", (event) => {
  event.preventDefault();
  invoke("show_menu");
});

listen("settings-changed", (event) => apply(event.payload));
invoke("get_settings").then(apply).finally(tick);
