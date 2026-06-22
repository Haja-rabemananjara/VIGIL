import {
    AlertCircle,
    CheckCircle2,
    AlertTriangle,
    Clock,
    type LucideIcon
} from "lucide-react";
import { cn } from "@/lib/utils";
import { t } from "@/lib/i18n";

export type IncidentState = "open" | "acknowledged" | "escalated" | "resolved";

interface StateConfig {
    icon: LucideIcon;
    label: string;
    className: string;
}

const config: Record<IncidentState, StateConfig> = {
    open: {
        icon: AlertCircle,
        label: t("incident.state.open"),
        className: "bg-muted text-muted-foreground",
    },
    acknowledged: {
        icon: Clock,
        label: t("incident.state.acknowledged"),
        className: "bg-primary/10 text-primary",
    },
    escalated: {
        icon: AlertTriangle,
        label: t("incident.state.escalated"),
        className: "bg-warning/10 text-warning",
    },
    resolved: {
        icon: CheckCircle2,
        label: t("incident.state.resolved"),
        className: "bg-success/10 text-success",
    },
};

interface StateBadgeProps {
    state: IncidentState;
    className?: string;
}

export function StateBadge({ state, className }: StateBadgeProps) {
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