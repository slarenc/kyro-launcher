const fs = require('node:fs');
const path = require('node:path');
const {root, run, build} = require('./build.cjs');

function installer({skipBuild = false, iscc, testRoot} = {}) {
  if (!skipBuild) build('release');
  const source = path.join(root, 'dist/KyroLauncher.exe');
  if (!fs.existsSync(source)) throw new Error('Build dist/KyroLauncher.exe first');
  const candidates = [iscc, path.join(root, '.tools/InnoSetup7/ISCC.exe')];
  for (const base of [process.env.LOCALAPPDATA && path.join(process.env.LOCALAPPDATA, 'Programs'), process.env.ProgramFiles, process.env['ProgramFiles(x86)']]) {
    if (base) for (const version of [7, 6]) candidates.push(path.join(base, `Inno Setup ${version}/ISCC.exe`));
  }
  for (const directory of (process.env.PATH || '').split(path.delimiter)) candidates.push(path.join(directory, 'ISCC.exe'));
  const compiler = candidates.find(candidate => candidate && fs.existsSync(candidate));
  if (!compiler) throw new Error('Install Inno Setup or pass --iscc <path>');
  const version = fs.readFileSync(path.join(root, 'src-tauri/Cargo.toml'), 'utf8').match(/^version\s*=\s*"(\d+\.\d+\.\d+)"/m)?.[1];
  if (!version) throw new Error('Cannot read launcher version');
  const args = ['/Qp', `/DAppVersion=${version}`, `/DSourceExe=${source}`];
  if (testRoot) {
    const resolved = path.resolve(testRoot);
    const lexical = path.relative(path.join(root, 'target'), resolved);
    if (!lexical || lexical.startsWith('..') || path.isAbsolute(lexical)) throw new Error('Test root must be inside workspace target directory');
    fs.mkdirSync(path.join(root, 'target'), {recursive: true});
    fs.mkdirSync(resolved, {recursive: true});
    const relative = path.relative(fs.realpathSync(path.join(root, 'target')), fs.realpathSync(resolved));
    if (!relative || relative.startsWith('..') || path.isAbsolute(relative)) throw new Error('Test root must be inside workspace target directory');
    args.push(`/DTestRoot=${resolved}`);
  }
  args.push(path.join(root, 'installer/KyroLauncher.iss'));
  run(compiler, args);
  console.log(`Built: ${path.join(root, 'dist', testRoot ? 'KyroLauncherSetup-Test.exe' : 'KyroLauncherSetup.exe')}`);
}
if (require.main === module) {
  try {
    const args = process.argv.slice(2), options = {};
    while (args.length) {
      const option = args.shift();
      if (option === '--skip-build') options.skipBuild = true;
      else if (option === '--iscc' || option === '--test-root') {
        const value = args.shift(); if (!value) throw new Error(`${option} requires a value`);
        options[option === '--iscc' ? 'iscc' : 'testRoot'] = value;
      } else throw new Error(`Unknown option: ${option}`);
    }
    installer(options);
  } catch (error) { console.error(error.message); process.exitCode = 1; }
}
module.exports = {installer};
