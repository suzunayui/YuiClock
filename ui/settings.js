const { invoke } = window.__TAURI__.core;
const form = document.getElementById("form");
const fontSelect = form.elements.fontFamily;
const preview = document.getElementById("preview");
const previewDate = document.getElementById("previewDate");
const previewTime = document.getElementById("previewTime");

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
    weekdayFormat: el.weekdayFormat.value,
    showSeconds: el.showSeconds.checked,
    cornerRadius: Number(el.cornerRadius.value),
    alwaysOnTop: el.alwaysOnTop.checked,
  };
}

function refresh() {
  const s = collect();
  document.getElementById("radiusValue").textContent = `${s.cornerRadius}%`;

  const now = new Date();
  previewDate.textContent = formatDate(now, s.weekdayFormat);
  previewTime.textContent = formatTime(now, s.showSeconds);
  previewDate.hidden = !s.showDate;
  document.getElementById("weekdayChoices").disabled = !s.showDate;

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

async function setUpObs() {
  const info = await invoke("obs_info");
  const link = document.getElementById("obsLink");
  document.getElementById("obsUrl").textContent = info.url;
  if (info.error) {
    const error = document.getElementById("obsError");
    error.textContent = info.error;
    error.hidden = false;
    document.querySelector(".obs-actions").hidden = true;
    document.getElementById("obsHelp").hidden = true;
    return;
  }

  link.href = info.dragUrl;
  // The link is only for dragging; following it would navigate this window away.
  link.addEventListener("click", (event) => event.preventDefault());
  link.addEventListener("dragstart", (event) => {
    event.dataTransfer.effectAllowed = "copyLink";
    event.dataTransfer.setData("text/uri-list", info.dragUrl);
    event.dataTransfer.setData("text/plain", info.dragUrl);
  });

  const copied = document.getElementById("copied");
  document.getElementById("copyUrl").addEventListener("click", async () => {
    await navigator.clipboard.writeText(info.url);
    copied.hidden = false;
    setTimeout(() => (copied.hidden = true), 2000);
  });
}

(async () => {
  fonts = await invoke("list_fonts");
  fill(await invoke("get_settings"));
  await setUpObs();
  // Size the window to its content so nothing ever needs scrolling, whatever the
  // system font size or display scaling.
  await document.fonts.ready;
  // Measure <body> itself: scrollHeight never reports less than the current window height.
  await invoke("fit_window_height", { height: document.body.getBoundingClientRect().height });
})();
