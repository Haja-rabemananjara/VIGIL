

/**
 * Detects whether we are running inside the Tauri desktop shell.
 * Tauri injects `window.__TAURI__` at app startup.
 */
export function isDesktop(): boolean {
  if (typeof window === "undefined") return false;
  return "__TAURI__" in window;
}

/**
 * Trigger a notification visible to the user.
 *
 * Web: uses the browser Notification API (asks permission once).
 * Desktop : will use @tauri-apps/api/notification.
 */
export async function notify(title: string, body: string): Promise<void> {
  if (typeof window === "undefined") return;

  // Web fallback: ask permission then notify
  if (!("Notification" in window)) {
    console.warn("Notifications not supported in this browser");
    return;
  }

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