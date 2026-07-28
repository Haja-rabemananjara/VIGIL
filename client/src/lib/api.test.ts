import { describe, it, expect, vi, beforeEach, afterEach } from "vitest";
import { api, ApiError } from "./api";

describe("api", () => {
  const originalFetch = global.fetch;

  beforeEach(() => {
    global.fetch = vi.fn();
  });

  afterEach(() => {
    global.fetch = originalFetch;
    vi.clearAllMocks();
  });

  it("makes a GET request by default", async () => {
    (global.fetch as ReturnType<typeof vi.fn>).mockResolvedValue({
      ok: true,
      status: 200,
      text: async () => JSON.stringify({ hello: "world" }),
    });

    const result = await api<{ hello: string }>("/test");

    expect(global.fetch).toHaveBeenCalledWith(
      expect.stringContaining("/test"),
      expect.objectContaining({ method: "GET" }),
    );
    expect(result).toEqual({ hello: "world" });
  });

  it("makes a POST request with a JSON body", async () => {
    (global.fetch as ReturnType<typeof vi.fn>).mockResolvedValue({
      ok: true,
      status: 200,
      text: async () => JSON.stringify({ ok: true }),
    });

    await api("/create", { method: "POST", body: { name: "Alice" } });

    const call = (global.fetch as ReturnType<typeof vi.fn>).mock.calls[0];
    expect(call[1].method).toBe("POST");
    expect(call[1].headers["Content-Type"]).toBe("application/json");
    expect(JSON.parse(call[1].body)).toEqual({ name: "Alice" });
  });

  it("attaches an Authorization header when a token is provided", async () => {
    (global.fetch as ReturnType<typeof vi.fn>).mockResolvedValue({
      ok: true,
      status: 200,
      text: async () => JSON.stringify({}),
    });

    await api("/me", { token: "my-token" });

    const call = (global.fetch as ReturnType<typeof vi.fn>).mock.calls[0];
    expect(call[1].headers["Authorization"]).toBe("Bearer my-token");
  });

  it("does not attach an Authorization header when no token is provided", async () => {
    (global.fetch as ReturnType<typeof vi.fn>).mockResolvedValue({
      ok: true,
      status: 200,
      text: async () => JSON.stringify({}),
    });

    await api("/public");

    const call = (global.fetch as ReturnType<typeof vi.fn>).mock.calls[0];
    expect(call[1].headers["Authorization"]).toBeUndefined();
  });

  it("returns undefined for 204 No Content responses", async () => {
    (global.fetch as ReturnType<typeof vi.fn>).mockResolvedValue({
      ok: true,
      status: 204,
      text: async () => "",
    });

    const result = await api("/signout", { method: "POST" });
    expect(result).toBeUndefined();
  });

  it("throws ApiError on non-2xx responses", async () => {
    (global.fetch as ReturnType<typeof vi.fn>).mockResolvedValue({
      ok: false,
      status: 401,
      text: async () =>
        JSON.stringify({
          error: { code: "UNAUTHORIZED", message: "Invalid credentials" },
        }),
    });

    await expect(api("/protected")).rejects.toBeInstanceOf(ApiError);
  });

  it("attaches the server error code and message to ApiError", async () => {
    (global.fetch as ReturnType<typeof vi.fn>).mockResolvedValue({
      ok: false,
      status: 422,
      text: async () =>
        JSON.stringify({
          error: { code: "VALIDATION_ERROR", message: "Bad payload" },
        }),
    });

    try {
      await api("/broken");
      expect.fail("should have thrown");
    } catch (e) {
      expect(e).toBeInstanceOf(ApiError);
      const err = e as ApiError;
      expect(err.status).toBe(422);
      expect(err.code).toBe("VALIDATION_ERROR");
      expect(err.message).toBe("Bad payload");
    }
  });

  it("falls back to a default error message when server body is empty", async () => {
    (global.fetch as ReturnType<typeof vi.fn>).mockResolvedValue({
      ok: false,
      status: 500,
      text: async () => "",
    });

    try {
      await api("/error");
      expect.fail("should have thrown");
    } catch (e) {
      const err = e as ApiError;
      expect(err.status).toBe(500);
      expect(err.code).toBe("unknown");
      expect(err.message).toContain("500");
    }
  });

  it("does not send a Content-Type header when body is undefined", async () => {
    (global.fetch as ReturnType<typeof vi.fn>).mockResolvedValue({
      ok: true,
      status: 200,
      text: async () => "{}",
    });

    await api("/simple", { method: "GET" });

    const call = (global.fetch as ReturnType<typeof vi.fn>).mock.calls[0];
    expect(call[1].headers["Content-Type"]).toBeUndefined();
  });
});

describe("ApiError", () => {
  it("preserves the status, code, and message", () => {
    const err = new ApiError(403, "FORBIDDEN", "Access denied");
    expect(err.status).toBe(403);
    expect(err.code).toBe("FORBIDDEN");
    expect(err.message).toBe("Access denied");
    expect(err.name).toBe("ApiError");
  });

  it("is instanceof Error", () => {
    const err = new ApiError(404, "NOT_FOUND", "Missing");
    expect(err).toBeInstanceOf(Error);
  });
});
