import { defineConfig, devices } from "@playwright/test";

const baseURL = "http://127.0.0.1:8123";

export default defineConfig({
  testDir: "e2e",
  workers: 1,
  fullyParallel: false,
  forbidOnly: !!process.env.CI,
  reporter: process.env.CI
    ? [["list"], ["html", { open: "never" }]]
    : [["list"]],
  use: {
    ...devices["Desktop Chrome"],
    baseURL,
    trace: "retain-on-failure",
  },
  webServer: {
    command: "npm run e2e:server",
    url: `${baseURL}/api/health`,
    reuseExistingServer: false,
    timeout: 300_000,
  },
});
