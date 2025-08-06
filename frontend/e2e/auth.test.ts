import { expect, test } from "@playwright/test";
import { generateEmail, jwt } from "./utils";

declare global {
  interface Window {
    setLoginToken: (token: string) => void;
  }
}

test.describe.configure({ mode: "parallel" });

test.describe("Auth Tests", () => {
  test.beforeEach(async ({ page }) => {
    await page.goto("/");
  });

  test("encrypt and decrypt with policy succeeds", async ({
    context,
    page,
  }) => {
    await context.grantPermissions(["clipboard-read", "clipboard-write"]);

    const email = generateEmail();
    const token = jwt(email);

    // Encrypt the secret.
    await page.getByRole("link", { name: "Encrypt", exact: true }).click();
    await page
      .getByRole("textbox", { name: "Enter the plaintext secret" })
      .fill("Some secret");
    await page.getByRole("textbox", { name: "Authorized Emails" }).fill(email);
    await page.keyboard.press("Enter");
    await page.getByRole("button", { name: "Encrypt" }).click();

    // Copy the ciphertext.
    await page.getByRole("button", { name: "Copy Ciphertext" }).click();

    // Decrypt the secret after "logging in".
    await page.getByRole("link", { name: "Decrypt" }).click();
    await page
      .getByRole("textbox", { name: "Enter the ciphertext payload" })
      .click();
    await page.keyboard.press("ControlOrMeta+V");
    await page.evaluate(([token]) => window.setLoginToken(token), [token]);
    await page.getByRole("button", { name: "Decrypt" }).click();

    // Verify the plain text matches what we encrypted initially.
    await page.getByRole("button", { name: "Copy Ciphertext" }).click();
    expect(await page.evaluate(() => navigator.clipboard.readText())).toEqual(
      "Some secret",
    );
  });
});
