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
    token: string | null;
  };
  let {
    token = $bindable(sessionStorage.getItem(CREDENTIAL_KEY) || null),
  }: Props = $props();

  function logout() {
    googleLogout();
    token = null;
    sessionStorage.removeItem(CREDENTIAL_KEY);
  }

  function set_login_token(newToken: string) {
    sessionStorage.setItem(CREDENTIAL_KEY, newToken);
    token = newToken;
  }

  window.set_login_token = set_login_token;

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

  let user = $derived.by(() => {
    if (token === null) {
      return null;
    }
    const user = jwtDecode(token) as User;
    if (user.exp * 1000 < Date.now()) {
      return null;
    }
    return user;
  });
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
    <Button onclick={() => logout()} color="secondary">Logout</Button>
  </div>
{/if}
