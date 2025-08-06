<script lang="ts">
  import googleLogo from "$lib/assets/google.svg";
  import type { User } from "$lib/auth";
  import Avatar from "$lib/components/Avatar.svelte";
  import { GoogleOAuthProvider, googleLogout } from "google-oauth-gsi";
  import { jwtDecode } from "jwt-decode";
  import { Button, Skeleton } from "kosui";
  import { onMount } from "svelte";

  const CREDENTIAL_KEY = "credential";

  type Props = {
    onToken: (token?: string) => void;
  };
  let { onToken }: Props = $props();
  let token: string | undefined = $state();

  function logout() {
    setLoginToken();
  }

  function setLoginToken(newToken?: string) {
    if (newToken) {
      sessionStorage.setItem(CREDENTIAL_KEY, newToken);
    } else {
      googleLogout();
      sessionStorage.removeItem(CREDENTIAL_KEY);
    }
    token = newToken;
    onToken(newToken);
  }

  window.setLoginToken = setLoginToken;

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
            setLoginToken(res.credential);
          },
        })();
      },
    });
  });

  let user = $derived.by(() => {
    if (token === undefined) {
      return;
    }
    const user = jwtDecode(token) as User;
    if (user.exp * 1000 < Date.now()) {
      return;
    }
    return user;
  });
</script>

<div class={user ? "hidden" : ""}>
  <div id="login-button" class="w-[200px]" style="color-scheme:light">
    <Skeleton class="h-10" />
  </div>
</div>

{#if user && token}
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
    <Button onclick={() => logout()} color="secondary">Logout</Button>
  </div>
{/if}
