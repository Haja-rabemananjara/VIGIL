import { describe, it, expect } from "vitest";
import { render, screen } from "@testing-library/react";
import { SeverityBadge, type Severity } from "./SeverityBadge";

describe("SeverityBadge", () => {
  const severities: Severity[] = ["low", "medium", "high", "critical"];

  it.each(severities)("renders text label for severity '%s'", (severity) => {
    render(<SeverityBadge severity={severity} />);
    expect(screen.getByText(/.+/)).toBeInTheDocument();
  });

  it.each(severities)("renders an icon for severity '%s'", (severity) => {
    const { container } = render(<SeverityBadge severity={severity} />);
    expect(container.querySelector("svg")).toBeInTheDocument();
  });

  it("renders distinct labels per severity", () => {
    const labels = severities.map((s) => {
      const { container } = render(<SeverityBadge severity={s} />);
      return container.textContent;
    });
    expect(new Set(labels).size).toBe(severities.length);
  });
});
