<script lang="ts">
  import {
    EncryptionScheme,
    Payload,
    authDecrypt,
    decodePayload,
    isAuthPayload,
    isPasswordPayload,
    passwordDecrypt,
  } from "$lib/cipherly";
  import Label from "$lib/components/Label.svelte";
  import TextOrFileInput from "$lib/components/TextOrFileInput.svelte";
  import TextOrFileOutput from "$lib/components/TextOrFileOutput.svelte";
  import ValidationError from "$lib/components/ValidationError.svelte";
  import { Button, Input } from "kosui";
  import { z } from "zod";
  import Auth from "./auth.svelte";

  const InputData = z
    .object({
      data: z.instanceof(Uint8Array),
      filename: z.string().optional(),
    })
    .transform(({ data, filename }, ctx) => {
      if (data.length !== 0 || filename) {
        try {
          return decodePayload(data, !!filename);
        } catch (_error) {
          ctx.addIssue({
            code: "custom",
            message: "Invalid Cipherly payload",
            path: ["payload"],
            fatal: true,
          });
        }
      }
      return z.NEVER;
    });
  type InputData = z.input<typeof InputData>;

  const DecryptData = z
    .object({
      payload: Payload.nullable(),
      password: z.string().default(""),
      token: z.string().optional(),
    })
    .transform(({ payload, password, token }, ctx) => {
      if (isPasswordPayload(payload)) {
        if (!password) {
          ctx.addIssue({
            code: "custom",
            message: "Password must be present",
            path: ["password"],
            fatal: true,
          });
          return z.NEVER;
        }
        return { payload, password };
      }
      if (isAuthPayload(payload)) {
        if (!token) {
          ctx.addIssue({
            code: "custom",
            message: "User must be authorized",
            path: ["token"],
            fatal: true,
          });
          return z.NEVER;
        }
        return { payload, token };
      }
      ctx.addIssue({
        code: "custom",
        message: "Payload must be present",
        path: ["payload"],
        fatal: true,
      });
      return z.NEVER;
    });
  type DecryptData = z.input<typeof DecryptData>;

  let decryptData: DecryptData = $state({
    payload: null,
    password: "",
  });

  let error: z.ZodError | null = $state(null);
  let plaintext: Promise<Uint8Array<ArrayBuffer>[]> | null = $state(null);

  function decrypt(e: SubmitEvent) {
    e.preventDefault();
    plaintext = null;
    if (error) return;
    const parsed = DecryptData.safeParse(decryptData);
    if (!parsed.success) {
      error = parsed.error;
      plaintext = null;
      return;
    }
    error = null;
    if (isAuthPayload(parsed.data.payload)) {
      plaintext = Promise.all([
        authDecrypt(parsed.data.payload, parsed.data.token!),
      ]);
    } else if (isPasswordPayload(parsed.data.payload)) {
      plaintext = Promise.all([
        passwordDecrypt(parsed.data.payload, parsed.data.password!),
      ]);
    }
  }
</script>

<div class="space-y-8 p-1">
  <form class="space-y-4" onsubmit={decrypt}>
    <div>
      <Label for="payload">Ciphertext Payload</Label>
      <ValidationError {error} path="payload" />
      <TextOrFileInput
        text={location.hash ? location.href : ""}
        placeholder="ciphertext payload"
        onInput={async (data, filename?) => {
          const result = await InputData.safeParseAsync({ data, filename });
          if (result.success) {
            decryptData.payload = result.data;
            error = null;
          } else {
            decryptData.payload = null;
            error = result.error;
          }
        }}
      />
    </div>

    {#if decryptData.payload?.es === EncryptionScheme.Password}
      <div>
        <Label for="password">Password</Label>
        <ValidationError {error} path="password" />
        <Input
          id="password"
          type="password"
          autocomplete="off"
          placeholder="The password to use for decryption"
          class="w-full"
          bind:value={decryptData.password}
        />
      </div>
    {:else if decryptData.payload?.es === EncryptionScheme.Auth}
      <div>
        <ValidationError {error} path="token" />
        <Auth onToken={(token) => (decryptData.token = token)} />
      </div>
    {/if}

    <Button
      variant="filled"
      class="min-w-[140px] text-lg font-bold"
      type="submit"
    >
      Decrypt
    </Button>
  </form>

  {#if plaintext}
    <TextOrFileOutput
      kind="Decrypt"
      data={plaintext}
      name={decryptData.payload?.fn ?? undefined}
    />
  {/if}
</div>
