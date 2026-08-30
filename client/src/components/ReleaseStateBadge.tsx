import {
  PackagePlus,
  Play,
  CheckCircle2,
  XCircle,
  ShieldAlert,
  type LucideIcon,
} from "lucide-react";
import { cn } from "@/lib/utils";
import { t } from "@/lib/i18n";
import { useAuth } from "@/stores/auth";

export type ReleaseState =
  "created" | "in_progress" | "completed" | "cancelled" | "blocked";

interface StateConfig {
  icon: LucideIcon;
  label: string;
  className: string;
}

interface ReleaseStateBadgeProps {
  state: ReleaseState;
  className?: string;
}

export function ReleaseStateBadge({
  state,
  className,
}: ReleaseStateBadgeProps) {
  const { language } = useAuth();
  void language;

  const config: Record<ReleaseState, StateConfig> = {
    created: {
      icon: PackagePlus,
      label: t("release.state.created"),
      className: "bg-muted text-muted-foreground",
    },
    in_progress: {
      icon: Play,
      label: t("release.state.in_progress"),
      className: "bg-primary/10 text-primary",
    },
    completed: {
      icon: CheckCircle2,
      label: t("release.state.completed"),
      className: "bg-success/10 text-success",
    },
    cancelled: {
      icon: XCircle,
      label: t("release.state.cancelled"),
      className: "bg-muted text-muted-foreground line-through",
    },
    blocked: {
      icon: ShieldAlert,
      label: t("release.state.blocked"),
      className: "bg-destructive/10 text-destructive",
    },
  };

  const { icon: Icon, label, className: variantClass } = config[state];
  return (
    <span
      className={cn(
        "inline-flex items-center gap-1.5 rounded-full px-2.5 py-0.5 text-xs font-medium",
        variantClass,
        className,
      )}
    >
      <Icon className="h-3.5 w-3.5" aria-hidden="true" />
      <span>{label}</span>
    </span>
  );
}
