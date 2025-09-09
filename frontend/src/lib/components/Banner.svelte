<script lang="ts">
  import { resolve } from "$app/paths";
  import cipherlyLogo from "$lib/assets/cipherly.svg";
</script>

<div class="relative container mx-auto px-5">
  <div class="mb-8 flex w-full flex-col items-center space-y-4 text-center">
    <a href={resolve("/")}>
      <div class="flex items-center space-x-2">
        <img alt="The project logo" src={cipherlyLogo} width="40px" />
        <div class="text-4xl font-extrabold uppercase">Cipherly</div>
      </div>
    </a>
  </div>
</div>
