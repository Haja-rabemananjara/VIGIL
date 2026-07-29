import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, screen, fireEvent, waitFor } from "@testing-library/react";

// Mocks

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
  }),
}));

let mockLastEvent: unknown = null;
vi.mock("@/stores/socket", () => ({
  useVigilSocket: () => ({
    get lastEvent() {
      return mockLastEvent;
    },
  }),
}));

vi.mock("./RuleFormDialog", () => ({
  RuleFormDialog: (props: { open: boolean; rule: { name: string } | null }) => {
    if (!props.open) return null;
    return (
      <div data-testid="rule-form-dialog">
        {props.rule ? `Editing: ${props.rule.name}` : "New rule form"}
      </div>
    );
  },
}));

// eslint-disable-next-line @typescript-eslint/no-unused-vars
let capturedOnConfirm: (() => void) | null = null;
vi.mock("@/components/ConfirmDialog", () => ({
  ConfirmDialog: (props: {
    open: boolean;
    description: string;
    onConfirm: () => void;
  }) => {
    capturedOnConfirm = props.onConfirm;
    if (!props.open) return null;
    return (
      <div data-testid="confirm-dialog">
        <p>{props.description}</p>
        <button onClick={props.onConfirm}>Confirm delete</button>
      </div>
    );
  },
}));

import { RulesClient } from "./client";

const RULE = {
  id: "r1",
  team_id: "t1",
  name: "CI failure rule",
  enabled: true,
  trigger_service: "github",
  trigger_event: "workflow_run",
  trigger_filters: {},
  reaction_type: "vigil_create_incident",
  reaction_payload: {},
  created_by: "me",
  created_at: 1700000000,
  updated_at: 1700000000,
};

const MEMBERS_MANAGER = [{ user_id: "me", role: "manager" }];
const MEMBERS_OBSERVER = [{ user_id: "me", role: "observer" }];

describe("RulesClient", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    mockLastEvent = null;
    capturedOnConfirm = null;
  });

  // Loading / Error / Empty

  it("shows loading state", () => {
    mockApi.mockImplementation(() => new Promise(() => {}));
    render(<RulesClient />);
    expect(screen.getByText("Loading...")).toBeInTheDocument();
  });

  it("shows error on fetch failure", async () => {
    mockApi.mockRejectedValue(new Error("fail"));
    render(<RulesClient />);
    await waitFor(() =>
      expect(screen.getByText("Something went wrong")).toBeInTheDocument(),
    );
  });

  it("shows empty state when no rules", async () => {
    mockApi.mockResolvedValueOnce([]).mockResolvedValueOnce(MEMBERS_MANAGER);
    render(<RulesClient />);
    await waitFor(() =>
      expect(screen.getByText("No rules yet.")).toBeInTheDocument(),
    );
  });

  // Rules rendering

  it("renders rule name and enabled badge", async () => {
    mockApi
      .mockResolvedValueOnce([RULE])
      .mockResolvedValueOnce(MEMBERS_MANAGER);
    render(<RulesClient />);
    await waitFor(() => {
      expect(screen.getByText("CI failure rule")).toBeInTheDocument();
      expect(screen.getByText("Enabled")).toBeInTheDocument();
    });
  });

  it("shows disabled badge for disabled rule", async () => {
    mockApi
      .mockResolvedValueOnce([{ ...RULE, enabled: false }])
      .mockResolvedValueOnce(MEMBERS_MANAGER);
    render(<RulesClient />);
    await waitFor(() =>
      expect(screen.getByText("CI failure rule")).toBeInTheDocument(),
    );
    expect(screen.getAllByText("Disabled").length).toBeGreaterThanOrEqual(1);
  });

  it("shows trigger and reaction info", async () => {
    mockApi
      .mockResolvedValueOnce([RULE])
      .mockResolvedValueOnce(MEMBERS_MANAGER);
    render(<RulesClient />);
    await waitFor(() => {
      expect(screen.getByText(/github\.workflow_run/)).toBeInTheDocument();
      expect(screen.getByText(/vigil_create_incident/)).toBeInTheDocument();
    });
  });

  // Manager vs Observer

  it("shows 'New rule' button for manager", async () => {
    mockApi.mockResolvedValueOnce([]).mockResolvedValueOnce(MEMBERS_MANAGER);
    render(<RulesClient />);
    await waitFor(() =>
      expect(
        screen.getByRole("button", { name: /new rule/i }),
      ).toBeInTheDocument(),
    );
  });

  it("hides 'New rule' button and shows manager-only message for observer", async () => {
    mockApi.mockResolvedValueOnce([]).mockResolvedValueOnce(MEMBERS_OBSERVER);
    render(<RulesClient />);
    await waitFor(() =>
      expect(screen.getByText(/only the manager/i)).toBeInTheDocument(),
    );
    expect(
      screen.queryByRole("button", { name: /new rule/i }),
    ).not.toBeInTheDocument();
  });

  it("shows edit/toggle/delete buttons for manager", async () => {
    mockApi
      .mockResolvedValueOnce([RULE])
      .mockResolvedValueOnce(MEMBERS_MANAGER);
    render(<RulesClient />);
    await waitFor(() =>
      expect(screen.getByText("CI failure rule")).toBeInTheDocument(),
    );
    expect(
      screen.getByRole("button", { name: /edit rule/i }),
    ).toBeInTheDocument();
    expect(
      screen.getByRole("button", { name: /^delete$/i }),
    ).toBeInTheDocument();
  });

  it("hides action buttons for observer", async () => {
    mockApi
      .mockResolvedValueOnce([RULE])
      .mockResolvedValueOnce(MEMBERS_OBSERVER);
    render(<RulesClient />);
    await waitFor(() =>
      expect(screen.getByText("CI failure rule")).toBeInTheDocument(),
    );
    expect(
      screen.queryByRole("button", { name: /edit rule/i }),
    ).not.toBeInTheDocument();
    expect(
      screen.queryByRole("button", { name: /^delete$/i }),
    ).not.toBeInTheDocument();
  });

  // Toggle

  it("toggles rule enabled to disabled", async () => {
    mockApi
      .mockResolvedValueOnce([RULE])
      .mockResolvedValueOnce(MEMBERS_MANAGER)
      .mockResolvedValueOnce({ ...RULE, enabled: false });

    render(<RulesClient />);
    await waitFor(() =>
      expect(screen.getByText("CI failure rule")).toBeInTheDocument(),
    );
    fireEvent.click(screen.getByRole("button", { name: /^disabled$/i }));

    await waitFor(() =>
      expect(mockApi).toHaveBeenCalledWith(
        "/teams/t1/rules/r1",
        expect.objectContaining({
          method: "PATCH",
          body: { enabled: false },
        }),
      ),
    );
  });

  it("shows error when toggle fails", async () => {
    mockApi
      .mockResolvedValueOnce([RULE])
      .mockResolvedValueOnce(MEMBERS_MANAGER)
      .mockRejectedValueOnce(new ApiError(500, "server", "Toggle failed"));

    render(<RulesClient />);
    await waitFor(() =>
      expect(screen.getByText("CI failure rule")).toBeInTheDocument(),
    );
    fireEvent.click(screen.getByRole("button", { name: /^disabled$/i }));

    await waitFor(() =>
      expect(screen.getByText("Toggle failed")).toBeInTheDocument(),
    );
  });

  // Delete

  it("opens confirm dialog and deletes rule", async () => {
    mockApi
      .mockResolvedValueOnce([RULE])
      .mockResolvedValueOnce(MEMBERS_MANAGER)
      .mockResolvedValueOnce(undefined);

    render(<RulesClient />);
    await waitFor(() =>
      expect(screen.getByText("CI failure rule")).toBeInTheDocument(),
    );

    fireEvent.click(screen.getByRole("button", { name: /^delete$/i }));
    await waitFor(() =>
      expect(screen.getByTestId("confirm-dialog")).toBeInTheDocument(),
    );

    fireEvent.click(screen.getByText("Confirm delete"));

    await waitFor(() => {
      expect(mockApi).toHaveBeenCalledWith(
        "/teams/t1/rules/r1",
        expect.objectContaining({ method: "DELETE" }),
      );
      expect(screen.queryByText("CI failure rule")).not.toBeInTheDocument();
    });
  });

  it("shows error when delete fails", async () => {
    mockApi
      .mockResolvedValueOnce([RULE])
      .mockResolvedValueOnce(MEMBERS_MANAGER)
      .mockRejectedValueOnce(new ApiError(500, "err", "Delete failed"));

    render(<RulesClient />);
    await waitFor(() =>
      expect(screen.getByText("CI failure rule")).toBeInTheDocument(),
    );

    fireEvent.click(screen.getByRole("button", { name: /^delete$/i }));
    await waitFor(() =>
      expect(screen.getByTestId("confirm-dialog")).toBeInTheDocument(),
    );
    fireEvent.click(screen.getByText("Confirm delete"));

    await waitFor(() =>
      expect(screen.getByText("Delete failed")).toBeInTheDocument(),
    );
  });

  // Form dialog

  it("opens form dialog for new rule", async () => {
    mockApi.mockResolvedValueOnce([]).mockResolvedValueOnce(MEMBERS_MANAGER);
    render(<RulesClient />);
    await waitFor(() =>
      expect(
        screen.getByRole("button", { name: /new rule/i }),
      ).toBeInTheDocument(),
    );

    fireEvent.click(screen.getByRole("button", { name: /new rule/i }));
    await waitFor(() =>
      expect(screen.getByTestId("rule-form-dialog")).toBeInTheDocument(),
    );
    expect(screen.getByText("New rule form")).toBeInTheDocument();
  });

  it("opens form dialog for editing a rule", async () => {
    mockApi
      .mockResolvedValueOnce([RULE])
      .mockResolvedValueOnce(MEMBERS_MANAGER);
    render(<RulesClient />);
    await waitFor(() =>
      expect(screen.getByText("CI failure rule")).toBeInTheDocument(),
    );

    fireEvent.click(screen.getByRole("button", { name: /edit rule/i }));
    await waitFor(() =>
      expect(screen.getByTestId("rule-form-dialog")).toBeInTheDocument(),
    );
    expect(screen.getByText(/editing: ci failure rule/i)).toBeInTheDocument();
  });

  // Activity feed

  it("shows empty activity state", async () => {
    mockApi.mockResolvedValueOnce([]).mockResolvedValueOnce(MEMBERS_MANAGER);
    render(<RulesClient />);
    await waitFor(() =>
      expect(screen.getByText("Recent activity")).toBeInTheDocument(),
    );
    expect(screen.getByText(/nothing yet/i)).toBeInTheDocument();
  });
});
