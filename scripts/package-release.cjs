'use strict';

// Collects what a GitHub Release needs into release/: the installer under its public
// name and its checksum, which YuiLauncher checks before installing.
const crypto = require('node:crypto');
const fs = require('node:fs');
const path = require('node:path');
const { version } = require('../package.json');

const root = path.join(__dirname, '..');
const targetDir = process.env.CARGO_TARGET_DIR || path.join(root, 'src-tauri', 'target');
const built = path.join(targetDir, 'release', 'bundle', 'nsis', `YuiClock_${version}_x64-setup.exe`);
const output = path.join(root, 'release');
const name = `YuiClock-Setup-${version}.exe`;

fs.rmSync(output, { recursive: true, force: true });
fs.mkdirSync(output, { recursive: true });
fs.copyFileSync(built, path.join(output, name));
const sha256 = crypto.createHash('sha256').update(fs.readFileSync(built)).digest('hex');
fs.writeFileSync(path.join(output, `SHA256SUMS-${version}.txt`), `${sha256}  ${name}\n`);
console.log(`Packaged ${name}`);
