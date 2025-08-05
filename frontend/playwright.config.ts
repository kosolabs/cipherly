import type { PlaywrightTestConfig } from "@playwright/test";

const port = process.env.CI ? 8000 : process.env.PW_SERVER_PORT || 5173;

const config: PlaywrightTestConfig = {
  webServer: {
    command: process.env.CIPHERLY_IMAGE
      ? `docker run \
        --env KEKS='{"v1":"jRg36ErQ6FLcc7nZgngOpjJnJLGwA3xaMy0yx1pxJrI"}' \
        --env ENABLE_TEST_CREDS=true \
        --env RUST_LOG=info \
        --network=host \
        --rm ${process.env.CIPHERLY_IMAGE}`
      : `pnpm build && pnpm preview --port ${port}`,
    port,
    reuseExistingServer: !process.env.CI,
    stdout: "pipe",
    timeout: 180000,
  },
  testDir: "e2e",
  fullyParallel: true,
};

export default config;
