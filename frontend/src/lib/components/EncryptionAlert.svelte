<script lang="ts">
  import { Alert } from "kosui";

  type Props = {
    title: string;
    error: { status?: number; name: string; message?: string };
  };

  let { title, error }: Props = $props();
</script>

<Alert variant="outlined" color="error">
  <div>{title}</div>
  <div class="text-sm">
    {#if error?.status === 401}
      Unauthorized
    {:else if error?.name === "OperationError"}
      Incorrect Password
    {:else if typeof error == "object"}
      {error.message}
    {:else}
      {error}
    {/if}
  </div>
</Alert>
