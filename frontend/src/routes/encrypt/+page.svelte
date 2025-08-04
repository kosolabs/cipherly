<script lang="ts">
  import { authEncrypt, encodePayload, passwordEncrypt } from "$lib/cipherly";
  import Chip from "$lib/components/Chip.svelte";
  import IconText from "$lib/components/IconText.svelte";
  import Label from "$lib/components/Label.svelte";
  import TextOrFileInput from "$lib/components/TextOrFileInput.svelte";
  import TextOrFileOutput from "$lib/components/TextOrFileOutput.svelte";
  import ValidationError from "$lib/components/ValidationError.svelte";
  import { KeyRound, User } from "@lucide/svelte";
  import { Box, Button, Input, ToggleButton, ToggleGroup } from "kosui";
  import { z } from "zod";

  const EncryptData = z
    .object({
      data: z.instanceof(Uint8Array),
      filename: z.string().nullable(),
      mode: z.enum(["policy", "password"]),
      password: z.string().default(""),
      emails: z.array(z.string().email()),
    })
    .check(({ issues, value }) => {
      if (value.mode === "policy" && value.emails.length === 0) {
        issues.push({
          code: "too_small",
          minimum: 1,
          inclusive: true,
          origin: "array",
          message: "At least one email address must be present",
          path: ["emails"],
          input: value,
        });
      } else if (value.mode === "password" && value.password.length === 0) {
        issues.push({
          code: "too_small",
          minimum: 1,
          inclusive: true,
          origin: "string",
          message: "Password must be present",
          path: ["password"],
          input: value,
        });
      }
    })
    .refine(({ data, filename }) => data.length !== 0 || filename !== null, {
      message: "Either text or file input must be present",
      path: ["plaintext"],
    });
  type EncryptData = z.input<typeof EncryptData>;

  let encrypt: EncryptData = {
    data: new Uint8Array(),
    filename: null,
    mode: "policy",
    password: "",
    emails: [],
  };

  let error: z.ZodError | null;
  let payload: Promise<Uint8Array> | null = null;

  $: {
    // eslint-disable-next-line @typescript-eslint/no-unused-expressions
    encrypt;
    payload = null;
    error = null;
  }
</script>

<div class="space-y-8 p-1">
  <form
    class="space-y-4"
    on:submit|preventDefault={() => {
      const parsed = EncryptData.safeParse(encrypt);
      if (!parsed.success) {
        error = parsed.error;
        payload = null;
        return;
      }
      error = null;
      if (parsed.data.mode === "password") {
        payload = passwordEncrypt(
          parsed.data.data,
          parsed.data.password,
          parsed.data.filename ? parsed.data.filename : undefined,
        );
      } else if (parsed.data.mode === "policy") {
        payload = authEncrypt(
          parsed.data.data,
          parsed.data.emails,
          parsed.data.filename ? parsed.data.filename : undefined,
        );
      } else {
        throw new Error("Invalid encryption mode");
      }
    }}
  >
    <div>
      <Label for="plaintext">Plaintext</Label>
      <ValidationError {error} path="plaintext" />
      <TextOrFileInput
        bind:data={encrypt.data}
        bind:filename={encrypt.filename}
        placeholder="plaintext secret"
      />
    </div>

    <div>
      <Label for="mode">Encryption Type</Label>
      <ToggleGroup bind:value={encrypt.mode}>
        <ToggleButton value="policy">
          <IconText icon={User}>Policy</IconText>
        </ToggleButton>
        <ToggleButton value="password">
          <IconText icon={KeyRound}>Password</IconText>
        </ToggleButton>
      </ToggleGroup>
      <Box
        variant="elevated"
        shape="rounded"
        class="mt-2 border px-4 pt-1 pb-2"
      >
        {#if encrypt.mode === "policy"}
          <Label for="emails">Authorized Emails</Label>
          <ValidationError {error} path="emails" />
          <Chip
            id="emails"
            bind:values={encrypt.emails}
            placeholder="List of email addresses authorized to decrypt"
          />
        {:else if encrypt.mode === "password"}
          <Label for="password">Password</Label>
          <ValidationError {error} path="password" />
          <Input
            id="password"
            type="password"
            class="w-full"
            variant="plain"
            placeholder="The password to use for encryption"
            bind:value={encrypt.password}
          />
        {/if}
      </Box>
    </div>

    <Button
      variant="filled"
      class="min-w-[140px] text-lg font-bold"
      type="submit"
    >
      Encrypt
    </Button>
  </form>

  {#if payload}
    <TextOrFileOutput
      kind="Encrypt"
      data={payload.then((data) => encodePayload(data, !!encrypt.filename))}
      name={encrypt.filename ? encrypt.filename + ".cly" : null}
    />
  {/if}
</div>
