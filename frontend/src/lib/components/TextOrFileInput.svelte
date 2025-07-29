<script lang="ts">
  import { encodeUtf8 } from "$lib/cipherly";
  import { FileText, HardDriveUpload, XCircle } from "@lucide/svelte";
  import { filesize } from "filesize";
  import { z } from "zod";
  import Button from "./ui/button/button.svelte";

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

  let files: FileList | null = null;
  export let text: string = "";
  export let placeholder: string;
  export let data: Uint8Array = new Uint8Array();
  export let filename: string | null;

  async function parse(text: string, file: File | null) {
    const output = await schema.parseAsync({ text, file });
    data = output.data;
    filename = output.filename;
  }

  function handleUpload(_event: unknown) {
    document.getElementById("fileInput")?.click();
  }

  function drop(event: DragEvent) {
    event.preventDefault();
    files = event.dataTransfer?.files ?? null;
  }

  $: file = files?.item(0) ?? null;
  $: parse(text, file);
</script>

<input id="fileInput" type="file" bind:files hidden />
{#if file}
  <div
    class="flex items-center justify-between rounded-md border bg-accent px-3 py-4"
  >
    <div class="flex items-center space-x-3">
      <FileText class="w-5 text-slate-400" />
      <div class="space-y-[-3px]">
        <div class="text-md">{file.name}</div>
        <div class="text-xs text-muted-foreground">
          {filesize(file.size)}
        </div>
      </div>
    </div>
    <button
      on:click={() => {
        file = null;
      }}
    >
      <XCircle class="w-5 cursor-pointer text-red-800" />
    </button>
  </div>
{:else}
  <div class="space-y-2">
    <textarea
      class="flex min-h-[80px] w-full rounded-md border-2 border-input bg-background px-3 py-2 text-base text-foreground ring-offset-background placeholder:text-muted-foreground focus:ring-0 focus-visible:outline-none focus-visible:ring-0 focus-visible:ring-ring focus-visible:ring-offset-2 disabled:cursor-not-allowed disabled:opacity-50"
      placeholder={`Enter the ${placeholder} or drag and drop a ${placeholder} file here`}
      bind:value={text}
      on:drop={(e) => drop(e)}
    ></textarea>

    {#if !text}
      <Button variant="outline" on:click={(e) => handleUpload(e)}>
        <div class="flex items-center space-x-3">
          <HardDriveUpload class="w-5" />
          <span>Upload {placeholder} file</span>
        </div>
      </Button>
    {/if}
  </div>
{/if}
