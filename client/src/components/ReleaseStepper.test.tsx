import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, screen, fireEvent } from "@testing-library/react";
import { ReleaseStepper, type ReleaseStep } from "./ReleaseStepper";

const makeStep = (overrides: Partial<ReleaseStep>): ReleaseStep => ({
  id: "step-1",
  name: "build",
  position: 0,
  validated_by: null,
  validated_at: null,
  ...overrides,
});

const threeSteps: ReleaseStep[] = [
  makeStep({ id: "s1", name: "build", position: 0 }),
  makeStep({ id: "s2", name: "staging", position: 1 }),
  makeStep({ id: "s3", name: "production", position: 2 }),
];

describe("ReleaseStepper", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it("renders all step names", () => {
    render(<ReleaseStepper steps={threeSteps} releaseStatus="created" />);
    expect(screen.getByText("build")).toBeInTheDocument();
    expect(screen.getByText("staging")).toBeInTheDocument();
    expect(screen.getByText("production")).toBeInTheDocument();
  });

  it("renders nothing when steps list is empty", () => {
    const { container } = render(
      <ReleaseStepper steps={[]} releaseStatus="created" />,
    );
    expect(container.textContent).toBe("");
  });

  it("shows validation timestamp for validated steps", () => {
    const validated = makeStep({
      validated_by: "user-1",
      validated_at: 1718000000,
    });
    render(<ReleaseStepper steps={[validated]} releaseStatus="in_progress" />);
    expect(screen.getByText(/validated/i)).toBeInTheDocument();
  });

  it("shows the validate button on the current step when release is in_progress and callback provided", () => {
    const onValidate = vi.fn();
    render(
      <ReleaseStepper
        steps={threeSteps}
        releaseStatus="in_progress"
        onValidate={onValidate}
      />,
    );
    expect(
      screen.getByRole("button", { name: /validate/i }),
    ).toBeInTheDocument();
  });

  it("does not show the validate button when release is created", () => {
    const onValidate = vi.fn();
    render(
      <ReleaseStepper
        steps={threeSteps}
        releaseStatus="created"
        onValidate={onValidate}
      />,
    );
    expect(
      screen.queryByRole("button", { name: /validate/i }),
    ).not.toBeInTheDocument();
  });

  it("does not show the validate button when release is blocked", () => {
    const onValidate = vi.fn();
    render(
      <ReleaseStepper
        steps={threeSteps}
        releaseStatus="blocked"
        onValidate={onValidate}
      />,
    );
    expect(
      screen.queryByRole("button", { name: /validate/i }),
    ).not.toBeInTheDocument();
  });

  it("does not show the validate button when release is completed", () => {
    const onValidate = vi.fn();
    render(
      <ReleaseStepper
        steps={threeSteps}
        releaseStatus="completed"
        onValidate={onValidate}
      />,
    );
    expect(
      screen.queryByRole("button", { name: /validate/i }),
    ).not.toBeInTheDocument();
  });

  it("does not show the validate button when onValidate is not provided (read-only)", () => {
    render(<ReleaseStepper steps={threeSteps} releaseStatus="in_progress" />);
    expect(
      screen.queryByRole("button", { name: /validate/i }),
    ).not.toBeInTheDocument();
  });

  it("calls onValidate with the current step id when clicked", () => {
    const onValidate = vi.fn();
    render(
      <ReleaseStepper
        steps={threeSteps}
        releaseStatus="in_progress"
        onValidate={onValidate}
      />,
    );
    fireEvent.click(screen.getByRole("button", { name: /validate/i }));
    expect(onValidate).toHaveBeenCalledWith("s1");
  });

  it("disables the validate button while validating", () => {
    const onValidate = vi.fn();
    render(
      <ReleaseStepper
        steps={threeSteps}
        releaseStatus="in_progress"
        onValidate={onValidate}
        validating
      />,
    );
    expect(screen.getByRole("button")).toBeDisabled();
  });

  it("marks the first unvalidated step as current (skips validated ones)", () => {
    const stepsPartiallyDone: ReleaseStep[] = [
      makeStep({
        id: "s1",
        name: "build",
        position: 0,
        validated_by: "user-1",
        validated_at: 100,
      }),
      makeStep({ id: "s2", name: "staging", position: 1 }),
      makeStep({ id: "s3", name: "production", position: 2 }),
    ];
    const onValidate = vi.fn();
    render(
      <ReleaseStepper
        steps={stepsPartiallyDone}
        releaseStatus="in_progress"
        onValidate={onValidate}
      />,
    );
    fireEvent.click(screen.getByRole("button", { name: /validate/i }));
    expect(onValidate).toHaveBeenCalledWith("s2");
  });

  it("does not render a validate button when all steps are validated", () => {
    const allDone: ReleaseStep[] = threeSteps.map((s, i) => ({
      ...s,
      validated_by: "user-1",
      validated_at: 100 + i,
    }));
    const onValidate = vi.fn();
    render(
      <ReleaseStepper
        steps={allDone}
        releaseStatus="in_progress"
        onValidate={onValidate}
      />,
    );
    expect(
      screen.queryByRole("button", { name: /validate/i }),
    ).not.toBeInTheDocument();
  });

  it("uses distinct visual signals for validated/current/pending states", () => {
    const mixed: ReleaseStep[] = [
      makeStep({
        id: "s1",
        name: "build",
        position: 0,
        validated_by: "user-1",
        validated_at: 100,
      }),
      makeStep({ id: "s2", name: "staging", position: 1 }),
      makeStep({ id: "s3", name: "production", position: 2 }),
    ];
    const { container } = render(
      <ReleaseStepper steps={mixed} releaseStatus="in_progress" />,
    );
    const html = container.innerHTML;
    expect(html).toMatch(/success|primary/);
    expect(html).toMatch(/muted-foreground/);
  });
});
