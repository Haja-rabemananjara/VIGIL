import { describe, it, expect, vi, beforeEach, afterEach } from "vitest";
import { isDesktop, notify, getApiUrl } from "./platform";

describe("isDesktop", () => {
  afterEach(() => {
    // Cleanup: remove Tauri globals we may have added
    // @ts-expect-error - test cleanup
    delete window.__TAURI__;
    // @ts-expect-error - test cleanup
    delete window.__TAURI_INTERNALS__;
  });

  it("returns false in a browser environment without Tauri", () => {
    expect(isDesktop()).toBe(false);
  });

  it("returns true when __TAURI__ is defined on window", () => {
    // @ts-expect-error - test setup
    window.__TAURI__ = {};
    expect(isDesktop()).toBe(true);
  });

  it("returns true when __TAURI_INTERNALS__ is defined on window", () => {
    // @ts-expect-error - test setup
    window.__TAURI_INTERNALS__ = {};
    expect(isDesktop()).toBe(true);
  });
});

describe("getApiUrl", () => {
  const originalEnv = process.env.NEXT_PUBLIC_API_URL;

  afterEach(() => {
    process.env.NEXT_PUBLIC_API_URL = originalEnv;
  });

  it("returns the configured URL when NEXT_PUBLIC_API_URL is set", () => {
    process.env.NEXT_PUBLIC_API_URL = "https://api.vigil.example";
    expect(getApiUrl()).toBe("https://api.vigil.example");
  });

  it("falls back to localhost:8080 when unset", () => {
    delete process.env.NEXT_PUBLIC_API_URL;
    expect(getApiUrl()).toBe("http://localhost:8080");
  });

  it("falls back to localhost:8080 when the env var is empty", () => {
    process.env.NEXT_PUBLIC_API_URL = "";
    expect(getApiUrl()).toBe("http://localhost:8080");
  });

  it("strips trailing slashes from the URL", () => {
    process.env.NEXT_PUBLIC_API_URL = "https://api.vigil.example/";
    expect(getApiUrl()).toBe("https://api.vigil.example");
  });
});

describe("notify", () => {
  const originalNotification = global.Notification;

  beforeEach(() => {
    // Silence the console during these tests
    vi.spyOn(console, "log").mockImplementation(() => {});
  });

  afterEach(() => {
    vi.restoreAllMocks();
    global.Notification = originalNotification;
  });

  it("returns early when Notification API is absent", async () => {
    // @ts-expect-error - test setup
    delete global.Notification;
    await expect(notify("Title", "Body")).resolves.toBeUndefined();
  });

  it("creates a notification when permission is granted", async () => {
    const NotificationMock = vi.fn();
    // @ts-expect-error - static property
    NotificationMock.permission = "granted";
    // @ts-expect-error - test setup
    global.Notification = NotificationMock;
    // @ts-expect-error - required for jsdom
    window.Notification = NotificationMock;

    await notify("Test", "Body text");

    expect(NotificationMock).toHaveBeenCalledWith("Test", {
      body: "Body text",
    });
  });

  it("requests permission when it is default", async () => {
    const requestPermission = vi.fn().mockResolvedValue("denied");
    const NotificationMock = vi.fn();
    // @ts-expect-error - static properties
    NotificationMock.permission = "default";
    // @ts-expect-error - static properties
    NotificationMock.requestPermission = requestPermission;
    // @ts-expect-error - test setup
    global.Notification = NotificationMock;
    // @ts-expect-error - required for jsdom
    window.Notification = NotificationMock;

    await notify("Test", "Body");

    expect(requestPermission).toHaveBeenCalled();
  });

  it("does not create a notification when permission is denied", async () => {
    const NotificationMock = vi.fn();
    // @ts-expect-error - static property
    NotificationMock.permission = "denied";
    // @ts-expect-error - test setup
    global.Notification = NotificationMock;
    // @ts-expect-error - required for jsdom
    window.Notification = NotificationMock;

    await notify("Test", "Body");

    expect(NotificationMock).not.toHaveBeenCalled();
  });
});
