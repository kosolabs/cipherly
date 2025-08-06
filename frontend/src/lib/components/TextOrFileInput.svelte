<script lang="ts">
  import { encodeUtf8 } from "$lib/cipherly";
  import { CircleX, FileText, HardDriveUpload } from "@lucide/svelte";
  import { filesize } from "filesize";
  import { Button, Textarea } from "kosui";
  import { z } from "zod";

  type Props = {
    text?: string;
    placeholder: string;
    onInput: (data: Uint8Array<ArrayBuffer>, filename: string | null) => void;
  };

  let { text = "", placeholder, onInput }: Props = $props();

  const schema = z
    .object({
      text: z.string(),
      file: z.instanceof(File).nullable(),
    })
    .transform(async ({ text, file }) => {
      if (file !== null) {
        return {
          data: new Uint8Array(await file.arrayBuffer()),
          filename: file?.name,
        };
      }
      return { data: encodeUtf8(text), filename: null };
    });

  let fileInputEl: HTMLInputElement | undefined = $state();
  let files: FileList | undefined = $state();
  let file = $derived(files?.item(0) ?? null);

  $effect(() => {
    schema
      .parseAsync({ text, file })
      .then(({ data, filename }) => onInput(data, filename));
  });
</script>

<input bind:this={fileInputEl} type="file" multiple={false} bind:files hidden />
{#if file}
  <div
    class="bg-accent flex items-center justify-between rounded-md border px-3 py-4"
  >
    <div class="flex items-center space-x-3">
      <FileText class="w-5 text-slate-400" />
      <div class="space-y-[-3px]">
        <div class="text-md">{file.name}</div>
        <div class="text-muted-foreground text-xs">
          {filesize(file.size)}
        </div>
      </div>
    </div>
    <Button
      variant="plain"
      icon={CircleX}
      onclick={() => {
        if (fileInputEl) {
          fileInputEl.value = "";
        }
        onInput(new Uint8Array(), null);
      }}
    />
  </div>
{:else}
  <div class="space-y-2">
    <Textarea
      class="w-full"
      variant="plain"
      placeholder={`Enter the ${placeholder} or drag and drop a ${placeholder} file here`}
      bind:value={text}
      ondrop={(event) => {
        event.preventDefault();
        files = event.dataTransfer?.files;
      }}
    />

    {#if !text}
      <Button
        icon={HardDriveUpload}
        onclick={() => {
          if (fileInputEl) {
            fileInputEl.click();
          }
        }}
      >
        Upload {placeholder} file
      </Button>
    {/if}
  </div>
{/if}
