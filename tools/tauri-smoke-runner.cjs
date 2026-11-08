const fs = require('node:fs');
const path = require('node:path');
const {randomUUID} = require('node:crypto');
const {spawn} = require('node:child_process');
const {root} = require('./build.cjs');

function wait(child) {
  return new Promise((resolve, reject) => {
    child.once('error', reject);
    child.once('exit', code => code === 0 ? resolve() : reject(new Error(`Process exited with code ${code}`)));
  });
}
(async () => {
  const args = process.argv.slice(2);
  if (args.length > 1) throw new Error('Usage: node tools/tauri-smoke-runner.cjs [exe-path]');
  const exe = path.resolve(root, args[0] || 'target/debug/kyro-launcher.exe');
  if (!fs.existsSync(exe)) throw new Error('Build the launcher before running this test');
  const fixture = path.join(root, 'target/tauri-ui-smoke', randomUUID());
  fs.mkdirSync(fixture, {recursive: true});
  fs.mkdirSync(path.join(root, 'output/app-preview'), {recursive: true});
  const app = spawn(exe, [], {cwd: root, windowsHide: true, stdio: 'ignore', env: {
    ...process.env, KYRO_DATA_DIR: fixture, WEBVIEW2_USER_DATA_FOLDER: path.join(fixture, 'webview'),
    WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS: '--remote-debugging-port=9228',
  }});
  let startupError;
  app.on('error', error => { startupError = error; });
  try {
    await new Promise((resolve, reject) => { app.once('spawn', resolve); app.once('error', reject); });
    await wait(spawn(process.execPath, [path.join(__dirname, 'tauri-smoke.cjs')], {cwd: root, stdio: 'inherit', windowsHide: true}));
    if (startupError) throw startupError;
  } finally {
    if (app.pid && app.exitCode === null) {
      const exited = new Promise(resolve => app.once('exit', resolve));
      app.kill();
      await exited;
    }
  }
})().catch(error => { console.error(error.message); process.exitCode = 1; });
