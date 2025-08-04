<script lang="ts">
  import AppBar from "$lib/components/AppBar.svelte";
  import Footer from "$lib/components/Footer.svelte";
  import PrimaryNav from "$lib/components/PrimaryNav.svelte";
  import { Lock, Unlock } from "@lucide/svelte";
  import { ModeWatcher } from "mode-watcher";
  import type { Snippet } from "svelte";
  import "../app.css";

  type Props = {
    children: Snippet;
  };
  let { children }: Props = $props();
</script>

<ModeWatcher />
<div class="min-h-full">
  <AppBar />
  <section class="text-m3-foregound bg-m3-background pt-16">
    <PrimaryNav
      items={[
        {
          title: "Encrypt",
          href: "/encrypt/",
          icon: Lock,
        },
        {
          title: "Decrypt",
          href: "/decrypt/",
          icon: Unlock,
        },
      ]}
    />
    <main
      class="bg-m3-background flex w-full flex-1 flex-col overflow-hidden p-4 lg:p-10"
    >
      {@render children()}
    </main>
  </section>
  <Footer />
</div>
