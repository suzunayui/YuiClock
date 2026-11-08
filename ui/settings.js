const { invoke } = window.__TAURI__.core;
const form = document.getElementById("form");
const fontSelect = form.elements.fontFamily;
const preview = document.getElementById("preview");
const previewDate = document.getElementById("previewDate");
const previewTime = document.getElementById("previewTime");
const pad = (n) => String(n).padStart(2, "0");
const WEEKDAYS = ["日", "月", "火", "水", "木", "金", "土"];

const monoOnly = document.getElementById("monoOnly");
let fonts = [];

function addFontOption(name, label) {
  const option = document.createElement("option");
  option.value = name;
  option.textContent = label === name ? name : `${label}（${name}）`;
  fontSelect.append(option);
}

// Rebuild the font list; the chosen font always stays in it, even if it is
// proportional or has since been uninstalled.
function renderFonts(selected) {
  fontSelect.replaceChildren();
  const shown = fonts.filter((f) => !monoOnly.checked || f.monospace || f.name === selected);
  for (const f of shown) addFontOption(f.name, f.label);
  if (!shown.some((f) => f.name === selected)) addFontOption(selected, selected);
  fontSelect.value = selected;
}

monoOnly.addEventListener("change", (event) => {
  event.stopPropagation();
  renderFonts(fontSelect.value);
});

function fill(settings) {
  renderFonts(settings.fontFamily);
  for (const [key, value] of Object.entries(settings)) {
    const field = form.elements[key];
    if (!field) continue;
    if (field.type === "checkbox") field.checked = value;
    else field.value = value;
  }
  refresh();
}

function collect() {
  const el = form.elements;
  return {
    textColor: el.textColor.value,
    backgroundColor: el.backgroundColor.value,
    fontFamily: el.fontFamily.value,
    bold: el.bold.checked,
    showDate: el.showDate.checked,
    showSeconds: el.showSeconds.checked,
    cornerRadius: Number(el.cornerRadius.value),
    alwaysOnTop: el.alwaysOnTop.checked,
  };
}

function refresh() {
  const s = collect();
  document.getElementById("radiusValue").textContent = `${s.cornerRadius}%`;

  const now = new Date();
  previewDate.textContent = `${now.getFullYear()}/${pad(now.getMonth() + 1)}/${pad(now.getDate())} (${WEEKDAYS[now.getDay()]})`;
  const time = `${pad(now.getHours())}:${pad(now.getMinutes())}`;
  previewTime.textContent = s.showSeconds ? `${time}:${pad(now.getSeconds())}` : time;
  previewDate.hidden = !s.showDate;

  preview.style.fontFamily = `"${s.fontFamily.replace(/["\\]/g, "")}", "Yu Gothic UI", monospace`;
  preview.style.fontWeight = s.bold ? "700" : "400";
  preview.style.color = s.textColor;
  preview.style.background = s.backgroundColor;
}

const save = () => invoke("set_settings", { settings: collect() });

form.addEventListener("input", () => {
  refresh();
  save();
});

document.getElementById("reset").addEventListener("click", async () => {
  fill(await invoke("default_settings"));
  await save();
});

document.getElementById("close").addEventListener("click", () => {
  window.__TAURI__.window.getCurrentWindow().close();
});

setInterval(refresh, 1000);

// The right-click menu can flip "always on top" while this window is open.
window.__TAURI__.event.listen("settings-changed", (event) => {
  form.elements.alwaysOnTop.checked = event.payload.alwaysOnTop;
});

(async () => {
  fonts = await invoke("list_fonts");
  fill(await invoke("get_settings"));
})();
