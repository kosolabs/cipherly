<script lang="ts">
  import type { Snippet } from "svelte";
  import type { HTMLLabelAttributes } from "svelte/elements";

  type Props = {
    children: Snippet;
  } & HTMLLabelAttributes;
  let { children, ...restProps }: Props = $props();
</script>

<label
  class="text-m3-secondary text-sm tracking-wider uppercase"
  {...restProps}
>
  {@render children()}
</label>
