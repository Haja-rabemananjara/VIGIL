import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, screen, fireEvent, waitFor } from "@testing-library/react";

// Mocks
const mockSignin = vi.fn();
vi.mock("@/stores/auth", () => ({
  useAuth: () => ({ signin: mockSignin }),
}));

const mockPush = vi.fn();
vi.mock("next/navigation", () => ({
  useRouter: () => ({ push: mockPush }),
}));

const mockPostLoginDestination = vi.fn();
vi.mock("@/lib/navigation", () => ({
  postLoginDestination: (...args: unknown[]) =>
    mockPostLoginDestination(...args),
}));

// Mock ApiError with a working constructor
vi.mock("@/lib/api", () => {
  class ApiError extends Error {
    constructor(
      public status: number,
      public code: string,
      message: string,
    ) {
      super(message);
      this.name = "ApiError";
    }
  }
  return { ApiError };
});

import { ApiError } from "@/lib/api";
import SigninPage from "./page";

describe("SigninPage", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    mockSignin.mockResolvedValue(undefined);
    mockPostLoginDestination.mockResolvedValue("/teams/team-1/incidents");
  });

  it("renders email and password inputs", () => {
    render(<SigninPage />);
    expect(screen.getByLabelText(/email/i)).toBeInTheDocument();
    expect(screen.getByLabelText(/^password$/i)).toBeInTheDocument();
  });

  it("renders a submit button", () => {
    render(<SigninPage />);
    expect(
      screen.getByRole("button", { name: /^sign in$/i }),
    ).toBeInTheDocument();
  });

  it("renders a link to the signup page", () => {
    render(<SigninPage />);
    const link = screen.getByRole("link", { name: /sign up|don't have/i });
    expect(link).toHaveAttribute("href", "/signup");
  });

  it("updates the email field when typed into", () => {
    render(<SigninPage />);
    const emailInput = screen.getByLabelText(/email/i) as HTMLInputElement;
    fireEvent.change(emailInput, { target: { value: "alice@example.com" } });
    expect(emailInput.value).toBe("alice@example.com");
  });

  it("updates the password field when typed into", () => {
    render(<SigninPage />);
    const passwordInput = screen.getByLabelText(
      /^password$/i,
    ) as HTMLInputElement;
    fireEvent.change(passwordInput, { target: { value: "secret123" } });
    expect(passwordInput.value).toBe("secret123");
  });

  it("hides the password by default", () => {
    render(<SigninPage />);
    const passwordInput = screen.getByLabelText(
      /^password$/i,
    ) as HTMLInputElement;
    expect(passwordInput.type).toBe("password");
  });

  it("shows the password when the toggle button is clicked", () => {
    render(<SigninPage />);
    const passwordInput = screen.getByLabelText(
      /^password$/i,
    ) as HTMLInputElement;
    const toggleBtn = screen.getByRole("button", { name: /show password/i });
    fireEvent.click(toggleBtn);
    expect(passwordInput.type).toBe("text");
  });

  it("hides the password again when toggle is clicked twice", () => {
    render(<SigninPage />);
    const passwordInput = screen.getByLabelText(
      /^password$/i,
    ) as HTMLInputElement;
    const toggleBtn = screen.getByRole("button", { name: /show password/i });
    fireEvent.click(toggleBtn);
    fireEvent.click(screen.getByRole("button", { name: /hide password/i }));
    expect(passwordInput.type).toBe("password");
  });

  it("calls signin with the entered credentials on submit", async () => {
    render(<SigninPage />);
    fireEvent.change(screen.getByLabelText(/email/i), {
      target: { value: "alice@example.com" },
    });
    fireEvent.change(screen.getByLabelText(/^password$/i), {
      target: { value: "password123" },
    });
    fireEvent.click(screen.getByRole("button", { name: /^sign in$/i }));

    await waitFor(() => {
      expect(mockSignin).toHaveBeenCalledWith(
        "alice@example.com",
        "password123",
      );
    });
  });

  it("shows an error message when signin fails with ApiError", async () => {
    mockSignin.mockRejectedValueOnce(
      new ApiError(401, "invalid_credentials", "Invalid email or password"),
    );

    render(<SigninPage />);
    fireEvent.change(screen.getByLabelText(/email/i), {
      target: { value: "alice@example.com" },
    });
    fireEvent.change(screen.getByLabelText(/^password$/i), {
      target: { value: "wrong-password" },
    });
    fireEvent.click(screen.getByRole("button", { name: /^sign in$/i }));

    await waitFor(() => {
      expect(screen.getByText("Invalid email or password")).toBeInTheDocument();
    });
  });

  it("shows a generic error message on non-ApiError failure", async () => {
    mockSignin.mockRejectedValueOnce(new Error("network failure"));

    render(<SigninPage />);
    fireEvent.change(screen.getByLabelText(/email/i), {
      target: { value: "alice@example.com" },
    });
    fireEvent.change(screen.getByLabelText(/^password$/i), {
      target: { value: "password123" },
    });
    fireEvent.click(screen.getByRole("button", { name: /^sign in$/i }));

    await waitFor(() => {
      expect(screen.getByText(/error|went wrong|failed/i)).toBeInTheDocument();
    });
  });

  it("disables the submit button while loading", async () => {
    mockSignin.mockImplementation(() => new Promise(() => {}));

    render(<SigninPage />);
    fireEvent.change(screen.getByLabelText(/email/i), {
      target: { value: "alice@example.com" },
    });
    fireEvent.change(screen.getByLabelText(/^password$/i), {
      target: { value: "password123" },
    });
    fireEvent.click(screen.getByRole("button", { name: /^sign in$/i }));

    await waitFor(() => {
      expect(screen.getByRole("button", { name: /loading/i })).toBeDisabled();
    });
  });
});
