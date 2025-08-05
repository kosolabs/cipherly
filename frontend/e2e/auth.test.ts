import { expect, test } from "@playwright/test";

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
    await page.evaluate(([token]) => window.loginn(token), [token]);
    await page.getByRole("button", { name: "Decrypt" }).click();

    // Verify the plain text matches what we encrypted initially.
    // TODO
    // await expect(
    //   page.getByRole("textbox", { name: "Ciphertext Payload Ciphertext" }),
    // ).toHaveText(/.*Some secret.*/);

    await page.getByRole("button", { name: "Copy Ciphertext" }).click();
    expect(await page.evaluate(() => navigator.clipboard.readText())).toEqual(
      "Some secret",
    );
  });
});

export function jwt(email: string) {
  const base64 = (s: string) => Buffer.from(s).toString("base64url");
  const header = {
    alg: "HS256",
    typ: "JWT",
    kid: "koso-integration-test",
  };
  const encodedHeader = base64(JSON.stringify(header));
  const expirationEpochSeconds = Math.floor(
    (Date.now() + 24 * 60 * 60 * 1000) / 1000,
  );
  const payload = {
    email: email,
    name: "Pointy-Haired Boss",
    picture: "https://static.wikia.nocookie.net/dilbert/images/6/60/Boss.PNG",
    exp: expirationEpochSeconds,
    iss: "cipherly-tests",
    aud: "cipherly-tests",
  };
  const encodedSignature = base64("test_signature_cannot_validate");
  const encodedPayload = base64(JSON.stringify(payload));
  return `${encodedHeader}.${encodedPayload}.${encodedSignature}`;
}

export function generateEmail() {
  return `${Math.random().toString(36).slice(2)}-${Date.now()}-test@test.koso.app`;
}
