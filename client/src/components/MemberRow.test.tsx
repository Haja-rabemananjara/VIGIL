import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, screen, fireEvent } from "@testing-library/react";
import { MemberRow, type MemberView } from "./MemberRow";

// Mock next/navigation
const mockPush = vi.fn();
vi.mock("next/navigation", () => ({
  useRouter: () => ({ push: mockPush }),
}));

const baseMember: MemberView = {
  user_id: "user-1",
  display_name: "Alice",
  email: "alice@example.com",
  role: "observer",
  joined_at: "2026-01-01T00:00:00Z",
};

describe("MemberRow", () => {
  const defaultHandlers = {
    onPromote: vi.fn(),
    onDemote: vi.fn(),
    onTransfer: vi.fn(),
    onKick: vi.fn(),
    onBan: vi.fn(),
  };

  beforeEach(() => {
    vi.clearAllMocks();
  });

  it("displays the member's name", () => {
    render(
      <MemberRow
        member={baseMember}
        isMe={false}
        isManager={false}
        {...defaultHandlers}
      />,
    );
    expect(screen.getByText("Alice")).toBeInTheDocument();
  });

  it("shows the '(you)' label when isMe is true", () => {
    render(
      <MemberRow
        member={baseMember}
        isMe
        isManager={false}
        {...defaultHandlers}
      />,
    );
    expect(screen.getByText(/you/i)).toBeInTheDocument();
  });

  it("does not show the message button on self", () => {
    render(
      <MemberRow
        member={baseMember}
        isMe
        isManager={false}
        {...defaultHandlers}
      />,
    );
    expect(screen.queryByTitle(/message/i)).not.toBeInTheDocument();
  });

  it("shows the message button on other members", () => {
    render(
      <MemberRow
        member={baseMember}
        isMe={false}
        isManager={false}
        {...defaultHandlers}
      />,
    );
    expect(screen.getByTitle(/message/i)).toBeInTheDocument();
  });

  it("navigates to the DM page when the message button is clicked", () => {
    render(
      <MemberRow
        member={baseMember}
        isMe={false}
        isManager={false}
        {...defaultHandlers}
      />,
    );
    fireEvent.click(screen.getByTitle(/message/i));
    expect(mockPush).toHaveBeenCalledWith("/messages/user-1");
  });

  it("shows no manager actions when isManager is false", () => {
    render(
      <MemberRow
        member={baseMember}
        isMe={false}
        isManager={false}
        {...defaultHandlers}
      />,
    );
    expect(
      screen.queryByRole("button", { name: /promote/i }),
    ).not.toBeInTheDocument();
    expect(
      screen.queryByRole("button", { name: /kick/i }),
    ).not.toBeInTheDocument();
    expect(
      screen.queryByRole("button", { name: /ban/i }),
    ).not.toBeInTheDocument();
  });

  it("shows no manager actions on self even when isManager is true", () => {
    render(
      <MemberRow member={baseMember} isMe isManager {...defaultHandlers} />,
    );
    expect(
      screen.queryByRole("button", { name: /promote/i }),
    ).not.toBeInTheDocument();
    expect(
      screen.queryByRole("button", { name: /kick/i }),
    ).not.toBeInTheDocument();
  });

  it("shows no manager actions on other managers", () => {
    render(
      <MemberRow
        member={{ ...baseMember, role: "manager" }}
        isMe={false}
        isManager
        {...defaultHandlers}
      />,
    );
    expect(
      screen.queryByRole("button", { name: /promote/i }),
    ).not.toBeInTheDocument();
    expect(
      screen.queryByRole("button", { name: /demote/i }),
    ).not.toBeInTheDocument();
  });

  it("shows promote button on observers when manager", () => {
    render(
      <MemberRow
        member={baseMember}
        isMe={false}
        isManager
        {...defaultHandlers}
      />,
    );
    expect(
      screen.getByRole("button", { name: /promote/i }),
    ).toBeInTheDocument();
  });

  it("shows demote button on responders when manager", () => {
    render(
      <MemberRow
        member={{ ...baseMember, role: "responder" }}
        isMe={false}
        isManager
        {...defaultHandlers}
      />,
    );
    expect(screen.getByRole("button", { name: /demote/i })).toBeInTheDocument();
  });

  it("calls onPromote when promote is clicked", () => {
    const onPromote = vi.fn();
    render(
      <MemberRow
        member={baseMember}
        isMe={false}
        isManager
        {...defaultHandlers}
        onPromote={onPromote}
      />,
    );
    fireEvent.click(screen.getByRole("button", { name: /promote/i }));
    expect(onPromote).toHaveBeenCalledWith("user-1");
  });

  it("calls onDemote when demote is clicked", () => {
    const onDemote = vi.fn();
    render(
      <MemberRow
        member={{ ...baseMember, role: "responder" }}
        isMe={false}
        isManager
        {...defaultHandlers}
        onDemote={onDemote}
      />,
    );
    fireEvent.click(screen.getByRole("button", { name: /demote/i }));
    expect(onDemote).toHaveBeenCalledWith("user-1");
  });

  it("calls onTransfer with the member when transfer is clicked", () => {
    const onTransfer = vi.fn();
    render(
      <MemberRow
        member={baseMember}
        isMe={false}
        isManager
        {...defaultHandlers}
        onTransfer={onTransfer}
      />,
    );
    fireEvent.click(screen.getByRole("button", { name: /transfer/i }));
    expect(onTransfer).toHaveBeenCalledWith(baseMember);
  });

  it("calls onKick with the member when kick is clicked", () => {
    const onKick = vi.fn();
    render(
      <MemberRow
        member={baseMember}
        isMe={false}
        isManager
        {...defaultHandlers}
        onKick={onKick}
      />,
    );
    fireEvent.click(screen.getByRole("button", { name: /remove from team/i }));
    expect(onKick).toHaveBeenCalledWith(baseMember);
  });

  it("calls onBan with the member when ban is clicked", () => {
    const onBan = vi.fn();
    render(
      <MemberRow
        member={baseMember}
        isMe={false}
        isManager
        {...defaultHandlers}
        onBan={onBan}
      />,
    );
    fireEvent.click(screen.getByRole("button", { name: /^ban$/i }));
    expect(onBan).toHaveBeenCalledWith(baseMember);
  });
});
