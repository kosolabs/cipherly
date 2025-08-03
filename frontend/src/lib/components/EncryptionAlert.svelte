<script lang="ts">
  import { Alert } from "kosui";

  export let title: string;
  export let error: { status?: number; name: string; message?: string };
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
