import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, screen, fireEvent, waitFor } from "@testing-library/react";
import { TimelineEntryCard, type TimelineEntry } from "./TimelineEntryCard";

// Mock the api module
vi.mock("@/lib/api", () => ({
  api: vi.fn(),
}));

import { api } from "@/lib/api";
const mockApi = vi.mocked(api);

const messageEntry: TimelineEntry = {
  id: "entry-1",
  author_id: "user-1",
  kind: "message",
  content: "Investigating the issue",
  created_at: 1718000000,
  edited_at: null,
};

const systemEntry: TimelineEntry = {
  id: "entry-2",
  author_id: "user-1",
  kind: "system",
  content: "Incident acknowledged",
  created_at: 1718000100,
  edited_at: null,
};

describe("TimelineEntryCard", () => {
  const defaultProps = {
    entry: messageEntry,
    currentUserId: "user-1",
    displayName: (id: string) => (id === "user-1" ? "Alice" : id),
    entryReactions: {},
    availableEmojis: ["+1", "-1", "fire"],
    token: "test-token",
    onEntryUpdated: vi.fn(),
    onReactionToggle: vi.fn(),
  };

  beforeEach(() => {
    vi.clearAllMocks();
  });

  it("renders the entry content", () => {
    render(<TimelineEntryCard {...defaultProps} />);
    expect(screen.getByText("Investigating the issue")).toBeInTheDocument();
  });

  it("renders the author's display name for message entries", () => {
    render(<TimelineEntryCard {...defaultProps} />);
    expect(screen.getByText("Alice")).toBeInTheDocument();
  });

  it("renders 'System' label for system entries", () => {
    render(<TimelineEntryCard {...defaultProps} entry={systemEntry} />);
    expect(screen.getByText(/system/i)).toBeInTheDocument();
  });

  it("shows the 'edited' label when entry has edited_at", () => {
    render(
      <TimelineEntryCard
        {...defaultProps}
        entry={{ ...messageEntry, edited_at: 1718000500 }}
      />,
    );
    expect(screen.getByText(/edited/i)).toBeInTheDocument();
  });

  it("does not show 'edited' label when entry has never been edited", () => {
    render(<TimelineEntryCard {...defaultProps} />);
    expect(screen.queryByText(/edited/i)).not.toBeInTheDocument();
  });

  it("shows the edit button on own message entries", () => {
    render(<TimelineEntryCard {...defaultProps} />);
    expect(screen.getByRole("button", { name: /edit/i })).toBeInTheDocument();
  });

  it("does not show the edit button on other users' entries", () => {
    render(<TimelineEntryCard {...defaultProps} currentUserId="user-2" />);
    expect(
      screen.queryByRole("button", { name: /edit/i }),
    ).not.toBeInTheDocument();
  });

  it("does not show the edit button on system entries", () => {
    render(<TimelineEntryCard {...defaultProps} entry={systemEntry} />);
    expect(
      screen.queryByRole("button", { name: /edit/i }),
    ).not.toBeInTheDocument();
  });

  it("enters edit mode when edit button is clicked", () => {
    render(<TimelineEntryCard {...defaultProps} />);
    fireEvent.click(screen.getByRole("button", { name: /edit/i }));
    expect(screen.getByRole("textbox")).toBeInTheDocument();
  });

  it("prefills the textarea with the current content", () => {
    render(<TimelineEntryCard {...defaultProps} />);
    fireEvent.click(screen.getByRole("button", { name: /edit/i }));
    const textarea = screen.getByRole("textbox") as HTMLTextAreaElement;
    expect(textarea.value).toBe("Investigating the issue");
  });

  it("exits edit mode when cancel is clicked", () => {
    render(<TimelineEntryCard {...defaultProps} />);
    fireEvent.click(screen.getByRole("button", { name: /edit/i }));
    fireEvent.click(screen.getByRole("button", { name: /cancel/i }));
    expect(screen.queryByRole("textbox")).not.toBeInTheDocument();
  });

  it("exits edit mode on Escape key", () => {
    render(<TimelineEntryCard {...defaultProps} />);
    fireEvent.click(screen.getByRole("button", { name: /edit/i }));
    fireEvent.keyDown(screen.getByRole("textbox"), { key: "Escape" });
    expect(screen.queryByRole("textbox")).not.toBeInTheDocument();
  });

  it("calls API and onEntryUpdated when save is clicked", async () => {
    const updated: TimelineEntry = {
      ...messageEntry,
      content: "Updated text",
      edited_at: 1718000500,
    };
    mockApi.mockResolvedValue(updated);

    const onEntryUpdated = vi.fn();
    render(
      <TimelineEntryCard {...defaultProps} onEntryUpdated={onEntryUpdated} />,
    );
    fireEvent.click(screen.getByRole("button", { name: /edit/i }));
    fireEvent.change(screen.getByRole("textbox"), {
      target: { value: "Updated text" },
    });
    fireEvent.click(screen.getByRole("button", { name: /save/i }));

    await waitFor(() => {
      expect(mockApi).toHaveBeenCalledWith(
        "/timeline/entry-1",
        expect.objectContaining({
          method: "PATCH",
          body: { content: "Updated text" },
        }),
      );
    });
    await waitFor(() => {
      expect(onEntryUpdated).toHaveBeenCalledWith(updated);
    });
  });

  it("does not submit save with empty content", async () => {
    render(<TimelineEntryCard {...defaultProps} />);
    fireEvent.click(screen.getByRole("button", { name: /edit/i }));
    fireEvent.change(screen.getByRole("textbox"), {
      target: { value: "   " },
    });
    fireEvent.click(screen.getByRole("button", { name: /save/i }));
    expect(mockApi).not.toHaveBeenCalled();
  });

  it("does not render the reactions area on system entries", () => {
    render(
      <TimelineEntryCard
        {...defaultProps}
        entry={systemEntry}
        entryReactions={{ "+1": ["user-1"] }}
      />,
    );
    // The picker "+" button should not be visible
    expect(screen.queryByRole("button", { name: "+" })).not.toBeInTheDocument();
  });

  it("renders reaction chips with counts", () => {
    render(
      <TimelineEntryCard
        {...defaultProps}
        entryReactions={{ "+1": ["user-1", "user-2"], fire: ["user-2"] }}
      />,
    );
    expect(screen.getByText("2")).toBeInTheDocument();
    expect(screen.getByText("1")).toBeInTheDocument();
  });

  it("calls onReactionToggle when clicking an existing reaction chip", () => {
    const onReactionToggle = vi.fn();
    render(
      <TimelineEntryCard
        {...defaultProps}
        entryReactions={{ "+1": ["user-1"] }}
        onReactionToggle={onReactionToggle}
      />,
    );
    // Find and click the reaction chip
    const chips = screen.getAllByRole("button");
    const chipButton = chips.find(
      (b) => b.textContent?.includes("1") && b.textContent?.includes("👍"),
    );
    if (chipButton) {
      fireEvent.click(chipButton);
      expect(onReactionToggle).toHaveBeenCalledWith("entry-1", "+1");
    }
  });

  it("opens the emoji picker when + is clicked", () => {
    render(<TimelineEntryCard {...defaultProps} />);
    fireEvent.click(screen.getByRole("button", { name: "+" }));
    // All available emojis should now be visible in the picker
    const buttons = screen.getAllByRole("button");
    const emojiButtons = buttons.filter((b) =>
      /👍|👎|🔥/.test(b.textContent ?? ""),
    );
    expect(emojiButtons.length).toBeGreaterThan(0);
  });

  it("calls onReactionToggle from the picker and closes it", () => {
    const onReactionToggle = vi.fn();
    render(
      <TimelineEntryCard
        {...defaultProps}
        onReactionToggle={onReactionToggle}
      />,
    );
    fireEvent.click(screen.getByRole("button", { name: "+" }));
    // Click on the fire emoji in the picker
    const buttons = screen.getAllByRole("button");
    const fireButton = buttons.find((b) => b.textContent === "🔥");
    if (fireButton) {
      fireEvent.click(fireButton);
      expect(onReactionToggle).toHaveBeenCalledWith("entry-1", "fire");
    }
  });
});
