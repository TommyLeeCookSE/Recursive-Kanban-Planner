import { defineConfig } from '@playwright/test';

const port = 3921;
const baseURL = `http://127.0.0.1:${port}`;

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
    command: `dx serve --platform web --port ${port} --open false`,
    url: baseURL,
    reuseExistingServer: true,
    timeout: 120_000,
  },
});
