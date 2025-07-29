import type { Handle } from "@sveltejs/kit";

export const handle: Handle = async ({ event, resolve }) => {
  // Handle chrome workspace requests to avoid cluttering development logs.
  // See https://github.com/sveltejs/kit/issues/13743
  // and https://chromium.googlesource.com/devtools/devtools-frontend/+/main/docs/ecosystem/automatic_workspace_folders.md
  if (
    event.url.pathname.startsWith(
      "/.well-known/appspecific/com.chrome.devtools",
    )
  ) {
    return new Response(null, { status: 204 }); // Return empty response with 204 No Content
  }

  return await resolve(event);
};
