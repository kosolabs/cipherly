import {
  decode as decodeMessagePack,
  encode as encodeMessagePack,
} from "@msgpack/msgpack";
import { Base64 } from "js-base64";
import { z } from "zod";

function decodeBase64(data: string): Uint8Array<ArrayBuffer> {
  return new Uint8Array(Base64.toUint8Array(data));
}

function encodeBase64(data: Uint8Array): string {
  return Base64.fromUint8Array(data, true);
}

export function encodeUtf8(data: string): Uint8Array<ArrayBuffer> {
  return new Uint8Array(new TextEncoder().encode(data));
}

export function decodeUtf8(data: Uint8Array): string {
  return new TextDecoder().decode(data);
}

function generateSalt(): Uint8Array<ArrayBuffer> {
  return crypto.getRandomValues(new Uint8Array(16));
}

async function generateKey(): Promise<CryptoKey> {
  return crypto.subtle.generateKey(
    {
      name: "AES-GCM",
      length: 256,
    },
    true,
    ["encrypt", "decrypt"],
  );
}

function generateIv(): Uint8Array<ArrayBuffer> {
  return crypto.getRandomValues(new Uint8Array(12));
}

async function deriveKey(
  password: Uint8Array,
  salt: Uint8Array,
): Promise<CryptoKey> {
  const keyMaterial = await crypto.subtle.importKey(
    "raw",
    password,
    { name: "PBKDF2" },
    false,
    ["deriveKey"],
  );

  return crypto.subtle.deriveKey(
    {
      name: "PBKDF2",
      salt,
      iterations: 100000,
      hash: "SHA-256",
    },
    keyMaterial,
    { name: "AES-GCM", length: 256 },
    true,
    ["encrypt", "decrypt"],
  );
}

async function encrypt(
  data: Uint8Array,
  key: CryptoKey,
  iv: Uint8Array,
): Promise<Uint8Array<ArrayBuffer>> {
  return new Uint8Array(
    await crypto.subtle.encrypt({ name: "AES-GCM", iv }, key, data),
  );
}

async function decrypt(
  data: Uint8Array,
  key: CryptoKey,
  iv: Uint8Array,
): Promise<Uint8Array> {
  return new Uint8Array(
    await crypto.subtle.decrypt({ name: "AES-GCM", iv }, key, data),
  );
}

export enum EncryptionScheme {
  Password = 0,
  Auth = 1,
}

const PayloadHeader = z.object({
  es: z.nativeEnum(EncryptionScheme),
  fn: z.string().nullable().optional(),
});
type PayloadHeader = z.infer<typeof PayloadHeader>;

const PasswordBody = z.object({
  s: z.instanceof(Uint8Array),
  iv: z.instanceof(Uint8Array),
  ct: z.instanceof(Uint8Array),
});
type PasswordBody = z.infer<typeof PasswordBody>;

export const PasswordPayload = PayloadHeader.merge(PasswordBody);
export type PasswordPayload = z.infer<typeof PasswordPayload>;

export function isPasswordPayload(
  payload: unknown,
): payload is PasswordPayload {
  return PasswordPayload.safeParse(payload).success;
}

function encodePasswordPayload(
  payload: PasswordBody,
  filename?: string,
): Uint8Array<ArrayBuffer> {
  const msgPackPayload: PasswordPayload = {
    es: EncryptionScheme.Password,
    fn: filename,
    ...payload,
  };
  return new Uint8Array(
    encodeMessagePack(msgPackPayload, { ignoreUndefined: true }),
  );
}

function decodePasswordPayload(data: Uint8Array): PasswordPayload {
  return decodeMessagePack(data) as PasswordPayload;
}

export async function passwordEncrypt(
  plaintext: Uint8Array,
  password: string,
  filename?: string,
): Promise<Uint8Array> {
  const salt = generateSalt();
  const key = await deriveKey(encodeUtf8(password), salt);

  const iv = generateIv();
  const ciphertext = await encrypt(plaintext, key, iv);

  return encodePasswordPayload({ s: salt, iv: iv, ct: ciphertext }, filename);
}

export async function passwordDecrypt(
  payload: Payload,
  password: string,
): Promise<Uint8Array> {
  const { s: salt, iv: iv, ct: ciphertext } = payload as PasswordPayload;
  const key = await deriveKey(encodeUtf8(password), salt);
  return await decrypt(ciphertext, key, iv);
}

const AuthBody = z.object({
  k: z.string(),
  n: z.instanceof(Uint8Array),
  se: z.instanceof(Uint8Array),
  iv: z.instanceof(Uint8Array),
  ct: z.instanceof(Uint8Array),
});
type AuthBody = z.infer<typeof AuthBody>;

export const AuthPayload = PayloadHeader.merge(AuthBody);
export type AuthPayload = z.infer<typeof AuthPayload>;

export function isAuthPayload(payload: unknown): payload is AuthPayload {
  const parsed = AuthPayload.safeParse(payload);
  console.log(parsed.error);
  return AuthPayload.safeParse(payload).success;
}

function encodeAuthPayload(payload: AuthBody, filename?: string): Uint8Array {
  const msgPackPayload: AuthPayload = {
    es: EncryptionScheme.Auth,
    fn: filename,
    ...payload,
  };
  return encodeMessagePack(msgPackPayload);
}

function decodeAuthPayload(data: Uint8Array): AuthPayload {
  return decodeMessagePack(data) as AuthPayload;
}

export const Payload = z.union([PasswordPayload, AuthPayload]);
export type Payload = z.infer<typeof Payload>;

function decryptUrl() {
  return `${location.protocol}//${location.host}/decrypt/#`;
}

export function encodePayload(data: Uint8Array, file: boolean): Uint8Array[] {
  const url = encodeUtf8(decryptUrl());
  if (file) {
    return [url, data];
  } else {
    return [url, encodeUtf8(encodeBase64(data))];
  }
}

export function decodePayload(data: Uint8Array, file: boolean): Payload {
  const endOfUrl = data.indexOf(0x23) + 1;
  if (endOfUrl === 0) {
    throw new Error("Payload is missing URL header");
  }
  const hostPath = decodeUtf8(data.subarray(0, endOfUrl));
  if (hostPath !== decryptUrl()) {
    throw new Error("Payload is not intended for this Cipherly instance");
  }
  let payloadData = data.subarray(endOfUrl);
  if (!file) {
    payloadData = decodeBase64(decodeUtf8(payloadData));
  }
  return Payload.parse(decodeMessagePack(payloadData));
}

type Envelope = {
  dek: CryptoKey;
  emails: string[];
};

type SealedEnvelope = {
  kid: string;
  nonce: Uint8Array<ArrayBuffer>;
  data: Uint8Array<ArrayBuffer>;
};

async function seal(envelope: Envelope): Promise<SealedEnvelope> {
  const encodedDek = await crypto.subtle.exportKey("raw", envelope.dek);
  const response = await fetch("/api/seal", {
    method: "POST",
    headers: {
      "Content-Type": "application/json",
    },
    body: JSON.stringify({
      dek: encodeBase64(new Uint8Array(encodedDek)),
      emails: envelope.emails,
    }),
  });
  if (!response.ok) {
    throw { code: response.status, message: response.statusText };
  }
  const result = await response.json();
  return {
    kid: result.kid,
    nonce: decodeBase64(result.nonce),
    data: decodeBase64(result.data),
  };
}

async function unseal(
  envelope: SealedEnvelope,
  token: string,
): Promise<Envelope> {
  const response = await fetch("/api/unseal", {
    method: "POST",
    headers: {
      "Content-Type": "application/json",
      Authorization: "Bearer " + token,
    },
    body: JSON.stringify({
      kid: envelope.kid,
      nonce: encodeBase64(envelope.nonce),
      data: encodeBase64(envelope.data),
    }),
  });
  if (!response.ok) {
    throw response;
  }
  const result = await response.json();
  const dek = await crypto.subtle.importKey(
    "raw",
    decodeBase64(result.dek),
    { name: "AES-GCM" },
    true,
    ["encrypt", "decrypt"],
  );

  return { dek, emails: result.emails };
}

export async function authEncrypt(
  plaintext: Uint8Array,
  emails: string[],
  filename?: string,
): Promise<Uint8Array> {
  const dek = await generateKey();
  const iv = generateIv();
  const ciphertext = await encrypt(plaintext, dek, iv);
  const { kid, nonce, data } = await seal({ dek, emails });
  return encodeAuthPayload(
    {
      k: kid,
      n: nonce,
      se: data,
      iv: iv,
      ct: ciphertext,
    },
    filename,
  );
}

export async function authDecrypt(
  payload: Payload,
  token: string,
): Promise<Uint8Array> {
  const {
    k: kid,
    n: nonce,
    se: data,
    iv: iv,
    ct: ciphertext,
  } = payload as AuthPayload;
  const envelope = await unseal({ kid, nonce, data }, token);
  const plaintext = await decrypt(ciphertext, envelope.dek, iv);
  return plaintext;
}

export const exportedForTesting = {
  decodeAuthPayload,
  decodePasswordPayload,
  decrypt,
  deriveKey,
  encodeAuthPayload,
  encodePasswordPayload,
  encrypt,
  generateIv,
  generateKey,
  generateSalt,
};
