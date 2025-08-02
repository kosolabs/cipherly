import { expect, test } from "@playwright/test";

test.describe.configure({ mode: "parallel" });

test.describe("Home Tests", () => {
  test.beforeEach(async ({ page }) => {
    await page.goto("/");
  });

  test("home should render", async ({ page }) => {
    await expect(
      page.getByRole("heading", { name: "What is Cipherly?" }),
    ).toBeVisible();
    await expect(
      page.getByRole("heading", { name: "How can I learn more?" }),
    ).toBeVisible();
  });

  test("footer should render", async ({ page }) => {
    await expect(page.getByText("Made with ♥")).toBeVisible();
  });
});
