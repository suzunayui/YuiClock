// The desktop clock window: settings arrive from the Rust side.
const { invoke } = window.__TAURI__.core;
const { listen } = window.__TAURI__.event;

window.addEventListener("contextmenu", (event) => {
  event.preventDefault();
  invoke("show_menu");
});

listen("settings-changed", (event) => applySettings(event.payload));
invoke("get_settings").then(applySettings).finally(startClock);
