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
 */
export async function notify(title: string, body: string): Promise<void> {
  console.log("[notify] called", { title });
  if (typeof window === "undefined") return;

  // web
  if (!("Notification" in window)) {
    console.log("[notify] Notification API absente");
    return;
  }
  console.log("[notify] permission:", Notification.permission);
  if (Notification.permission === "default") {
    const p = await Notification.requestPermission();
    console.log("[notify] after request:", p);
  }
  if (Notification.permission === "granted") {
    new Notification(title, { body });
    console.log("[notify] sent via browser API");
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
