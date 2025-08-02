import { sveltekit } from "@sveltejs/kit/vite";
import { existsSync, lstatSync, readlinkSync } from "fs";
import { join } from "path";
import { defineConfig } from "vitest/config";

// Check if kosui is linked (symlinked)
// To link kosui, in the kosui repo run (just once):
//   pnpm link
// In this repo, run:
//   pnpm link kosui && pnpm install
// To unlink kosui, run:
//   pnpm unlink kosui && pnpm install
const kosuiPath = join(process.cwd(), "node_modules/kosui");
const isKosuiLinked =
  existsSync(kosuiPath) &&
  lstatSync(kosuiPath).isSymbolicLink() &&
  !readlinkSync(kosuiPath).startsWith(".pnpm/");

export default defineConfig({
  plugins: [sveltekit()],
  server: {
    proxy: {
      "/api": {
        target: "http://localhost:8000",
        changeOrigin: true,
        secure: false,
      },
    },
    ...(isKosuiLinked && {
      watch: {
        ignored: ["!**/node_modules/kosui/**"],
      },
    }),
  },
  ...(isKosuiLinked && {
    optimizeDeps: {
      exclude: ["kosui"],
    },
  }),
  test: {
    include: ["src/**/*.{test,spec}.{js,ts}"],
    environment: "jsdom",
  },
  resolve: process.env.VITEST
    ? {
        conditions: ["browser"],
      }
    : undefined,
});
