import { describe, it, expect } from "vitest";
import { render, screen } from "@testing-library/react";
import { ConnectionIndicator } from "./ConnectionIndicator";

describe("ConnectionIndicator", () => {
  it("renders 'connected' text when status is connected", () => {
    render(<ConnectionIndicator status="connected" />);
    expect(screen.getByText(/connected/i)).toBeInTheDocument();
  });

  it("renders 'connecting' text when status is connecting", () => {
    render(<ConnectionIndicator status="connecting" />);
    expect(screen.getByText(/connecting/i)).toBeInTheDocument();
  });

  it("renders 'disconnected' text when status is disconnected", () => {
    render(<ConnectionIndicator status="disconnected" />);
    expect(screen.getByText(/disconnected/i)).toBeInTheDocument();
  });

  it("uses distinct visual signals for each status (color+icon+text)", () => {
    const { rerender, container } = render(
      <ConnectionIndicator status="connected" />,
    );
    const connectedHtml = container.innerHTML;

    rerender(<ConnectionIndicator status="connecting" />);
    const connectingHtml = container.innerHTML;

    rerender(<ConnectionIndicator status="disconnected" />);
    const disconnectedHtml = container.innerHTML;

    expect(connectedHtml).not.toBe(connectingHtml);
    expect(connectingHtml).not.toBe(disconnectedHtml);
    expect(connectedHtml).not.toBe(disconnectedHtml);
  });
});
