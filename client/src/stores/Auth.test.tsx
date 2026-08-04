import { describe, it, expect, vi, beforeEach, afterEach } from "vitest";
import { render, screen, act, waitFor } from "@testing-library/react";
import { AuthProvider, useAuth } from "./auth";

// Mock the api module
vi.mock("@/lib/api", () => ({
  api: vi.fn(),
}));

import { api } from "@/lib/api";
const mockApi = vi.mocked(api);

const mockUser = {
  id: "user-1",
  email: "alice@example.com",
  display_name: "Alice",
  language: "en",
  avatar_seed: null,
  created_at: 1718000000,
};

function AuthConsumer({
  onRender,
}: {
  onRender: (v: ReturnType<typeof useAuth>) => void;
}) {
  const value = useAuth();
  onRender(value);
  return (
    <div>
      <span data-testid="user">{value.user?.email ?? "no-user"}</span>
      <span data-testid="token">{value.token ?? "no-token"}</span>
      <span data-testid="loading">{value.isLoading ? "loading" : "ready"}</span>
    </div>
  );
}

describe("AuthProvider", () => {
  beforeEach(() => {
    localStorage.clear();
    vi.clearAllMocks();
  });

  afterEach(() => {
    localStorage.clear();
  });

  it("starts with no user and no token when localStorage is empty", async () => {
    const captured: Array<ReturnType<typeof useAuth>> = [];
    render(
      <AuthProvider>
        <AuthConsumer onRender={(v) => captured.push(v)} />
      </AuthProvider>,
    );
    expect(screen.getByTestId("user").textContent).toBe("no-user");
    expect(screen.getByTestId("token").textContent).toBe("no-token");
  });

  it("loads the user when a token exists in localStorage", async () => {
    localStorage.setItem("vigil_token", "stored-token");
    mockApi.mockResolvedValue(mockUser);

    render(
      <AuthProvider>
        <AuthConsumer onRender={() => {}} />
      </AuthProvider>,
    );

    await waitFor(() => {
      expect(screen.getByTestId("user").textContent).toBe("alice@example.com");
    });
    expect(mockApi).toHaveBeenCalledWith("/me", { token: "stored-token" });
  });

  it("clears the token when /me fails on startup", async () => {
    localStorage.setItem("vigil_token", "invalid-token");
    mockApi.mockRejectedValue(new Error("401"));

    render(
      <AuthProvider>
        <AuthConsumer onRender={() => {}} />
      </AuthProvider>,
    );

    await waitFor(() => {
      expect(screen.getByTestId("token").textContent).toBe("no-token");
    });
    expect(localStorage.getItem("vigil_token")).toBeNull();
  });

  it("signin stores token and user", async () => {
    let capturedValue: ReturnType<typeof useAuth> | null = null;
    render(
      <AuthProvider>
        <AuthConsumer onRender={(v) => (capturedValue = v)} />
      </AuthProvider>,
    );

    mockApi.mockResolvedValueOnce({ token: "new-token", user: mockUser });

    await act(async () => {
      await capturedValue!.signin("alice@example.com", "password123");
    });

    await waitFor(() => {
      expect(screen.getByTestId("user").textContent).toBe("alice@example.com");
      expect(screen.getByTestId("token").textContent).toBe("new-token");
    });
    expect(localStorage.getItem("vigil_token")).toBe("new-token");
  });

  it("signup calls the signup endpoint then signs in", async () => {
    let capturedValue: ReturnType<typeof useAuth> | null = null;
    render(
      <AuthProvider>
        <AuthConsumer onRender={(v) => (capturedValue = v)} />
      </AuthProvider>,
    );

    mockApi.mockResolvedValueOnce(mockUser);
    mockApi.mockResolvedValueOnce({ token: "signup-token", user: mockUser });

    await act(async () => {
      await capturedValue!.signup("alice@example.com", "password123", "Alice");
    });

    expect(mockApi).toHaveBeenNthCalledWith(1, "/auth/signup", {
      method: "POST",
      body: {
        email: "alice@example.com",
        password: "password123",
        display_name: "Alice",
      },
    });

    expect(mockApi).toHaveBeenNthCalledWith(2, "/auth/signin", {
      method: "POST",
      body: { email: "alice@example.com", password: "password123" },
    });

    await waitFor(() => {
      expect(screen.getByTestId("user").textContent).toBe("alice@example.com");
      expect(screen.getByTestId("token").textContent).toBe("signup-token");
    });
  });

  it("signout clears user, token, and localStorage", async () => {
    localStorage.setItem("vigil_token", "existing-token");
    mockApi.mockResolvedValueOnce(mockUser);

    let capturedValue: ReturnType<typeof useAuth> | null = null;
    render(
      <AuthProvider>
        <AuthConsumer onRender={(v) => (capturedValue = v)} />
      </AuthProvider>,
    );

    await waitFor(() => {
      expect(screen.getByTestId("user").textContent).toBe("alice@example.com");
    });

    mockApi.mockResolvedValueOnce(undefined);

    await act(async () => {
      await capturedValue!.signout();
    });

    expect(screen.getByTestId("user").textContent).toBe("no-user");
    expect(screen.getByTestId("token").textContent).toBe("no-token");
    expect(localStorage.getItem("vigil_token")).toBeNull();
  });

  it("signout still clears local state when the API call fails", async () => {
    localStorage.setItem("vigil_token", "existing-token");
    mockApi.mockResolvedValueOnce(mockUser);

    let capturedValue: ReturnType<typeof useAuth> | null = null;
    render(
      <AuthProvider>
        <AuthConsumer onRender={(v) => (capturedValue = v)} />
      </AuthProvider>,
    );

    await waitFor(() => {
      expect(screen.getByTestId("user").textContent).toBe("alice@example.com");
    });
    mockApi.mockRejectedValueOnce(new Error("Network error"));

    await act(async () => {
      await capturedValue!.signout();
    });

    expect(screen.getByTestId("user").textContent).toBe("no-user");
    expect(screen.getByTestId("token").textContent).toBe("no-token");
    expect(localStorage.getItem("vigil_token")).toBeNull();
  });
});

describe("useAuth", () => {
  it("throws when used outside of an AuthProvider", () => {
    const err = vi.spyOn(console, "error").mockImplementation(() => {});
    expect(() => render(<AuthConsumer onRender={() => {}} />)).toThrow(
      /AuthProvider/,
    );
    err.mockRestore();
  });
});
