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
  import Input from "$lib/components/Input.svelte";
  import Label from "$lib/components/Label.svelte";
  import TextOrFileInput from "$lib/components/TextOrFileInput.svelte";
  import TextOrFileOutput from "$lib/components/TextOrFileOutput.svelte";
  import ValidationError from "$lib/components/ValidationError.svelte";
  import { Button } from "$lib/components/ui/button";
  import { z } from "zod";
  import Auth from "./auth.svelte";

  const InputData = z
    .object({
      data: z.instanceof(Uint8Array),
      filename: z.string().nullable(),
    })
    .transform(({ data, filename }, ctx) => {
      if (data.length !== 0 || filename !== null) {
        try {
          return decodePayload(data, !!filename);
        } catch (_error) {
          ctx.addIssue({
            code: z.ZodIssueCode.custom,
            message: "Invalid Cipherly payload",
            path: ["payload"],
            fatal: true,
          });
        }
      }
      return z.NEVER;
    });
  type InputData = z.input<typeof InputData>;

  let inputData: InputData = {
    data: new Uint8Array(),
    filename: null,
  };

  const DecryptData = z
    .object({
      payload: Payload.nullable(),
      password: z.string().default(""),
      token: z.string().nullable(),
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

  let decryptData: DecryptData = {
    payload: null,
    password: "",
    token: null,
  };

  let error: z.ZodError | null;
  let plaintext: Promise<Uint8Array[]> | null = null;

  $: {
    // eslint-disable-next-line @typescript-eslint/no-unused-expressions
    inputData;
    plaintext = null;
    InputData.safeParseAsync(inputData).then((p) => {
      if (p.success) {
        decryptData.payload = p.data;
        error = null;
      } else {
        decryptData.payload = null;
        error = p.error;
      }
    });
  }

  function decrypt() {
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
  <form class="space-y-4" on:submit|preventDefault={decrypt}>
    <div>
      <Label for="payload">Ciphertext Payload</Label>
      <ValidationError {error} path="payload" />
      <TextOrFileInput
        text={location.hash ? location.href : ""}
        bind:data={inputData.data}
        bind:filename={inputData.filename}
        placeholder="ciphertext payload"
      />
    </div>

    {#if decryptData.payload?.es === EncryptionScheme.Password}
      <div>
        <Label for="password">Password</Label>
        <ValidationError {error} path="password" />
        <Input
          id="password"
          type="password"
          placeholder="The password to use for decryption"
          bind:value={decryptData.password}
        />
      </div>
    {:else if decryptData.payload?.es === EncryptionScheme.Auth}
      <div>
        <ValidationError {error} path="token" />
        <Auth bind:token={decryptData.token} />
      </div>
    {/if}

    <Button class="min-w-[140px] text-lg font-bold" type="submit">
      Decrypt
    </Button>
  </form>

  {#if plaintext}
    <TextOrFileOutput
      kind="Decrypt"
      data={plaintext}
      name={decryptData.payload?.fn}
    />
  {/if}
</div>
