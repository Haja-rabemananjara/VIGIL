import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, screen, fireEvent, waitFor } from "@testing-library/react";

// Mocks
const mockSignup = vi.fn();
vi.mock("@/stores/auth", () => ({
  useAuth: () => ({ signup: mockSignup }),
}));

const mockPush = vi.fn();
vi.mock("next/navigation", () => ({
  useRouter: () => ({ push: mockPush }),
}));

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

import SignupPage from "./page";

describe("SignupPage", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    mockSignup.mockResolvedValue(undefined);
  });

  it("renders the display name, email, and password inputs", () => {
    render(<SignupPage />);
    expect(screen.getByLabelText(/name/i)).toBeInTheDocument();
    expect(screen.getByLabelText(/email/i)).toBeInTheDocument();
    expect(screen.getByLabelText(/^password$/i)).toBeInTheDocument();
  });

  it("renders a submit button", () => {
    render(<SignupPage />);
    expect(
      screen.getByRole("button", { name: /sign up/i }),
    ).toBeInTheDocument();
  });

  it("renders a link to the signin page", () => {
    render(<SignupPage />);
    const link = screen.getByRole("link", { name: /sign in|already/i });
    expect(link).toHaveAttribute("href", "/signin");
  });

  it("updates the display name field when typed into", () => {
    render(<SignupPage />);
    const input = screen.getByLabelText(/name/i) as HTMLInputElement;
    fireEvent.change(input, { target: { value: "Alice" } });
    expect(input.value).toBe("Alice");
  });

  it("updates the email field when typed into", () => {
    render(<SignupPage />);
    const input = screen.getByLabelText(/email/i) as HTMLInputElement;
    fireEvent.change(input, { target: { value: "alice@example.com" } });
    expect(input.value).toBe("alice@example.com");
  });

  it("updates the password field when typed into", () => {
    render(<SignupPage />);
    const input = screen.getByLabelText(/^password$/i) as HTMLInputElement;
    fireEvent.change(input, { target: { value: "secret123" } });
    expect(input.value).toBe("secret123");
  });

  it("hides the password by default", () => {
    render(<SignupPage />);
    const passwordInput = screen.getByLabelText(
      /^password$/i,
    ) as HTMLInputElement;
    expect(passwordInput.type).toBe("password");
  });

  it("shows the password when the toggle button is clicked", () => {
    render(<SignupPage />);
    const passwordInput = screen.getByLabelText(
      /^password$/i,
    ) as HTMLInputElement;
    const toggleBtn = screen.getByRole("button", { name: /show password/i });
    fireEvent.click(toggleBtn);
    expect(passwordInput.type).toBe("text");
  });

  it("hides the password again when toggle is clicked twice", () => {
    render(<SignupPage />);
    const passwordInput = screen.getByLabelText(
      /^password$/i,
    ) as HTMLInputElement;
    const toggleBtn = screen.getByRole("button", { name: /show password/i });
    fireEvent.click(toggleBtn);
    fireEvent.click(screen.getByRole("button", { name: /hide password/i }));
    expect(passwordInput.type).toBe("password");
  });

  it("calls signup with the entered fields on submit", async () => {
    render(<SignupPage />);
    fireEvent.change(screen.getByLabelText(/name/i), {
      target: { value: "Alice" },
    });
    fireEvent.change(screen.getByLabelText(/email/i), {
      target: { value: "alice@example.com" },
    });
    fireEvent.change(screen.getByLabelText(/^password$/i), {
      target: { value: "password123" },
    });
    fireEvent.click(screen.getByRole("button", { name: /sign up/i }));

    await waitFor(() => {
      expect(mockSignup).toHaveBeenCalledWith(
        "alice@example.com",
        "password123",
        "Alice",
      );
    });
  });

  it("shows a generic error message on non-ApiError failure", async () => {
    mockSignup.mockRejectedValueOnce(new Error("network failure"));

    render(<SignupPage />);
    fireEvent.change(screen.getByLabelText(/name/i), {
      target: { value: "Alice" },
    });
    fireEvent.change(screen.getByLabelText(/email/i), {
      target: { value: "alice@example.com" },
    });
    fireEvent.change(screen.getByLabelText(/^password$/i), {
      target: { value: "password123" },
    });
    fireEvent.click(screen.getByRole("button", { name: /sign up/i }));

    await waitFor(() => {
      expect(screen.getByText(/error|went wrong|failed/i)).toBeInTheDocument();
    });
  });

  it("disables the submit button while loading", async () => {
    mockSignup.mockImplementation(() => new Promise(() => {}));

    render(<SignupPage />);
    fireEvent.change(screen.getByLabelText(/name/i), {
      target: { value: "Alice" },
    });
    fireEvent.change(screen.getByLabelText(/email/i), {
      target: { value: "alice@example.com" },
    });
    fireEvent.change(screen.getByLabelText(/^password$/i), {
      target: { value: "password123" },
    });
    fireEvent.click(screen.getByRole("button", { name: /sign up/i }));

    await waitFor(() => {
      expect(screen.getByRole("button", { name: /loading/i })).toBeDisabled();
    });
  });
});
