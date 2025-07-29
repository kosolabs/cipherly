<script lang="ts">
  import youTubeLogo from "$lib/assets/youtube.svg";

  interface YouTubeVideo {
    id: string;
    title: string;
  }
  export let video: YouTubeVideo;
  const videoUrl = `https://youtu.be/${video.id}`;
  const thumbnail = `https://i3.ytimg.com/vi/${video.id}/maxresdefault.jpg`;

  // Clip length to 9 words
  const videoTitle =
    video.title.length > 9
      ? video.title.split(" ").slice(0, 9).join(" ") + "..."
      : video.title;
</script>

<a
  class="mb-6 mr-6 w-full sm:w-auto sm:max-w-[16rem]"
  target="_blank"
  href={videoUrl}
  referrerpolicy="no-referrer"
>
  <div
    class="overflow-hidden rounded border border-border shadow-sm hover:shadow"
  >
    <img class="object-cover" src={thumbnail} alt="" srcset="" />
    <div class="flex items-center space-x-4 px-4">
      <img src={youTubeLogo} alt="YouTube" width="24px" />
      <p class="py-2 text-sm font-medium leading-tight">
        {videoTitle}
      </p>
    </div>
  </div>
</a>
