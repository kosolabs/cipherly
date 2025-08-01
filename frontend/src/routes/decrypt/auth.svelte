<script lang="ts">
  import googleLogo from "$lib/assets/google.svg";
  import type { User } from "$lib/auth";
  import Avatar from "$lib/components/Avatar.svelte";
  import { Button } from "$lib/components/ui/button";
  import Skeleton from "$lib/components/ui/skeleton/skeleton.svelte";
  import { GoogleOAuthProvider, googleLogout } from "google-oauth-gsi";
  import { jwtDecode } from "jwt-decode";
  import { onMount } from "svelte";

  const CREDENTIAL_KEY = "credential";

  export let token: string | null =
    sessionStorage.getItem(CREDENTIAL_KEY) || null;

  function logout() {
    googleLogout();
    token = null;
    sessionStorage.removeItem(CREDENTIAL_KEY);
  }

  onMount(() => {
    const googleProvider = new GoogleOAuthProvider({
      clientId:
        "981002175662-g8jr2n89bptsn8n9ds1fn5edfheojr7i.apps.googleusercontent.com",
      onScriptLoadSuccess: () => {
        googleProvider.useRenderButton({
          element: document.getElementById("login-button")!,
          use_fedcm_for_prompt: true,
          onSuccess: (res) => {
            if (!res.credential) {
              console.error("Credential is missing", res);
              return;
            }
            token = res.credential;
            sessionStorage.setItem(CREDENTIAL_KEY, res.credential);
          },
        })();
      },
    });
  });

  function decodeUser(token: string | null): User | null {
    if (token === null) {
      return null;
    }
    const user = jwtDecode(token) as User;
    if (user.exp * 1000 < Date.now()) {
      return null;
    }
    return user;
  }

  $: user = decodeUser(token);
</script>

<div class={user === null ? "" : "hidden"}>
  <div id="login-button" class="w-[200px]" style="color-scheme:light">
    <Skeleton class="h-10" />
  </div>
</div>

{#if user !== null && token !== null}
  <div class="flex items-center space-x-4">
    <div
      class="bg-muted text-muted-foreground flex items-center space-x-4 rounded-3xl px-4 py-2"
    >
      <img src={googleLogo} alt="Google" class="h-4 w-4" />
      <p>
        Logged in as {user.name}
      </p>
      <Avatar {user} class="h-6 w-6" />
    </div>
    <Button on:click={() => logout()} variant="secondary">Logout</Button>
  </div>
{/if}
