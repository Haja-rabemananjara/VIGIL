import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, screen, fireEvent, waitFor } from "@testing-library/react";
import { api, ApiError } from "@/lib/api";
import { ServicesClient } from "./client";

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

const mockUseAuth = vi.fn(() => ({ token: "test-token" }));
vi.mock("@/stores/auth", () => ({
  useAuth: () => mockUseAuth(),
}));

const mockApi = vi.mocked(api);

vi.mock("next/navigation", () => ({
  useRouter: () => ({ back: vi.fn(), push: vi.fn(), replace: vi.fn() }),
  usePathname: () => "/settings/services",
}));

const mockAbout = {
  server: {
    services: [
      { name: "github", connectable: true },
      { name: "discord", connectable: true },
      { name: "vigil", connectable: false },
    ],
  },
};

const mockConnection = {
  id: "conn-1",
  service: "github",
  created_at: 1718000000,
  updated_at: 1718000000,
};

describe("ServicesClient", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it("shows a loading state initially", () => {
    mockApi.mockImplementation(() => new Promise(() => {}));
    render(<ServicesClient />);
    expect(screen.getByText(/loading/i)).toBeInTheDocument();
  });

  it("shows an error state when the fetch fails", async () => {
    mockApi.mockRejectedValue(new Error("network"));
    render(<ServicesClient />);
    await waitFor(() => {
      expect(
        screen.getByText(/something went wrong|error/i),
      ).toBeInTheDocument();
    });
  });

  it("shows the empty state when no connectable services", async () => {
    mockApi
      .mockResolvedValueOnce({ server: { services: [] } })
      .mockResolvedValueOnce([]);
    render(<ServicesClient />);
    await waitFor(() => {
      expect(screen.getByText(/no connectable/i)).toBeInTheDocument();
    });
  });

  it("renders one card per connectable service", async () => {
    mockApi.mockResolvedValueOnce(mockAbout).mockResolvedValueOnce([]);
    render(<ServicesClient />);
    await waitFor(() => {
      expect(screen.getByText("github")).toBeInTheDocument();
      expect(screen.getByText("discord")).toBeInTheDocument();
    });
    expect(screen.queryByText("vigil")).not.toBeInTheDocument();
  });

  it("shows 'Not connected' when no connection exists", async () => {
    mockApi.mockResolvedValueOnce(mockAbout).mockResolvedValueOnce([]);
    render(<ServicesClient />);
    await waitFor(() => {
      const labels = screen.getAllByText(/not connected/i);
      expect(labels.length).toBe(2);
    });
  });

  it("shows 'Connected' when a connection exists", async () => {
    mockApi
      .mockResolvedValueOnce(mockAbout)
      .mockResolvedValueOnce([mockConnection]);
    render(<ServicesClient />);
    await waitFor(() => {
      const labels = screen.getAllByText(/^connected/i);
      expect(labels.length).toBeGreaterThan(0);
    });
  });

  it("shows the connect input for non-connected services", async () => {
    mockApi.mockResolvedValueOnce(mockAbout).mockResolvedValueOnce([]);
    render(<ServicesClient />);
    await waitFor(() => {
      // Two password inputs (github + discord, both non-connected)
      const inputs = document.querySelectorAll('input[type="password"]');
      expect(inputs.length).toBe(2);
    });
  });

  it("shows the disconnect button for connected services", async () => {
    mockApi
      .mockResolvedValueOnce(mockAbout)
      .mockResolvedValueOnce([mockConnection]);
    render(<ServicesClient />);
    await waitFor(() => {
      expect(
        screen.getAllByRole("button", { name: /disconnect/i }).length,
      ).toBeGreaterThan(0);
    });
  });

  it("shows an error when the token is empty on connect", async () => {
    mockApi.mockResolvedValueOnce(mockAbout).mockResolvedValueOnce([]);
    render(<ServicesClient />);

    // Wait for input to appear
    await waitFor(() =>
      expect(
        document.querySelector('input[type="password"]'),
      ).toBeInTheDocument(),
    );

    // Click the first "Connect" button without filling anything
    const connectBtns = screen.getAllByRole("button", { name: /^connect$/i });
    fireEvent.click(connectBtns[0]);

    await waitFor(() => {
      expect(screen.getByText(/token is required|empty/i)).toBeInTheDocument();
    });
  });

  it("calls the API and updates state on successful connect", async () => {
    mockApi
      .mockResolvedValueOnce(mockAbout)
      .mockResolvedValueOnce([])
      .mockResolvedValueOnce(mockConnection);

    render(<ServicesClient />);

    await waitFor(() =>
      expect(
        document.querySelector('input[type="password"]'),
      ).toBeInTheDocument(),
    );

    const inputs = document.querySelectorAll('input[type="password"]');
    fireEvent.change(inputs[0], { target: { value: "my-token" } });

    const connectBtns = screen.getAllByRole("button", { name: /^connect$/i });
    fireEvent.click(connectBtns[0]);

    await waitFor(() => {
      expect(mockApi).toHaveBeenCalledWith(
        "/me/services/github",
        expect.objectContaining({
          method: "POST",
          body: { token: "my-token" },
        }),
      );
    });
  });

  it("surfaces API errors on failed connect", async () => {
    mockApi
      .mockResolvedValueOnce(mockAbout)
      .mockResolvedValueOnce([])
      .mockRejectedValueOnce(new ApiError(401, "unauthorized", "Bad token"));

    render(<ServicesClient />);

    await waitFor(() =>
      expect(
        document.querySelector('input[type="password"]'),
      ).toBeInTheDocument(),
    );

    const inputs = document.querySelectorAll('input[type="password"]');
    fireEvent.change(inputs[0], { target: { value: "wrong-token" } });

    const connectBtns = screen.getAllByRole("button", { name: /^connect$/i });
    fireEvent.click(connectBtns[0]);

    await waitFor(() => {
      expect(screen.getByText("Bad token")).toBeInTheDocument();
    });
  });
});
