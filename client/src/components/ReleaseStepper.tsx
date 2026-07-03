import { Check, Circle, Lock } from "lucide-react";
import { cn } from "@/lib/utils";
import { t } from "@/lib/i18n";

export interface ReleaseStep {
  id: string;
  name: string;
  position: number;
  validated_by: string | null;
  validated_at: number | null;
}

interface ReleaseStepperProps {
  steps: ReleaseStep[];
  releaseStatus: string;
  /** Called when the user clicks "Validate" on a step. Undefined = read-only. */
  onValidate?: (stepId: string) => void;
  /** Is a validation request currently in flight? */
  validating?: boolean;
}

/**
 * Sequential stepper for release steps.
 *
 * Each step is one of:
 *   - validated (green check)
 *   - current (blue dot, next to validate)
 *   - pending (gray, locked)
 *
 * The "current" step is the first unvalidated one.
 * When the release is blocked, all unvalidated steps show a lock.
 */
export function ReleaseStepper({
  steps,
  releaseStatus,
  onValidate,
  validating = false,
}: ReleaseStepperProps) {
  const isBlocked = releaseStatus === "blocked";
  const isActive = releaseStatus === "in_progress";

  // Find the first unvalidated step (the "current" one)
  const currentPosition = steps.find((s) => !s.validated_by)?.position ?? null;

  return (
    <div className="space-y-1">
      {steps.map((step, idx) => {
        const isValidated = !!step.validated_by;
        const isCurrent = step.position === currentPosition;
        const isLast = idx === steps.length - 1;

        return (
          <div key={step.id} className="flex items-start gap-3">
            {/* Step indicator + connector line */}
            <div className="flex flex-col items-center">
              <StepIcon
                validated={isValidated}
                current={isCurrent}
                blocked={isBlocked}
              />
              {!isLast && (
                <div
                  className={cn(
                    "w-0.5 h-8",
                    isValidated ? "bg-success" : "bg-border",
                  )}
                />
              )}
            </div>

            {/* Step content */}
            <div className="flex flex-1 items-center justify-between pb-8 last:pb-0">
              <div>
                <p
                  className={cn(
                    "text-sm font-medium",
                    isValidated && "text-success",
                    isCurrent && !isBlocked && "text-primary",
                    !isValidated && !isCurrent && "text-muted-foreground",
                  )}
                >
                  {step.name}
                </p>
                {isValidated && step.validated_at && (
                  <p className="text-xs text-muted-foreground">
                    {t("release.step.validatedAt")}{" "}
                    {new Date(step.validated_at * 1000).toLocaleString()}
                  </p>
                )}
              </div>

              {/* Validate button: only on current step, only if in_progress, only if callback provided */}
              {isCurrent && isActive && onValidate && (
                <button
                  onClick={() => onValidate(step.id)}
                  disabled={validating}
                  className={cn(
                    "rounded-md px-3 py-1.5 text-xs font-medium",
                    "bg-primary text-primary-foreground hover:bg-primary/90",
                    "disabled:opacity-50 disabled:cursor-not-allowed",
                  )}
                >
                  {validating ? "…" : t("release.step.validate")}
                </button>
              )}
            </div>
          </div>
        );
      })}
    </div>
  );
}

function StepIcon({
  validated,
  current,
  blocked,
}: {
  validated: boolean;
  current: boolean;
  blocked: boolean;
}) {
  if (validated) {
    return (
      <div className="flex h-6 w-6 items-center justify-center rounded-full bg-success text-success-foreground">
        <Check className="h-3.5 w-3.5" />
      </div>
    );
  }

  if (blocked) {
    return (
      <div className="flex h-6 w-6 items-center justify-center rounded-full bg-destructive/10 text-destructive">
        <Lock className="h-3.5 w-3.5" />
      </div>
    );
  }

  if (current) {
    return (
      <div className="flex h-6 w-6 items-center justify-center rounded-full bg-primary/10 text-primary">
        <Circle className="h-3 w-3 fill-current" />
      </div>
    );
  }

  // Pending
  return (
    <div className="flex h-6 w-6 items-center justify-center rounded-full bg-muted text-muted-foreground">
      <Circle className="h-3 w-3" />
    </div>
  );
}
