import {
  ChevronDown,
  Equal,
  ChevronUp,
  Flame,
  type LucideIcon,
} from "lucide-react";
import { cn } from "@/lib/utils";
import { t } from "@/lib/i18n";

export type Severity = "low" | "medium" | "high" | "critical";

interface SeverityConfig {
  icon: LucideIcon;
  label: string;
  className: string;
}

const config: Record<Severity, SeverityConfig> = {
  low: {
    icon: ChevronDown,
    label: t("incident.severity.low"),
    className: "bg-muted text-muted-foreground",
  },
  medium: {
    icon: Equal,
    label: t("incident.severity.medium"),
    className: "bg-primary/10 text-primary",
  },
  high: {
    icon: ChevronUp,
    label: t("incident.severity.high"),
    className: "bg-warning/10 text-warning",
  },
  critical: {
    icon: Flame,
    label: t("incident.severity.critical"),
    className: "bg-destructive/10 text-destructive",
  },
};

interface SeverityBadgeProps {
  severity: Severity;
  className?: string;
}

export function SeverityBadge({ severity, className }: SeverityBadgeProps) {
  const { icon: Icon, label, className: variantClass } = config[severity];
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
