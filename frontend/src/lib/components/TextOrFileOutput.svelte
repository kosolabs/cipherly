<script lang="ts">
  import { decodeUtf8 } from "$lib/cipherly";
  import { Skeleton, Textarea } from "kosui";
  import { onMount } from "svelte";
  import CopyText from "./CopyText.svelte";
  import EncryptionAlert from "./EncryptionAlert.svelte";
  import Label from "./Label.svelte";

  type Props = {
    kind: string;
    data: Promise<Uint8Array<ArrayBuffer>[]>;
    name?: string;
  };

  let { kind, data, name }: Props = $props();

  onMount(async () => {
    if (name !== undefined) {
      const blob = new Blob(await data, { type: "application/octet-stream" });
      const url = URL.createObjectURL(blob);
      const a = document.createElement("a");
      a.href = url;
      a.download = name;
      a.click();
    }
  });
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
      <Textarea
        id="payload"
        class="w-full"
        variant="plain"
        value={text}
        placeholder="The plain text secret to encrypt"
      />
      <div class="space-x-2 pt-4">
        <CopyText label="Ciphertext" {text} />
      </div>
    {/if}
  {:catch error}
    <EncryptionAlert title={`Failed to ${kind}`} {error} />
  {/await}
</div>
