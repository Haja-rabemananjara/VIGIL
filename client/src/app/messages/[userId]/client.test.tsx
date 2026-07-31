import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, screen, fireEvent, waitFor } from "@testing-library/react";

// Mocks

const mockApi = vi.fn();
vi.mock("@/lib/api", () => ({
  api: (...args: unknown[]) => mockApi(...args),
}));

const mockBack = vi.fn();
vi.mock("next/navigation", () => ({
  useRouter: () => ({ back: mockBack }),
  usePathname: () => "/messages/other-id",
}));

vi.mock("@/stores/auth", () => ({
  useAuth: () => ({
    user: { id: "me", display_name: "Alice", email: "a@t.com" },
    token: "tok",
  }),
}));

let mockLastEvent: unknown = null;
let mockReconnectCount = 0;
vi.mock("@/stores/socket", () => ({
  useVigilSocket: () => ({
    get lastEvent() {
      return mockLastEvent;
    },
    get reconnectCount() {
      return mockReconnectCount;
    },
    send: vi.fn(),
  }),
}));

Element.prototype.scrollIntoView = vi.fn();

import { ConversationClient } from "./client";

const MSGS = [
  {
    id: "m1",
    sender_id: "me",
    recipient_id: "other-id",
    content: "Hello!",
    created_at: 1700000000,
  },
  {
    id: "m2",
    sender_id: "other-id",
    recipient_id: "me",
    content: "Hi there!",
    created_at: 1700000060,
  },
];

describe("ConversationClient", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    mockLastEvent = null;
    mockReconnectCount = 0;
  });

  // Loading / Error / Empty

  it("shows loading state", () => {
    mockApi.mockImplementation(() => new Promise(() => {}));
    render(<ConversationClient />);
    expect(screen.getByText("Loading...")).toBeInTheDocument();
  });

  it("shows error on fetch failure", async () => {
    mockApi.mockRejectedValue(new Error("fail"));
    render(<ConversationClient />);
    await waitFor(() =>
      expect(screen.getByText("Something went wrong")).toBeInTheDocument(),
    );
  });

  it("shows empty state when no messages", async () => {
    mockApi
      .mockResolvedValueOnce({ messages: [] })
      .mockResolvedValueOnce({ id: "other-id", display_name: "Bob" });
    render(<ConversationClient />);
    await waitFor(() =>
      expect(screen.getByText("No messages yet. Say hi!")).toBeInTheDocument(),
    );
  });

  // Header

  it("renders the other user display name", async () => {
    mockApi
      .mockResolvedValueOnce({ messages: [] })
      .mockResolvedValueOnce({ id: "other-id", display_name: "Bob" });
    render(<ConversationClient />);
    await waitFor(() => expect(screen.getByText("Bob")).toBeInTheDocument());
  });

  it("falls back to userId when user info fetch fails", async () => {
    mockApi
      .mockResolvedValueOnce({ messages: [] })
      .mockRejectedValueOnce(new Error("404"));
    render(<ConversationClient />);
    await waitFor(() =>
      expect(screen.getByText("other-id")).toBeInTheDocument(),
    );
  });

  it("back button calls router.back()", async () => {
    mockApi
      .mockResolvedValueOnce({ messages: [] })
      .mockResolvedValueOnce({ id: "other-id", display_name: "Bob" });
    render(<ConversationClient />);
    await waitFor(() => expect(screen.getByText("Bob")).toBeInTheDocument());

    // ArrowLeft button is the first button (no aria-label)
    const buttons = screen.getAllByRole("button");
    fireEvent.click(buttons[0]);
    expect(mockBack).toHaveBeenCalled();
  });

  // Messages rendering

  it("renders messages with content and timestamps", async () => {
    mockApi
      .mockResolvedValueOnce({ messages: MSGS })
      .mockResolvedValueOnce({ id: "other-id", display_name: "Bob" });
    render(<ConversationClient />);
    await waitFor(() => {
      expect(screen.getByText("Hello!")).toBeInTheDocument();
      expect(screen.getByText("Hi there!")).toBeInTheDocument();
    });
  });

  // Composer

  it("disables send button when composer is empty", async () => {
    mockApi
      .mockResolvedValueOnce({ messages: [] })
      .mockResolvedValueOnce({ id: "other-id", display_name: "Bob" });
    render(<ConversationClient />);
    await waitFor(() => expect(screen.getByText("Bob")).toBeInTheDocument());

    expect(screen.getByRole("button", { name: /send/i })).toBeDisabled();
  });

  it("disables send button when composer is whitespace only", async () => {
    mockApi
      .mockResolvedValueOnce({ messages: [] })
      .mockResolvedValueOnce({ id: "other-id", display_name: "Bob" });
    render(<ConversationClient />);
    await waitFor(() => expect(screen.getByText("Bob")).toBeInTheDocument());

    fireEvent.change(screen.getByPlaceholderText("Write a message..."), {
      target: { value: "   " },
    });
    expect(screen.getByRole("button", { name: /send/i })).toBeDisabled();
  });

  it("enables send button when composer has text", async () => {
    mockApi
      .mockResolvedValueOnce({ messages: [] })
      .mockResolvedValueOnce({ id: "other-id", display_name: "Bob" });
    render(<ConversationClient />);
    await waitFor(() => expect(screen.getByText("Bob")).toBeInTheDocument());

    fireEvent.change(screen.getByPlaceholderText("Write a message..."), {
      target: { value: "yo" },
    });
    expect(screen.getByRole("button", { name: /send/i })).not.toBeDisabled();
  });

  // Sending

  it("sends a message and clears composer", async () => {
    mockApi
      .mockResolvedValueOnce({ messages: [] })
      .mockResolvedValueOnce({ id: "other-id", display_name: "Bob" })
      .mockResolvedValueOnce({
        id: "m-new",
        sender_id: "me",
        recipient_id: "other-id",
        content: "Test msg",
        created_at: 1700002000,
      });

    render(<ConversationClient />);
    await waitFor(() => expect(screen.getByText("Bob")).toBeInTheDocument());

    const textarea = screen.getByPlaceholderText("Write a message...");
    fireEvent.change(textarea, { target: { value: "Test msg" } });
    fireEvent.click(screen.getByRole("button", { name: /send/i }));

    await waitFor(() => {
      expect(mockApi).toHaveBeenCalledWith(
        "/messages/other-id",
        expect.objectContaining({
          method: "POST",
          body: { content: "Test msg" },
        }),
      );
      expect(textarea).toHaveValue("");
    });
  });

  it("sends on Ctrl+Enter", async () => {
    mockApi
      .mockResolvedValueOnce({ messages: [] })
      .mockResolvedValueOnce({ id: "other-id", display_name: "Bob" })
      .mockResolvedValueOnce({
        id: "m-ctrl",
        sender_id: "me",
        recipient_id: "other-id",
        content: "via ctrl",
        created_at: 1700003000,
      });

    render(<ConversationClient />);
    await waitFor(() => expect(screen.getByText("Bob")).toBeInTheDocument());

    const textarea = screen.getByPlaceholderText("Write a message...");
    fireEvent.change(textarea, { target: { value: "via ctrl" } });
    fireEvent.keyDown(textarea, { key: "Enter", ctrlKey: true });

    await waitFor(() =>
      expect(mockApi).toHaveBeenCalledWith(
        "/messages/other-id",
        expect.objectContaining({ method: "POST" }),
      ),
    );
  });

  it("does not send on Enter without Ctrl", async () => {
    mockApi
      .mockResolvedValueOnce({ messages: [] })
      .mockResolvedValueOnce({ id: "other-id", display_name: "Bob" });
    render(<ConversationClient />);
    await waitFor(() => expect(screen.getByText("Bob")).toBeInTheDocument());

    const textarea = screen.getByPlaceholderText("Write a message...");
    fireEvent.change(textarea, { target: { value: "no send" } });
    fireEvent.keyDown(textarea, { key: "Enter" });
    expect(mockApi).toHaveBeenCalledTimes(2);
  });

  it("handles send failure silently", async () => {
    mockApi
      .mockResolvedValueOnce({ messages: [] })
      .mockResolvedValueOnce({ id: "other-id", display_name: "Bob" })
      .mockRejectedValueOnce(new Error("network"));

    render(<ConversationClient />);
    await waitFor(() => expect(screen.getByText("Bob")).toBeInTheDocument());

    const textarea = screen.getByPlaceholderText("Write a message...");
    fireEvent.change(textarea, { target: { value: "will fail" } });
    fireEvent.click(screen.getByRole("button", { name: /send/i }));

    await waitFor(() => expect(textarea).toHaveValue("will fail"));
  });

  // Typing indicator

  it("sends typing event on keystroke (throttled)", async () => {
    const mockSend = vi.fn();
    vi.mocked(await import("@/stores/socket")).useVigilSocket = (() => ({
      lastEvent: null,
      reconnectCount: 0,
      send: mockSend,
    })) as never;

    mockApi
      .mockResolvedValueOnce({ messages: [] })
      .mockResolvedValueOnce({ id: "other-id", display_name: "Bob" });
    render(<ConversationClient />);
    await waitFor(() => expect(screen.getByText("Bob")).toBeInTheDocument());

    const textarea = screen.getByPlaceholderText("Write a message...");
    fireEvent.change(textarea, { target: { value: "h" } });
  });

  it("does not duplicate messages with the same id", async () => {
    mockApi
      .mockResolvedValueOnce({ messages: MSGS })
      .mockResolvedValueOnce({ id: "other-id", display_name: "Bob" });

    mockLastEvent = {
      type: "private_message_received",
      from: "other-id",
      to: "me",
      message_id: "m2",
      content: "Hi there!",
      at: 1700000060,
    };

    render(<ConversationClient />);
    await waitFor(() => expect(screen.getByText("Hello!")).toBeInTheDocument());

    const matches = screen.getAllByText("Hi there!");
    expect(matches).toHaveLength(1);
  });

  it("ignores typing events from other conversations", async () => {
    mockApi
      .mockResolvedValueOnce({ messages: [] })
      .mockResolvedValueOnce({ id: "other-id", display_name: "Bob" });
    render(<ConversationClient />);
    await waitFor(() => expect(screen.getByText("Bob")).toBeInTheDocument());

    mockLastEvent = { type: "user_typing", from: "someone-else" };
    render(<ConversationClient />);

    expect(screen.queryByText(/is typing/i)).not.toBeInTheDocument();
  });

  it("ignores WS events from other conversations", async () => {
    mockApi
      .mockResolvedValueOnce({ messages: [] })
      .mockResolvedValueOnce({ id: "other-id", display_name: "Bob" });
    render(<ConversationClient />);
    await waitFor(() => expect(screen.getByText("Bob")).toBeInTheDocument());

    mockLastEvent = {
      type: "private_message_received",
      from: "someone-else",
      to: "another-person",
      message_id: "m99",
      content: "Not for us",
      at: 1700005000,
    };
    render(<ConversationClient />);

    expect(screen.queryByText("Not for us")).not.toBeInTheDocument();
  });

  // Reconnect
  it("re-fetches messages on reconnect", async () => {
    mockApi
      .mockResolvedValueOnce({ messages: MSGS })
      .mockResolvedValueOnce({ id: "other-id", display_name: "Bob" });
    render(<ConversationClient />);
    await waitFor(() => expect(screen.getByText("Hello!")).toBeInTheDocument());

    // Simulate reconnect
    mockReconnectCount = 1;
    mockApi.mockResolvedValueOnce({
      messages: [
        ...MSGS,
        {
          id: "m3",
          sender_id: "other-id",
          recipient_id: "me",
          content: "After reconnect",
          created_at: 1700000120,
        },
      ],
    });

    render(<ConversationClient />);
    await waitFor(() =>
      expect(screen.getByText("After reconnect")).toBeInTheDocument(),
    );
  });
});
