<script lang="ts">
  import * as Alert from "$lib/components/ui/alert";

  export let title: string;
  export let error: { status?: number; name: string; message?: string };
</script>

<Alert.Root variant="destructive" class="space-y-2 rounded">
  <Alert.Title>{title}</Alert.Title>
  <Alert.Description>
    {#if error?.status === 401}
      Unauthorized
    {:else if error?.name === "OperationError"}
      Incorrect Password
    {:else if typeof error == "object"}
      {error.message}
    {:else}
      {error}
    {/if}
  </Alert.Description>
</Alert.Root>
