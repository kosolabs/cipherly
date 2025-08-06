import { expect, test } from "@playwright/test";
import { generateEmail, jwt } from "./utils";

declare global {
  interface Window {
    setLoginToken: (token: string) => void;
  }
}

// test.describe.configure({ mode: "parallel" });

test.describe("Encrypt and Decrypt Tests", () => {
  test.beforeEach(async ({ page }) => {
    await page.goto("/");
  });

  test("password encryption of text payload", async ({ context, page }) => {
    const expected = "secret text";
    await context.grantPermissions(["clipboard-read", "clipboard-write"]);

    // Encrypt the secret.
    await page.getByRole("link", { name: "Encrypt", exact: true }).click();
    await page
      .getByRole("textbox", { name: "Enter the plaintext secret" })
      .fill(expected);
    await page.getByRole("option", { name: "Password" }).click();
    await page.getByRole("textbox", { name: "Password" }).fill("some-password");
    await page.getByRole("button", { name: "Encrypt" }).click();

    // Copy the ciphertext.
    await page.getByRole("button", { name: "Copy Ciphertext" }).click();

    // Decrypt the secret.
    await page.getByRole("link", { name: "Decrypt" }).click();
    await page
      .getByRole("textbox", { name: "Enter the ciphertext payload" })
      .click();
    await page.keyboard.press("ControlOrMeta+V");
    await page.getByRole("textbox", { name: "Password" }).fill("some-password");
    await page.getByRole("button", { name: "Decrypt" }).click();

    // Verify the plain text matches what we encrypted initially.
    await page.getByRole("button", { name: "Copy Ciphertext" }).click();
    const actual = await page.evaluate(() => navigator.clipboard.readText());
    expect(actual).toEqual(expected);
  });

  test("password encryption of file payload", async ({ page }) => {
    // Upload the plain text file.
    await page.getByRole("link", { name: "Encrypt", exact: true }).click();
    const plainTextTileChooserPromise = page.waitForEvent("filechooser");
    await page.getByRole("button", { name: "Upload" }).click();
    const plainTextFileChooser = await plainTextTileChooserPromise;
    await plainTextFileChooser.setFiles("e2e/plain.txt");

    // Set the password.
    await page.getByRole("option", { name: "Password" }).click();
    await page.getByRole("textbox", { name: "Password" }).fill("some-password");

    // Download the encrypted cipher text.
    const cipherTextDownloadPromise = page.waitForEvent("download");
    await page.getByRole("button", { name: "Encrypt" }).click();
    const cipherTextDownload = await cipherTextDownloadPromise;
    const cipherTextPath = `/tmp/${cipherTextDownload.suggestedFilename()}`;
    await cipherTextDownload.saveAs(cipherTextPath);

    // Upload the encrypted cipher text.
    await page.getByRole("link", { name: "Decrypt" }).click();
    const cipherTextTileChooserPromise = page.waitForEvent("filechooser");
    await page.getByRole("button", { name: "Upload" }).click();
    const cipherTextFileChooser = await cipherTextTileChooserPromise;
    await cipherTextFileChooser.setFiles(cipherTextPath);

    // Set the password.
    await page.getByRole("textbox", { name: "Password" }).fill("some-password");

    // Download the decrypted plain text.
    const plainTextDownloadPromise = page.waitForEvent("download");
    await page.getByRole("button", { name: "Decrypt" }).click();
    const plainTextDownload = await plainTextDownloadPromise;

    // Read the contents of the decrypted plain text.
    const stream = await plainTextDownload.createReadStream();
    const chunks: string[] = [];
    const decoder = new TextDecoder("utf-8");
    for await (const chunk of stream) {
      chunks.push(decoder.decode(chunk));
    }

    // Validate that it is equal to the expected value.
    expect(chunks.join("")).toEqual("secret file");
  });

  test("policy encryption of text payload", async ({ context, page }) => {
    await context.grantPermissions(["clipboard-read", "clipboard-write"]);

    const email = generateEmail();
    const token = jwt(email);

    // Encrypt the plain text.
    await page.getByRole("link", { name: "Encrypt", exact: true }).click();
    await page
      .getByRole("textbox", { name: "Enter the plaintext secret" })
      .fill("secret text");
    await page.getByRole("textbox", { name: "Authorized Emails" }).fill(email);
    await page.keyboard.press("Enter");
    await page.getByRole("button", { name: "Encrypt" }).click();

    // Copy the cipher text.
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
      "secret text",
    );
  });

  test("policy encryption of file payload", async ({ page }) => {
    const email = generateEmail();
    const token = jwt(email);

    // Upload the plain text file.
    await page.getByRole("link", { name: "Encrypt", exact: true }).click();
    const plainTextTileChooserPromise = page.waitForEvent("filechooser");
    await page.getByRole("button", { name: "Upload" }).click();
    const plainTextFileChooser = await plainTextTileChooserPromise;
    await plainTextFileChooser.setFiles("e2e/plain.txt");

    // Set the authorized user.
    await page.getByRole("textbox", { name: "Authorized Emails" }).fill(email);
    await page.keyboard.press("Enter");

    // Download the encrypted cipher text.
    const cipherTextDownloadPromise = page.waitForEvent("download");
    await page.getByRole("button", { name: "Encrypt" }).click();
    const cipherTextDownload = await cipherTextDownloadPromise;
    const cipherTextPath = `/tmp/${cipherTextDownload.suggestedFilename()}`;
    await cipherTextDownload.saveAs(cipherTextPath);

    // Upload the encrypted cipher text.
    await page.getByRole("link", { name: "Decrypt" }).click();
    const cipherTextTileChooserPromise = page.waitForEvent("filechooser");
    await page.getByRole("button", { name: "Upload" }).click();
    const cipherTextFileChooser = await cipherTextTileChooserPromise;
    await cipherTextFileChooser.setFiles(cipherTextPath);

    // Login as the authorized user
    await page.evaluate(([token]) => window.setLoginToken(token), [token]);

    // Download the decrypted plain text.
    const plainTextDownloadPromise = page.waitForEvent("download");
    await page.getByRole("button", { name: "Decrypt" }).click();
    const plainTextDownload = await plainTextDownloadPromise;

    // Read the contents of the decrypted plain text.
    const stream = await plainTextDownload.createReadStream();
    const chunks: string[] = [];
    const decoder = new TextDecoder("utf-8");
    for await (const chunk of stream) {
      chunks.push(decoder.decode(chunk));
    }

    // Validate that it is equal to the expected value.
    expect(chunks.join("")).toEqual("secret file");
  });
});
