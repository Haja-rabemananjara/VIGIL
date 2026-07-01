/**
 * Platform abstraction layer.
 *
 * Centralizes all native capability access. Components never call
 * window.__TAURI__ or Notification API directly, they go through here.
 *
 * Web build: uses browser fallbacks (or no-ops when no equivalent exists).
 * Desktop build (Tauri,): implementations will be swapped to call
 * @tauri-apps/api functions. Component code stays untouched.
 */

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
