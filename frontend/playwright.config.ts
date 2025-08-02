import type { PlaywrightTestConfig } from "@playwright/test";

const port = 5173;

const config: PlaywrightTestConfig = {
  webServer: {
    command: `pnpm build && pnpm preview --port ${port}`,
    port,
    reuseExistingServer: true,
    stdout: "pipe",
  },
  testDir: "e2e",
  fullyParallel: true,
};

export default config;
