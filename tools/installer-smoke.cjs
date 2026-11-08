const fs = require('node:fs');
const path = require('node:path');
const assert = require('node:assert/strict');
const {createHash, randomUUID} = require('node:crypto');
const {execFileSync} = require('node:child_process');
const {root, run} = require('./build.cjs');
const {installer} = require('./installer.cjs');

try {
  const testRoot = path.join(root, 'target/installer-smoke', randomUUID());
  const appPath = path.join(testRoot, 'App');
  installer({skipBuild: true, testRoot});
  const exe = path.join(appPath, 'KyroLauncher.exe');
  const setup = path.join(root, 'dist/KyroLauncherSetup-Test.exe');
  const hash = file => createHash('sha256').update(fs.readFileSync(file)).digest('hex');
  const expected = hash(path.join(root, 'dist/KyroLauncher.exe'));
  const install = log => {
    run(setup, ['/VERYSILENT', '/SUPPRESSMSGBOXES', '/NORESTART', '/SP-', `/DIR=${appPath}`, `/LOG=${path.join(testRoot, log)}`]);
    assert.equal(hash(exe), expected, 'Installed EXE must match source');
    for (const folder of ['Desktop', 'StartMenu']) {
      const shortcut = path.join(testRoot, folder, 'KyroLauncher.lnk');
      assert(fs.existsSync(shortcut), `Missing shortcut: ${folder}`);
      const fields = execFileSync('cscript.exe', ['//nologo', path.join(__dirname, 'shortcut-info.vbs'), shortcut], {encoding: 'utf8', windowsHide: true})
        .trim().split(/\r?\n/).map(line => String.fromCharCode(...line.split(',').map(Number)));
      assert.equal(fields[0].toLowerCase(), exe.toLowerCase(), 'Shortcut target');
      assert.equal(fields[1].toLowerCase(), appPath.toLowerCase(), 'Shortcut working directory');
    }
  };
  install('install.log');
  const dataPath = path.join(appPath, 'data/state.json');
  fs.mkdirSync(path.dirname(dataPath), {recursive: true});
  const original = '{"installer_smoke":"user-created data must survive"}';
  fs.writeFileSync(dataPath, original);
  install('reinstall.log');
  assert.equal(fs.readFileSync(dataPath, 'utf8'), original, 'Reinstall preserves user data');
  const uninstaller = path.join(appPath, 'unins000.exe');
  const relative = path.relative(fs.realpathSync(testRoot), fs.realpathSync(uninstaller));
  if (relative.startsWith('..') || path.isAbsolute(relative)) throw new Error('Unsafe test uninstaller path');
  run(uninstaller, ['/VERYSILENT', '/SUPPRESSMSGBOXES', '/NORESTART', `/LOG=${path.join(testRoot, 'uninstall.log')}`]);
  assert(!fs.existsSync(exe), 'Uninstaller removes EXE');
  for (const folder of ['Desktop', 'StartMenu']) assert(!fs.existsSync(path.join(testRoot, folder, 'KyroLauncher.lnk')), 'Uninstaller removes shortcuts');
  assert.equal(fs.readFileSync(dataPath, 'utf8'), original, 'Uninstall preserves user data');
  console.log('PASS: installer, shortcut targets, reinstall, uninstall and preserved user data');
} catch (error) { console.error(error.message); process.exitCode = 1; }
