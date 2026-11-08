const fs = require('node:fs');
const path = require('node:path');
const {spawnSync, execFileSync} = require('node:child_process');
const root = path.resolve(__dirname, '..');

function run(file, args, options = {}) {
  const result = spawnSync(file, args, {cwd: root, stdio: 'inherit', windowsHide: true, ...options});
  if (result.error) throw result.error;
  if (result.status !== 0) throw new Error(`${path.basename(file)} exited with code ${result.status}`);
}
function msvcEnvironment() {
  const env = {...process.env};
  if (process.platform !== 'win32' || env.VCINSTALLDIR) return env;
  const vswhere = path.join(env['ProgramFiles(x86)'] || 'C:/Program Files (x86)', 'Microsoft Visual Studio/Installer/vswhere.exe');
  if (!fs.existsSync(vswhere)) return env;
  const install = execFileSync(vswhere, ['-latest', '-all', '-products', '*', '-property', 'installationPath', '-utf8'], {encoding: 'utf8', windowsHide: true}).trim();
  if (!install) return env;
  const vcvars = path.join(install, 'VC/Auxiliary/Build/vcvars64.bat');
  if (!fs.existsSync(vcvars)) return env;
  const output = execFileSync(env.ComSpec || 'cmd.exe', ['/u', '/d', '/s', '/c', 'tools\\msvc-env.cmd'], {
    cwd: root, encoding: 'utf16le', windowsHide: true, env: {...env, KYRO_VCVARS: vcvars},
  });
  for (const line of output.split(/\r?\n/)) {
    const split = line.indexOf('=');
    if (split > 0) {
      const key = line.slice(0, split);
      const previous = Object.keys(env).find(name => name.toLowerCase() === key.toLowerCase());
      if (previous) delete env[previous];
      env[key] = line.slice(split + 1);
    }
  }
  delete env.KYRO_VCVARS;
  return env;
}
function build(mode = 'release', {output = 'KyroLauncher.exe', networkTests = false} = {}) {
  if (!['check', 'test', 'debug', 'release'].includes(mode)) throw new Error('Mode must be check, test, debug or release');
  if (!/^[A-Za-z0-9 ._-]+\.exe$/.test(output)) throw new Error('Output must be an EXE filename');
  const env = msvcEnvironment();
  const localCargo = path.join(process.env.USERPROFILE || process.env.HOME || '', '.cargo/bin/cargo.exe');
  const cargo = fs.existsSync(localCargo) ? localCargo : 'cargo';
  const args = [mode === 'debug' || mode === 'release' ? 'build' : mode, '--locked'];
  if (mode === 'release') args.push('--release');
  if (mode === 'test' && networkTests) args.push('--', '--include-ignored');
  run(cargo, args, {env});
  if (mode !== 'debug' && mode !== 'release') return;
  const destination = path.join(root, 'dist');
  fs.mkdirSync(destination, {recursive: true});
  for (const name of ['LICENSE', 'APPLICATION_LICENSE.md', 'THIRD_PARTY_NOTICES.md']) {
    fs.copyFileSync(path.join(root, name), path.join(destination, name));
  }
  fs.copyFileSync(path.join(root, 'ui/fonts/OFL.txt'), path.join(destination, 'Inter-LICENSE.txt'));
  const target = path.join(destination, output);
  fs.copyFileSync(path.join(root, 'target', mode, 'kyro-launcher.exe'), target);
  console.log(`Built: ${target}`);
}
if (require.main === module) {
  try {
    const args = process.argv.slice(2);
    const mode = args.shift() || 'release';
    const options = {};
    while (args.length) {
      const option = args.shift();
      if (option === '--output') { options.output = args.shift(); if (!options.output) throw new Error('--output requires a filename'); }
      else if (option === '--network-tests') options.networkTests = true;
      else throw new Error(`Unknown option: ${option}`);
    }
    build(mode, options);
  } catch (error) { console.error(error.message); process.exitCode = 1; }
}
module.exports = {root, run, build};
