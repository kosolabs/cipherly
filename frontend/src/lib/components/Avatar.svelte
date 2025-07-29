<script lang="ts">
  import { cn } from "$lib/utils";
  import type { HTMLAttributes } from "svelte/elements";
  import type { User } from "../auth";

  type $$Props = HTMLAttributes<HTMLHeadingElement> & {
    user: User;
  };

  let className: $$Props["class"] = undefined;
  export { className as class };
  export let user: User;
</script>

<div class={cn("overflow-clip rounded-full", className)}>
  <img
    class="object-fit rounded-full"
    src={user.picture}
    alt={user.email}
    referrerpolicy="no-referrer"
  />
</div>
