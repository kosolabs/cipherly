<script lang="ts">
  import { cn } from "$lib/utils";
  import type { HTMLAttributes } from "svelte/elements";
  import type { User } from "../auth";

  export type Props = { user: User } & HTMLAttributes<HTMLDivElement>;

  let { class: className, user, ...restProps }: Props = $props();
</script>

<div class={cn("overflow-clip rounded-full", className)} {...restProps}>
  <img
    class="object-fit rounded-full"
    src={user.picture}
    alt={user.email}
    referrerpolicy="no-referrer"
  />
</div>
