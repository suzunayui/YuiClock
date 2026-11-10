// Text and colours shared by the clock and the settings preview.
const pad = (n) => String(n).padStart(2, "0");

const WEEKDAY_NAMES = {
  "short-ja": ["日", "月", "火", "水", "木", "金", "土"],
  "long-ja": ["日曜日", "月曜日", "火曜日", "水曜日", "木曜日", "金曜日", "土曜日"],
  "long-en": ["Sunday", "Monday", "Tuesday", "Wednesday", "Thursday", "Friday", "Saturday"],
  "short-en": ["Sun", "Mon", "Tue", "Wed", "Thu", "Fri", "Sat"],
};

function formatDate(now, weekdayFormat) {
  const names = WEEKDAY_NAMES[weekdayFormat] ?? WEEKDAY_NAMES["short-ja"];
  return `${now.getFullYear()}/${pad(now.getMonth() + 1)}/${pad(now.getDate())} (${names[now.getDay()]})`;
}

function formatTime(now, showSeconds) {
  const time = `${pad(now.getHours())}:${pad(now.getMinutes())}`;
  return showSeconds ? `${time}:${pad(now.getSeconds())}` : time;
}

const weekdayName = (now, weekdayFormat) => (WEEKDAY_NAMES[weekdayFormat] ?? WEEKDAY_NAMES["short-ja"])[now.getDay()];
const ymd = (now) => `${now.getFullYear()}/${pad(now.getMonth() + 1)}/${pad(now.getDate())}`;

// The pieces of the clock that can be shown on their own in OBS, in display order within
// each group: "set" (several pieces together), "part" (one piece) and "symbol". Symbols
// carry a spelled-out name, since "/" alone is hard to spot in OBS's source list.
const CLOCK_PARTS = {
  date: { group: "set", label: "年月日", text: (now) => ymd(now) },
  dateWeekday: { group: "set", label: "年月日(曜日)", text: (now, weekdayFormat) => formatDate(now, weekdayFormat) },
  weekdayParen: { group: "set", label: "(曜日)", text: (now, weekdayFormat) => `(${weekdayName(now, weekdayFormat)})` },
  time: { group: "set", label: "時分秒", text: (now) => formatTime(now, true) },
  timeShort: { group: "set", label: "時分", text: (now) => formatTime(now, false) },
  year: { group: "part", label: "年", text: (now) => String(now.getFullYear()) },
  month: { group: "part", label: "月", text: (now) => pad(now.getMonth() + 1) },
  day: { group: "part", label: "日", text: (now) => pad(now.getDate()) },
  weekday: { group: "part", label: "曜日", text: weekdayName },
  hour: { group: "part", label: "時", text: (now) => pad(now.getHours()) },
  minute: { group: "part", label: "分", text: (now) => pad(now.getMinutes()) },
  second: { group: "part", label: "秒", text: (now) => pad(now.getSeconds()) },
  slash: { group: "symbol", label: "/", name: "スラッシュ", text: () => "/" },
  colon: { group: "symbol", label: ":", name: "コロン", text: () => ":" },
  open: { group: "symbol", label: "(", name: "カッコ開き", text: () => "(" },
  close: { group: "symbol", label: ")", name: "カッコ閉じ", text: () => ")" },
};

// Size the date line so it is exactly as wide as the time line, or reset it to `baseSize`
// (e.g. "0.5em") when not aligning. Works in whatever unit `baseSize` uses.
function alignDateWidth(dateEl, timeEl, baseSize, align) {
  dateEl.style.fontSize = baseSize;
  if (!align || dateEl.hidden) return;
  const dateWidth = dateEl.getBoundingClientRect().width;
  const timeWidth = timeEl.getBoundingClientRect().width;
  if (!dateWidth || !timeWidth) return;
  const [, amount, unit] = baseSize.match(/^([\d.]+)(\D+)$/);
  dateEl.style.fontSize = `${(Number(amount) * timeWidth) / dateWidth}${unit}`;
}

// "#rrggbb" plus a transparency of 0 (opaque) to 100 (invisible), as a CSS colour.
function withTransparency(hex, transparency) {
  const n = parseInt(hex.slice(1), 16);
  const alpha = 1 - Math.min(Math.max(transparency ?? 0, 0), 100) / 100;
  return `rgba(${(n >> 16) & 255}, ${(n >> 8) & 255}, ${n & 255}, ${alpha})`;
}
