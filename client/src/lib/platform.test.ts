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
  const originalFetch = global.fetch;

  beforeEach(() => {
    global.fetch = vi.fn().mockResolvedValue({});
    delete (window as unknown as Record<string, unknown>).__TAURI__;
    delete (window as unknown as Record<string, unknown>).__TAURI_INTERNALS__;
  });

  afterEach(() => {
    global.fetch = originalFetch;
    delete (window as unknown as Record<string, unknown>).__TAURI__;
    delete (window as unknown as Record<string, unknown>).__TAURI_INTERNALS__;
  });

  it("returns early when not in desktop mode", async () => {
    await notify("Test", "Body");
    expect(global.fetch).not.toHaveBeenCalled();
  });

  it("sends a fetch to __notify endpoint when in desktop mode", async () => {
    (window as unknown as Record<string, unknown>).__TAURI__ = true;

    await notify("Test", "Body text");

    expect(global.fetch).toHaveBeenCalledWith(
      "http://localhost:9527/__notify",
      expect.objectContaining({
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ title: "Test", body: "Body text" }),
      }),
    );
  });

  it("works with __TAURI_INTERNALS__ too", async () => {
    (window as unknown as Record<string, unknown>).__TAURI_INTERNALS__ = true;

    await notify("Alert", "Something happened");

    expect(global.fetch).toHaveBeenCalledWith(
      "http://localhost:9527/__notify",
      expect.objectContaining({ method: "POST" }),
    );
  });

  it("does not throw when fetch fails", async () => {
    (window as unknown as Record<string, unknown>).__TAURI__ = true;
    (global.fetch as ReturnType<typeof vi.fn>).mockRejectedValueOnce(
      new Error("network"),
    );

    await expect(notify("Test", "Body")).resolves.toBeUndefined();
  });
});
