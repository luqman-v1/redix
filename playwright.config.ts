import { defineConfig } from "@playwright/test";
export default defineConfig({
  testDir: "./tests",
  // Unit tests in this directory run under `bun test`, not Playwright.
  testMatch: "**/*.spec.ts",
  timeout: 30000,
  use: {
    baseURL: "http://localhost:1420",
    headless: true,
  },
  webServer: {
    command: "bun dev",
    url: "http://localhost:1420",
    reuseExistingServer: true,
    timeout: 120000,
  },
});
