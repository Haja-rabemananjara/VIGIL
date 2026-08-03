import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, screen, fireEvent, waitFor } from "@testing-library/react";

// Mock api
vi.mock("@/lib/api", () => ({
  api: vi.fn(),
  ApiError: class ApiError extends Error {
    constructor(
      public status: number,
      public code: string,
      message: string,
    ) {
      super(message);
      this.name = "ApiError";
    }
  },
}));

const mockChangeLanguage = vi.fn();
const mockUpdateUser = vi.fn();
vi.mock("@/stores/auth", () => ({
  useAuth: () => ({
    user: {
      id: "u1",
      email: "test@example.com",
      display_name: "Tester",
      language: "en",
      created_at: 1718000000,
    },
    token: "test-token",
    language: "en",
    changeLanguage: mockChangeLanguage,
    updateUser: mockUpdateUser,
  }),
}));

vi.mock("next/navigation", () => ({
  useRouter: () => ({ back: vi.fn(), push: vi.fn(), replace: vi.fn() }),
  usePathname: () => "/settings/profile",
}));

import { api } from "@/lib/api";
import { ProfileClient } from "./client";

const mockApi = vi.mocked(api);

describe("ProfileClient", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it("renders the profile page with user info", () => {
    render(<ProfileClient />);
    expect(screen.getByText("test@example.com")).toBeInTheDocument();
    expect(screen.getByDisplayValue("Tester")).toBeInTheDocument();
  });

  it("renders language buttons", () => {
    render(<ProfileClient />);
    expect(screen.getByRole("button", { name: "English" })).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "Français" })).toBeInTheDocument();
  });

  it("renders password section", () => {
    render(<ProfileClient />);
    expect(document.querySelector('input[type="password"]')).toBeInTheDocument();
  });

  it("disables save when display name unchanged", () => {
    render(<ProfileClient />);
    const saveButtons = screen.getAllByRole("button", { name: /save/i });
    expect(saveButtons[0]).toBeDisabled();
  });

  it("enables save when display name changes", () => {
    render(<ProfileClient />);
    const input = screen.getByDisplayValue("Tester");
    fireEvent.change(input, { target: { value: "New Name" } });
    const saveButtons = screen.getAllByRole("button", { name: /save/i });
    expect(saveButtons[0]).not.toBeDisabled();
  });

  it("calls PATCH /me on name save", async () => {
    mockApi.mockResolvedValueOnce({
      id: "u1",
      email: "test@example.com",
      display_name: "New Name",
      language: "en",
      created_at: 1718000000,
    });

    render(<ProfileClient />);
    const input = screen.getByDisplayValue("Tester");
    fireEvent.change(input, { target: { value: "New Name" } });

    const saveButtons = screen.getAllByRole("button", { name: /save/i });
    fireEvent.click(saveButtons[0]);

    await waitFor(() => {
      expect(mockApi).toHaveBeenCalledWith("/me", {
        method: "PATCH",
        token: "test-token",
        body: { display_name: "New Name" },
      });
    });

    expect(mockUpdateUser).toHaveBeenCalledWith({ display_name: "New Name" });
  });

  it("calls PATCH /me on language change", async () => {
    mockApi.mockResolvedValueOnce({
      id: "u1",
      email: "test@example.com",
      display_name: "Tester",
      language: "fr",
      created_at: 1718000000,
    });

    render(<ProfileClient />);
    fireEvent.click(screen.getByRole("button", { name: "Français" }));

    await waitFor(() => {
      expect(mockApi).toHaveBeenCalledWith("/me", {
        method: "PATCH",
        token: "test-token",
        body: { language: "fr" },
      });
    });

    expect(mockChangeLanguage).toHaveBeenCalledWith("fr");
  });

  it("rejects short password client-side", async () => {
    render(<ProfileClient />);
    const pwInput = document.querySelector('input[type="password"]') as HTMLInputElement;
    fireEvent.change(pwInput, { target: { value: "short" } });

    const saveButtons = screen.getAllByRole("button", { name: /save/i });
    fireEvent.click(saveButtons[1]);

    await waitFor(() => {
      expect(screen.getByText(/at least 8/i)).toBeInTheDocument();
    });

    expect(mockApi).not.toHaveBeenCalled();
  });

  it("has a back button", () => {
    render(<ProfileClient />);
    expect(screen.getByRole("button", { name: /back/i })).toBeInTheDocument();
  });
});