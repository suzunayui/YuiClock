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
    textTransparency: Number(el.textTransparency.value),
    backgroundColor: el.backgroundColor.value,
    backgroundTransparency: Number(el.backgroundTransparency.value),
    fontFamily: el.fontFamily.value,
    bold: el.bold.checked,
    showDate: el.showDate.checked,
    weekdayFormat: el.weekdayFormat.value,
    showSeconds: el.showSeconds.checked,
    alignWidths: el.alignWidths.checked,
    cornerRadius: Number(el.cornerRadius.value),
    alwaysOnTop: el.alwaysOnTop.checked,
  };
}

function refresh() {
  const s = collect();
  for (const output of document.querySelectorAll("output[data-for]")) {
    output.textContent = `${s[output.dataset.for]}%`;
  }

  const now = new Date();
  previewDate.textContent = formatDate(now, s.weekdayFormat);
  previewTime.textContent = formatTime(now, s.showSeconds);
  previewDate.hidden = !s.showDate;
  document.getElementById("weekdayChoices").disabled = !s.showDate;
  form.elements.alignWidths.disabled = !s.showDate;

  preview.style.fontFamily = `"${s.fontFamily.replace(/["\\]/g, "")}", "Yu Gothic UI", monospace`;
  preview.style.fontWeight = s.bold ? "700" : "400";
  preview.style.color = withTransparency(s.textColor, s.textTransparency);
  // A checkerboard behind the colour shows how see-through the background is.
  const bg = withTransparency(s.backgroundColor, s.backgroundTransparency);
  preview.style.background = `linear-gradient(${bg}, ${bg}), repeating-conic-gradient(#c8c8c8 0 25%, #f0f0f0 0 50%) 0 0 / 14px 14px`;
  // After the font is set, since the match depends on the glyphs' widths.
  alignDateWidth(previewDate, previewTime, "16px", s.alignWidths);
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

// A source size that fits one piece snugly: wide CJK glyphs count as a full em, the rest
// (digits and Latin letters) roughly 0.62em. The clock page scales its text to fill whatever
// size this gives, leaving the same margins as the whole clock.
function partSize(part, weekdayFormat) {
  const height = 200;
  const ems = (text) => [...text].reduce((sum, ch) => sum + (ch.charCodeAt(0) > 0x2e80 ? 1 : 0.62), 0);
  // A week of sample dates, so anything showing the weekday is sized for its longest name.
  const texts = [...Array(7).keys()].map((i) => CLOCK_PARTS[part].text(new Date(2026, 10, 1 + i), weekdayFormat));
  const widest = Math.max(...texts.map(ems));
  const fontSize = height * 0.88;
  const width = Math.ceil((fontSize * widest) / 0.92) + 8;
  // Keep long sets like "2026/11/08 (Sunday)" well inside a 1920-wide canvas; the page keeps
  // the same proportions at any size, so shrinking both sides changes nothing else.
  const maxWidth = 1280;
  return width <= maxWidth ? [width, height] : [maxWidth, Math.round((height * maxWidth) / width)];
}

async function setUpObs() {
  const info = await invoke("obs_info");
  const link = document.getElementById("obsLink");
  document.getElementById("obsUrl").textContent = info.url;
  if (info.error) {
    const error = document.getElementById("obsError");
    error.textContent = info.error;
    error.hidden = false;
    document.getElementById("obsDrag").hidden = true;
    document.querySelector(".obs-actions").hidden = true;
    document.getElementById("obsHelp").hidden = true;
    return;
  }

  // OBS turns a dropped link into a browser source, reading the layer-* parameters as its
  // name and size and keeping the rest (our "part") in the URL it stores.
  const dragUrl = (name, width, height, part) => {
    const params = new URLSearchParams();
    if (part) params.set("part", part);
    params.set("layer-name", name);
    params.set("layer-width", String(width));
    params.set("layer-height", String(height));
    // OBS decodes the name without turning "+" back into a space.
    return `${info.url}?${params.toString().replaceAll("+", "%20")}`;
  };

  const makeDraggable = (element, urlNow) => {
    element.href = urlNow();
    // The link is only for dragging; following it would navigate this window away.
    element.addEventListener("click", (event) => event.preventDefault());
    element.addEventListener("dragstart", (event) => {
      const url = urlNow();
      event.dataTransfer.effectAllowed = "copyLink";
      event.dataTransfer.setData("text/uri-list", url);
      event.dataTransfer.setData("text/plain", url);
    });
  };

  makeDraggable(link, () => dragUrl("YuiClock", info.width, info.height));

  for (const [key, { group, label, name }] of Object.entries(CLOCK_PARTS)) {
    const chip = document.createElement("a");
    chip.className = "drag";
    chip.draggable = true;
    chip.textContent = label;
    if (name) chip.title = name;
    makeDraggable(chip, () => {
      const [width, height] = partSize(key, collect().weekdayFormat);
      return dragUrl(`YuiClock ${name ?? label}`, width, height, key);
    });
    document.querySelector(`.parts[data-group="${group}"]`).append(chip);
  }

  const copied = document.getElementById("copied");
  document.getElementById("copyUrl").addEventListener("click", async () => {
    await navigator.clipboard.writeText(info.url);
    copied.hidden = false;
    setTimeout(() => (copied.hidden = true), 2000);
  });
}

// Size the window to the page. Platforms disagree on whether a window's height includes its
// title bar, so after asking for the page's height, add whatever is still missing.
async function fitWindow() {
  // Measure <body> itself: scrollHeight never reports less than the current window height.
  let height = document.body.getBoundingClientRect().height;
  for (let attempt = 0; attempt < 4; attempt++) {
    await invoke("fit_window_height", { height });
    // Give the resize a moment to reach the page.
    await new Promise((resolve) => setTimeout(resolve, 120));
    const missing = document.documentElement.scrollHeight - window.innerHeight;
    if (missing <= 0) return;
    height += missing;
  }
}

(async () => {
  fonts = await invoke("list_fonts");
  fill(await invoke("get_settings"));
  await setUpObs();
  // Size the window to its content so nothing ever needs scrolling, whatever the
  // system font size or display scaling.
  await document.fonts.ready;
  await fitWindow();
})();
