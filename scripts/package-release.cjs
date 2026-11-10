'use strict';

// Collects what a GitHub Release needs into release/: the installer under its public
// name and its checksum, which YuiLauncher checks before installing.
// The Mac zip from scripts/package-mac.sh goes in release/ first. Its hash joins the same
// checksum file: older launchers read only the first checksum file a release lists.
const crypto = require('node:crypto');
const fs = require('node:fs');
const path = require('node:path');
const { version } = require('../package.json');

const root = path.join(__dirname, '..');
const targetDir = process.env.CARGO_TARGET_DIR || path.join(root, 'src-tauri', 'target');
const built = path.join(targetDir, 'release', 'bundle', 'nsis', `YuiClock_${version}_x64-setup.exe`);
const output = path.join(root, 'release');
const name = `YuiClock-Setup-${version}.exe`;
const macName = `YuiClock-${version}-mac-arm64.zip`;

fs.mkdirSync(output, { recursive: true });
for (const file of fs.readdirSync(output)) {
  if (file !== macName) fs.rmSync(path.join(output, file), { recursive: true, force: true });
}
fs.copyFileSync(built, path.join(output, name));
const sha256 = data => crypto.createHash('sha256').update(data).digest('hex');
const sums = [`${sha256(fs.readFileSync(built))}  ${name}`];
const macPath = path.join(output, macName);
if (fs.existsSync(macPath)) {
  sums.push(`${sha256(fs.readFileSync(macPath))}  ${macName}`);
} else {
  console.warn(`No ${macName} in release/: this release will have no Mac version.`);
}
fs.writeFileSync(path.join(output, `SHA256SUMS-${version}.txt`), `${sums.join('\n')}\n`);
console.log(`Packaged ${name}${fs.existsSync(macPath) ? ` and ${macName}` : ''}`);
