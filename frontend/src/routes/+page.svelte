<script lang="ts">
  import YouTubeVideoCard from "$lib/components/YouTubeVideoCard.svelte";
  import { Link } from "kosui";
  const youTubeVideos = [
    {
      id: "fP57rGxokow",
      title: "Encrypting sensitive files with Cipherly",
    },
    {
      id: "AsxSR4mtCuM",
      title: "Encrypting a secret decryptable only by the authorized user",
    },
    {
      id: "oKCNhjF9k8c",
      title: "Encrypting a secret without sending it over the internet",
    },
  ];
</script>

<section class="space-y-10 py-4 text-lg md:py-10">
  <div class="space-y-4">
    <h1 class="pb-1 text-3xl font-bold">What is Cipherly?</h1>
    <p>Cipherly is a browser-based encryption tool for sharing secrets</p>
    <p>
      It uses the browser's built-in
      <Link
        target="_blank"
        href="https://developer.mozilla.org/en-US/docs/Web/API/SubtleCrypto"
      >
        SubtleCrypto
      </Link>
      library to encrypt and decrypt the secret without ever sending the secret to
      the server.
    </p>
  </div>

  <div class="space-y-4">
    <h1 class="pb-1 text-3xl font-bold">How can I learn more?</h1>
    <p>
      You can follow along on the
      <Link target="_blank" href="https://www.youtube.com/@FriendlyTL">
        Friendly TL YouTube channel!
      </Link>
    </p>
    <div class="flex flex-wrap">
      {#each youTubeVideos as video (video)}
        <YouTubeVideoCard {...video} />
      {/each}
    </div>
  </div>
</section>
