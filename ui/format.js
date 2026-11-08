// Date and time text shared by the clock and the settings preview.
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
