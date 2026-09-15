// Finish the generated platform packages before publishing: make each one
// pack NOTICE next to the binding, and point the main package at them,
// refusing any platform package whose version differs from the main one.
import fs from 'node:fs';
import path from 'node:path';

const [packageDir, npmDir] = process.argv.slice(2);
const mainPath = path.join(packageDir, 'package.json');
const main = JSON.parse(fs.readFileSync(mainPath, 'utf8'));
main.optionalDependencies = {};
for (const dir of fs.readdirSync(npmDir).sort()) {
  const platformPath = path.join(npmDir, dir, 'package.json');
  const platform = JSON.parse(fs.readFileSync(platformPath, 'utf8'));
  if (platform.version !== main.version) {
    throw new Error(`${platform.name} is ${platform.version}, main package is ${main.version}`);
  }
  platform.files = [...new Set([...(platform.files ?? []), 'LICENSE', 'NOTICE'])];
  fs.writeFileSync(platformPath, JSON.stringify(platform, null, 2) + '\n');
  main.optionalDependencies[platform.name] = platform.version;
}
fs.writeFileSync(mainPath, JSON.stringify(main, null, 2) + '\n');
console.log(main.optionalDependencies);
