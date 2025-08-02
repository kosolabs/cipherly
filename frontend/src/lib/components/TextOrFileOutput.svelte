<script lang="ts">
  import { decodeUtf8 } from "$lib/cipherly";
  import CopyText from "./CopyText.svelte";
  import EncryptionAlert from "./EncryptionAlert.svelte";
  import { Label } from "./ui/label";
  import { Skeleton } from "./ui/skeleton";

  export let kind: string;
  export let data: Promise<Uint8Array[]>;
  export let name: string | null = null;

  function save(data: Uint8Array[], name: string) {
    const blob = new Blob(data, { type: "application/octet-stream" });
    const url = URL.createObjectURL(blob);
    const a = document.createElement("a");
    a.href = url;
    a.download = name;
    a.click();
  }

  $: if (name !== null) {
    data.then((data) => save(data, name));
  }
</script>

<div>
  {#await data}
    <div class="space-y-6 py-6">
      <Skeleton class="h-20 w-full" />
      <Skeleton class="h-10 w-full" />
    </div>
  {:then data}
    {#if !name}
      {@const text = data.map(decodeUtf8).join("")}
      <Label
        for="payload"
        class="text-background-foreground text-sm tracking-wider uppercase"
      >
        Ciphertext Payload
      </Label>
      <textarea
        id="payload"
        class="border-m3-secondary/40 text-m3-primary w-full rounded-lg shadow"
        disabled
        value={text}
        placeholder="The plain text secret to encrypt"
      ></textarea>
      <div class="space-x-2 pt-4">
        <CopyText label="Ciphertext" {text} />
      </div>
    {/if}
  {:catch error}
    <EncryptionAlert title={`Failed to ${kind}`} {error} />
  {/await}
</div>
