<script lang="ts">
  import youTubeLogo from "$lib/assets/youtube.svg";

  type Props = {
    id: string;
    title: string;
  };
  let { id, title }: Props = $props();

  const thumbnail = `https://i3.ytimg.com/vi/${id}/maxresdefault.jpg`;

  // Clip length to 9 words
  const videoTitle =
    title.length > 9 ? title.split(" ").slice(0, 9).join(" ") + "..." : title;
</script>

<a
  class="mr-6 mb-6 w-full sm:w-auto sm:max-w-[16rem]"
  target="_blank"
  href={`https://youtu.be/${id}`}
  referrerpolicy="no-referrer"
>
  <div
    class="border-m3-secondary/20 shadow-m3-shadow/20 hover:shadow-m3-shadow/40 overflow-hidden rounded-md border shadow-sm"
  >
    <img class="object-cover" src={thumbnail} alt="" srcset="" />
    <div class="flex items-center space-x-4 px-4">
      <img src={youTubeLogo} alt="YouTube" width="24px" />
      <p class="py-2 text-sm leading-tight font-medium">
        {videoTitle}
      </p>
    </div>
  </div>
</a>
