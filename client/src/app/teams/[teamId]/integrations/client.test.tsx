import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, screen, fireEvent, waitFor } from "@testing-library/react";

const mockApi = vi.fn();
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
  return {
    api: (...args: unknown[]) => mockApi(...args),
    ApiError,
  };
});
import { ApiError } from "@/lib/api";

vi.mock("@/lib/useRouteParams", () => ({
  useRouteParams: () => ({ teamId: "t1" }),
}));

vi.mock("@/stores/auth", () => ({
  useAuth: () => ({
    user: { id: "me", display_name: "Alice", email: "a@t.com" },
    token: "tok",
    language: "en",
  }),
}));

// eslint-disable-next-line @typescript-eslint/no-unused-vars
let capturedOnConfirm: (() => void) | null = null;
vi.mock("@/components/ConfirmDialog", () => ({
  ConfirmDialog: (props: {
    open: boolean;
    title: string;
    description: string;
    onConfirm: () => void;
  }) => {
    capturedOnConfirm = props.onConfirm;
    if (!props.open) return null;
    return (
      <div data-testid="confirm-dialog">
        <p>{props.title}</p>
        <p>{props.description}</p>
        <button onClick={props.onConfirm}>Confirm disconnect</button>
      </div>
    );
  },
}));

import { IntegrationsClient } from "./client";

const GITHUB_CONNECTION = {
  id: "c1",
  team_id: "t1",
  service: "github",
  created_by: "me",
  created_at: 1700000000,
  updated_at: 1700000000,
};

const DISCORD_CONNECTION = {
  id: "c2",
  team_id: "t1",
  service: "discord",
  created_by: "me",
  created_at: 1700000000,
  updated_at: 1700000000,
};

describe("IntegrationsClient", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    capturedOnConfirm = null;
  });

  it("shows loading state", () => {
    mockApi.mockImplementation(() => new Promise(() => {}));
    render(<IntegrationsClient />);
    expect(screen.getByText("Loading...")).toBeInTheDocument();
  });

  it("shows error on fetch failure", async () => {
    mockApi.mockRejectedValue(new Error("fail"));
    render(<IntegrationsClient />);
    await waitFor(() =>
      expect(screen.getByText("Something went wrong")).toBeInTheDocument(),
    );
  });

  it("shows both services when no connections exist", async () => {
    mockApi.mockResolvedValueOnce([]);
    render(<IntegrationsClient />);
    await waitFor(() => {
      expect(screen.getByText("GitHub")).toBeInTheDocument();
      expect(screen.getByText("Discord")).toBeInTheDocument();
    });
    expect(screen.getAllByText("Not connected")).toHaveLength(2);
  });

  it("shows connected status for a connected service", async () => {
    mockApi.mockResolvedValueOnce([GITHUB_CONNECTION]);
    render(<IntegrationsClient />);
    await waitFor(() => {
      expect(screen.getByText("GitHub")).toBeInTheDocument();
      expect(screen.getByText("Connected")).toBeInTheDocument();
    });
  });

  it("shows both services with mixed connection status", async () => {
    mockApi.mockResolvedValueOnce([GITHUB_CONNECTION]);
    render(<IntegrationsClient />);
    await waitFor(() => {
      expect(screen.getByText("GitHub")).toBeInTheDocument();
      expect(screen.getByText("Discord")).toBeInTheDocument();
      expect(screen.getByText("Connected")).toBeInTheDocument();
      expect(screen.getByText("Not connected")).toBeInTheDocument();
    });
  });

  it("shows error when connecting with empty token", async () => {
    mockApi.mockResolvedValueOnce([]);
    render(<IntegrationsClient />);
    await waitFor(() =>
      expect(screen.getByText("GitHub")).toBeInTheDocument(),
    );

    const connectButtons = screen.getAllByText("Connect");
    fireEvent.click(connectButtons[0]);

    await waitFor(() =>
      expect(
        screen.getByText("A token or URL is required"),
      ).toBeInTheDocument(),
    );
  });

  it("connects GitHub and shows webhook URL", async () => {
    mockApi.mockResolvedValueOnce([]);
    mockApi.mockResolvedValueOnce({
      connection: GITHUB_CONNECTION,
      webhook_url: "https://example.com/webhooks/c1",
    });

    render(<IntegrationsClient />);
    await waitFor(() =>
      expect(screen.getByText("GitHub")).toBeInTheDocument(),
    );

    const input = screen.getByPlaceholderText(
      /secret you set in github/i,
    );
    fireEvent.change(input, { target: { value: "my-secret" } });

    const connectButtons = screen.getAllByText("Connect");
    fireEvent.click(connectButtons[0]);

    await waitFor(() => {
      expect(mockApi).toHaveBeenCalledWith(
        "/teams/t1/connections/github",
        expect.objectContaining({
          method: "POST",
          body: { token: "my-secret" },
        }),
      );
      expect(screen.getByDisplayValue("https://example.com/webhooks/c1")).toBeInTheDocument();
    });
  });

  it("connects Discord without webhook URL", async () => {
    mockApi.mockResolvedValueOnce([]);
    mockApi.mockResolvedValueOnce({
      connection: DISCORD_CONNECTION,
    });

    render(<IntegrationsClient />);
    await waitFor(() =>
      expect(screen.getByText("Discord")).toBeInTheDocument(),
    );

    const input = screen.getByPlaceholderText(
      /discord\.com\/api\/webhooks/i,
    );
    fireEvent.change(input, {
      target: { value: "https://discord.com/api/webhooks/123/abc" },
    });

    const connectButtons = screen.getAllByText("Connect");
    fireEvent.click(connectButtons[1]);

    await waitFor(() => {
      expect(mockApi).toHaveBeenCalledWith(
        "/teams/t1/connections/discord",
        expect.objectContaining({
          method: "POST",
          body: { token: "https://discord.com/api/webhooks/123/abc" },
        }),
      );
    });
  });

  it("shows error when connect fails", async () => {
    mockApi.mockResolvedValueOnce([]);
    mockApi.mockRejectedValueOnce(
      new ApiError(422, "validation", "Token cannot be empty"),
    );

    render(<IntegrationsClient />);
    await waitFor(() =>
      expect(screen.getByText("GitHub")).toBeInTheDocument(),
    );

    const input = screen.getByPlaceholderText(/secret you set in github/i);
    fireEvent.change(input, { target: { value: "bad" } });

    const connectButtons = screen.getAllByText("Connect");
    fireEvent.click(connectButtons[0]);

    await waitFor(() =>
      expect(screen.getByText("Token cannot be empty")).toBeInTheDocument(),
    );
  });

  it("opens confirm dialog and disconnects", async () => {
    mockApi.mockResolvedValueOnce([GITHUB_CONNECTION]);
    mockApi.mockResolvedValueOnce(undefined);

    render(<IntegrationsClient />);
    await waitFor(() =>
      expect(screen.getByText("Connected")).toBeInTheDocument(),
    );

    fireEvent.click(screen.getByText("Disconnect"));
    await waitFor(() =>
      expect(screen.getByTestId("confirm-dialog")).toBeInTheDocument(),
    );

    fireEvent.click(screen.getByText("Confirm disconnect"));

    await waitFor(() => {
      expect(mockApi).toHaveBeenCalledWith(
        "/teams/t1/connections/github",
        expect.objectContaining({ method: "DELETE" }),
      );
      expect(screen.queryByText("Connected")).not.toBeInTheDocument();
    });
  });

  it("shows error when disconnect fails", async () => {
    mockApi.mockResolvedValueOnce([GITHUB_CONNECTION]);
    mockApi.mockRejectedValueOnce(
      new ApiError(500, "err", "Disconnect failed"),
    );

    render(<IntegrationsClient />);
    await waitFor(() =>
      expect(screen.getByText("Connected")).toBeInTheDocument(),
    );

    fireEvent.click(screen.getByText("Disconnect"));
    await waitFor(() =>
      expect(screen.getByTestId("confirm-dialog")).toBeInTheDocument(),
    );
    fireEvent.click(screen.getByText("Confirm disconnect"));

    await waitFor(() =>
      expect(screen.getByText("Disconnect failed")).toBeInTheDocument(),
    );
  });
});