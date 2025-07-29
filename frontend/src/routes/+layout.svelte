<script lang="ts">
  import Banner from "$lib/components/Banner.svelte";
  import DarkModeToggle from "$lib/components/DarkModeToggle.svelte";
  import Footer from "$lib/components/Footer.svelte";
  import PrimaryNav from "$lib/components/PrimaryNav.svelte";
  import { Lock, Unlock } from "@lucide/svelte";
  import { ModeWatcher } from "mode-watcher";
  import "../app.pcss";
</script>

<ModeWatcher />
<div class="min-h-full">
  <DarkModeToggle />
  <section class="text-foregound bg-background pt-16">
    <Banner />
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
      class="flex w-full flex-1 flex-col overflow-hidden bg-background p-4 lg:p-10"
    >
      <slot />
    </main>
  </section>
  <Footer />
</div>
