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

vi.mock("@/stores/auth", () => ({
  useAuth: () => ({ token: "tok" }),
}));

import { RuleFormDialog } from "./RuleFormDialog";
import type { Rule } from "./client";

const ABOUT = {
  server: {
    services: [
      {
        name: "github",
        actions: [
          { name: "workflow_run", description: "CI workflow event" },
          { name: "tag_push", description: "New tag pushed" },
        ],
        reactions: [],
      },
      {
        name: "vigil",
        actions: [],
        reactions: [
          {
            name: "vigil_create_incident",
            description: "Create an incident",
            payload_example: '{"title":"auto","severity":"high"}',
          },
        ],
      },
      {
        name: "discord",
        actions: [],
        reactions: [
          {
            name: "discord_message",
            description: "Send a message",
            payload_example: '{"content":"hello"}',
          },
        ],
      },
    ],
  },
};

const EXISTING_RULE: Rule = {
  id: "r1",
  team_id: "t1",
  name: "CI rule",
  enabled: true,
  trigger_service: "github",
  trigger_event: "workflow_run",
  trigger_filters: { conclusion: "failure" },
  reaction_type: "vigil_create_incident",
  reaction_payload: { title: "CI broke" },
  created_by: "me",
  created_at: 1700000000,
  updated_at: 1700000000,
};

const defaultProps = {
  open: true,
  onOpenChange: vi.fn(),
  teamId: "t1",
  rule: null as Rule | null,
  onSaved: vi.fn(),
};

function renderDialog(overrides: Partial<typeof defaultProps> = {}) {
  const props = { ...defaultProps, ...overrides };
  return render(<RuleFormDialog {...props} />);
}

/** Wait for about.json to load*/
async function waitForAbout() {
  await waitFor(() => expect(screen.getByRole("dialog")).toBeInTheDocument());
}

/** Fill minimal valid form: name + service + event + reaction */
async function fillValidForm() {
  fireEvent.change(screen.getByLabelText(/^name$/i), {
    target: { value: "My Rule" },
  });

  const serviceSelect = screen.getByLabelText(/trigger service/i);
  fireEvent.change(serviceSelect, { target: { value: "github" } });

  const eventSelect = screen.getByLabelText(/trigger event/i);
  fireEvent.change(eventSelect, { target: { value: "workflow_run" } });

  const reactionSelect = screen.getByLabelText(/^reaction$/i);
  fireEvent.change(reactionSelect, {
    target: { value: "vigil_create_incident" },
  });
}

describe("RuleFormDialog", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    defaultProps.onOpenChange = vi.fn();
    defaultProps.onSaved = vi.fn();
    mockApi.mockResolvedValue(ABOUT);
  });

  // Rendering

  it("renders 'New rule' title when no rule prop", async () => {
    renderDialog();
    await waitForAbout();
    expect(screen.getByText("New rule")).toBeInTheDocument();
  });

  it("renders 'Edit rule' title when rule prop is set", async () => {
    renderDialog({ rule: EXISTING_RULE });
    await waitForAbout();
    expect(screen.getByText("Edit rule")).toBeInTheDocument();
  });

  it("pre-fills fields when editing", async () => {
    renderDialog({ rule: EXISTING_RULE });
    await waitForAbout();

    expect(screen.getByLabelText(/^name$/i)).toHaveValue("CI rule");
    expect(screen.getByLabelText(/trigger service/i)).toHaveValue("github");
    expect(screen.getByLabelText(/trigger event/i)).toHaveValue("workflow_run");
    expect(screen.getByLabelText(/^reaction$/i)).toHaveValue(
      "vigil_create_incident",
    );
  });

  it("populates service select from about.json", async () => {
    renderDialog();
    await waitForAbout();

    const select = screen.getByLabelText(/trigger service/i);
    const options = select.querySelectorAll("option");
    const values = Array.from(options).map((o) => o.getAttribute("value"));
    expect(values).toContain("github");
  });

  it("populates event select when service is chosen", async () => {
    renderDialog();
    await waitForAbout();

    fireEvent.change(screen.getByLabelText(/trigger service/i), {
      target: { value: "github" },
    });

    const eventSelect = screen.getByLabelText(/trigger event/i);
    const options = eventSelect.querySelectorAll("option");
    const values = Array.from(options).map((o) => o.getAttribute("value"));
    expect(values).toContain("workflow_run");
    expect(values).toContain("tag_push");
  });

  it("disables event select when no service selected", async () => {
    renderDialog();
    await waitForAbout();
    expect(screen.getByLabelText(/trigger event/i)).toBeDisabled();
  });

  it("shows reaction description when selected", async () => {
    renderDialog();
    await waitForAbout();

    fireEvent.change(screen.getByLabelText(/^reaction$/i), {
      target: { value: "vigil_create_incident" },
    });

    await waitFor(() =>
      expect(screen.getByText("Create an incident")).toBeInTheDocument(),
    );
  });

  // Payload auto-fill

  it("prefills payload from payload_example when payload is pristine", async () => {
    renderDialog();
    await waitForAbout();

    fireEvent.change(screen.getByLabelText(/^reaction$/i), {
      target: { value: "vigil_create_incident" },
    });

    const payloadArea = screen.getByLabelText(
      /reaction payload/i,
    ) as HTMLTextAreaElement;
    expect(payloadArea.value).toContain('"title"');
    expect(payloadArea.value).toContain('"severity"');
  });

  it("does not overwrite payload if already edited", async () => {
    renderDialog();
    await waitForAbout();

    const payloadArea = screen.getByLabelText(/reaction payload/i);
    fireEvent.change(payloadArea, {
      target: { value: '{"custom": true}' },
    });

    fireEvent.change(screen.getByLabelText(/^reaction$/i), {
      target: { value: "vigil_create_incident" },
    });

    expect(payloadArea).toHaveValue('{"custom": true}');
  });

  // Validation errors

  it("shows error for empty name", async () => {
    renderDialog();
    await waitForAbout();

    fireEvent.click(screen.getByRole("button", { name: /save/i }));
    await waitFor(() =>
      expect(screen.getByText("Name is required")).toBeInTheDocument(),
    );
  });

  it("shows error for missing trigger", async () => {
    renderDialog();
    await waitForAbout();

    fireEvent.change(screen.getByLabelText(/^name$/i), {
      target: { value: "Test" },
    });
    fireEvent.click(screen.getByRole("button", { name: /save/i }));

    await waitFor(() =>
      expect(screen.getByText("Pick a trigger")).toBeInTheDocument(),
    );
  });

  it("shows error for missing reaction", async () => {
    renderDialog();
    await waitForAbout();

    fireEvent.change(screen.getByLabelText(/^name$/i), {
      target: { value: "Test" },
    });
    fireEvent.change(screen.getByLabelText(/trigger service/i), {
      target: { value: "github" },
    });
    fireEvent.change(screen.getByLabelText(/trigger event/i), {
      target: { value: "workflow_run" },
    });
    fireEvent.click(screen.getByRole("button", { name: /save/i }));

    await waitFor(() =>
      expect(screen.getByText("Pick a reaction")).toBeInTheDocument(),
    );
  });

  it("shows error for invalid JSON filters", async () => {
    renderDialog();
    await waitForAbout();

    await fillValidForm();
    fireEvent.change(screen.getByLabelText(/filters/i), {
      target: { value: "not json" },
    });
    fireEvent.click(screen.getByRole("button", { name: /save/i }));

    await waitFor(() =>
      expect(
        screen.getByText("Filters must be valid JSON"),
      ).toBeInTheDocument(),
    );
  });

  it("shows error for invalid JSON payload", async () => {
    renderDialog();
    await waitForAbout();

    await fillValidForm();
    fireEvent.change(screen.getByLabelText(/reaction payload/i), {
      target: { value: "{bad" },
    });
    fireEvent.click(screen.getByRole("button", { name: /save/i }));

    await waitFor(() =>
      expect(
        screen.getByText("Payload must be valid JSON"),
      ).toBeInTheDocument(),
    );
  });

  // Save (create)

  it("creates a new rule via POST", async () => {
    const savedRule = { ...EXISTING_RULE, id: "new-r", name: "My Rule" };
    mockApi.mockResolvedValueOnce(ABOUT).mockResolvedValueOnce(savedRule);

    renderDialog();
    await waitForAbout();
    await fillValidForm();

    fireEvent.click(screen.getByRole("button", { name: /save/i }));

    await waitFor(() => {
      expect(mockApi).toHaveBeenCalledWith(
        "/teams/t1/rules",
        expect.objectContaining({ method: "POST" }),
      );
      expect(defaultProps.onSaved).toHaveBeenCalledWith(savedRule);
      expect(defaultProps.onOpenChange).toHaveBeenCalledWith(false);
    });
  });

  // Save (edit)

  it("updates an existing rule via PATCH", async () => {
    const updated = { ...EXISTING_RULE, name: "Updated" };
    mockApi.mockResolvedValueOnce(ABOUT).mockResolvedValueOnce(updated);

    renderDialog({ rule: EXISTING_RULE });
    await waitForAbout();

    fireEvent.change(screen.getByLabelText(/^name$/i), {
      target: { value: "Updated" },
    });
    fireEvent.click(screen.getByRole("button", { name: /save/i }));

    await waitFor(() => {
      expect(mockApi).toHaveBeenCalledWith(
        "/teams/t1/rules/r1",
        expect.objectContaining({ method: "PATCH" }),
      );
      expect(defaultProps.onSaved).toHaveBeenCalledWith(updated);
    });
  });

  // Save error

  it("shows API error on save failure", async () => {
    mockApi
      .mockResolvedValueOnce(ABOUT)
      .mockRejectedValueOnce(new ApiError(422, "val", "Server said no"));

    renderDialog();
    await waitForAbout();
    await fillValidForm();
    fireEvent.click(screen.getByRole("button", { name: /save/i }));

    await waitFor(() =>
      expect(screen.getByText("Server said no")).toBeInTheDocument(),
    );
  });

  it("shows generic error on network failure", async () => {
    mockApi
      .mockResolvedValueOnce(ABOUT)
      .mockRejectedValueOnce(new Error("net"));

    renderDialog();
    await waitForAbout();
    await fillValidForm();
    fireEvent.click(screen.getByRole("button", { name: /save/i }));

    await waitFor(() =>
      expect(screen.getByText("Something went wrong")).toBeInTheDocument(),
    );
  });

  // Cancel

  it("calls onOpenChange(false) on cancel", async () => {
    renderDialog();
    await waitForAbout();

    fireEvent.click(screen.getByRole("button", { name: /cancel/i }));
    expect(defaultProps.onOpenChange).toHaveBeenCalledWith(false);
  });

  // Enabled checkbox

  it("toggles the enabled checkbox", async () => {
    renderDialog();
    await waitForAbout();

    const checkbox = screen.getByLabelText(/^enabled$/i) as HTMLInputElement;
    expect(checkbox.checked).toBe(true);
    fireEvent.click(checkbox);
    expect(checkbox.checked).toBe(false);
  });

  // about.json error

  it("shows error if about.json fails to load", async () => {
    mockApi.mockRejectedValueOnce(new Error("fail"));
    renderDialog();
    await waitFor(() =>
      expect(screen.getByText("Something went wrong")).toBeInTheDocument(),
    );
  });

  // Does not render when closed

  it("renders nothing when open is false", () => {
    renderDialog({ open: false });
    expect(screen.queryByRole("dialog")).not.toBeInTheDocument();
  });
});
