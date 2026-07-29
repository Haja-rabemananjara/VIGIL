/**
 * Detects whether we are running inside the Tauri desktop shell.
 * Tauri injects `window.__TAURI__` at app startup.
 */
export function isDesktop(): boolean {
  if (typeof window === "undefined") return false;
  return "__TAURI_INTERNALS__" in window || "__TAURI__" in window;
}

/**
 * Trigger a notification visible to the user.
 * * Uses the browser Notification API in both web and Tauri contexts.
 * Tauri's WebView (WebKitGTK on Linux) supports this API natively,
 * so no platform-specific branch is needed.
 */
export async function notify(title: string, body: string): Promise<void> {
  if (typeof window === "undefined") return;

  // web
  if (!("Notification" in window)) return;

  if (Notification.permission === "default") {
    await Notification.requestPermission();
  }

  if (Notification.permission === "granted") {
    new Notification(title, { body });
  }
}

/**
 * Single source of truth for the VIGIL backend base URL.
 *
 * Read from NEXT_PUBLIC_API_URL at build time, with a localhost dev default.
 * Centralized here so api.ts and socket.ts never duplicate the URL logic.
 */
export function getApiUrl(): string {
  const configured = process.env.NEXT_PUBLIC_API_URL;
  const base =
    configured && configured.length > 0 ? configured : "http://localhost:8080";
  return base.replace(/\/$/, "");
}
