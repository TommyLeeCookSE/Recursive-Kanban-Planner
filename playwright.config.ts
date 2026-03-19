import { defineConfig } from '@playwright/test';
import { existsSync } from 'node:fs';
import { join } from 'node:path';

const port = Number(process.env.PLAYWRIGHT_WEB_PORT ?? 43129);
const baseURL = `http://127.0.0.1:${port}`;

function resolveDxCommand(): string {
  const cargoHome = process.env.CARGO_HOME;
  const userProfile = process.env.USERPROFILE;
  const home = process.env.HOME;
  const executableName = process.platform === 'win32' ? 'dx.exe' : 'dx';
  const candidates = [
    cargoHome ? join(cargoHome, 'bin', executableName) : null,
    userProfile ? join(userProfile, '.cargo', 'bin', executableName) : null,
    home ? join(home, '.cargo', 'bin', executableName) : null,
  ].filter((candidate): candidate is string => Boolean(candidate));

  const resolved = candidates.find((candidate) => existsSync(candidate));
  return resolved ?? 'dx';
}

const dxCommand = resolveDxCommand();

export default defineConfig({
  testDir: './tests',
  fullyParallel: false,
  timeout: 60_000,
  expect: {
    timeout: 10_000,
  },
  use: {
    baseURL,
    headless: true,
    viewport: { width: 1440, height: 900 },
  },
  webServer: {
    command: `"${dxCommand}" serve --platform web --port ${port} --open false`,
    url: baseURL,
    reuseExistingServer: !process.env.CI,
    timeout: 300_000,
  },
});
