import { execSync } from 'node:child_process';
import { mkdtempSync, readFileSync } from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const rootDir = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
const tempDir = mkdtempSync(path.join(os.tmpdir(), 'recursive-kanban-css-'));
const generatedPath = path.join(tempDir, 'app.css');
const command = `tailwindcss -c ./tailwind.config.js -i ./src/interface/tailwind.css -o "${generatedPath}"`;

execSync(command, {
  cwd: rootDir,
  stdio: 'inherit',
  shell: true,
});

const generated = readFileSync(generatedPath, 'utf8');
const committed = readFileSync(path.join(rootDir, 'assets', 'app.css'), 'utf8');

if (generated !== committed) {
  console.error('Generated CSS does not match assets/app.css. Run `npm run build:css` and commit the result.');
  process.exit(1);
}
