"use client";

import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import { StateBadge, type IncidentState } from "@/components/StateBadge";
import { SeverityBadge, type Severity } from "@/components/SeverityBadge";
import { t, type TranslationKey } from "@/lib/i18n";

interface Incident {
  title: string;
  body: string;
  status: IncidentState;
  severity: Severity;
  created_by: string;
  created_at: number;
}

interface Props {
  incident: Incident;
  assignee: string | null;
  displayName: (userId: string) => string;
  canAct: boolean;
  isManager: boolean;
  nextTransitions: IncidentState[];
  transitionLoading: boolean;
  onTransition: (state: IncidentState) => void;
  onOpenAssign: () => void;
}

const TRANSITION_LABELS: Record<IncidentState, TranslationKey> = {
  acknowledged: "incidents.detail.acknowledge",
  escalated: "incidents.detail.escalate",
  resolved: "incidents.detail.resolve",
  open: "incident.state.open",
};

function formatDate(ts: number): string {
  return new Date(ts * 1000).toLocaleString(undefined, {
    month: "short",
    day: "numeric",
    hour: "2-digit",
    minute: "2-digit",
  });
}

export function IncidentHeader({
  incident,
  assignee,
  displayName,
  canAct,
  isManager,
  nextTransitions,
  transitionLoading,
  onTransition,
  onOpenAssign,
}: Props) {
  return (
    <Card>
      <CardHeader>
        <div className="flex items-start justify-between gap-4">
          <CardTitle className="text-xl">{incident.title}</CardTitle>
          <div className="flex shrink-0 items-center gap-2">
            <StateBadge state={incident.status} />
            <SeverityBadge severity={incident.severity} />
          </div>
        </div>
      </CardHeader>
      <CardContent className="space-y-4">
        {incident.body && (
          <p className="text-sm text-muted-foreground">{incident.body}</p>
        )}

        <div className="text-sm text-muted-foreground">
          {t("incidents.detail.createdBy")}{" "}
          <span className="text-foreground">
            {displayName(incident.created_by)}
          </span>
          {" · "}
          {formatDate(incident.created_at)}
          <div className="text-sm text-muted-foreground">
            {t("incidents.detail.assignee")}{" "}
            {assignee ? (
              <span className="font-medium text-foreground">
                {displayName(assignee)}
              </span>
            ) : (
              <span className="italic">{t("incidents.detail.noAssignee")}</span>
            )}
          </div>
        </div>

        {canAct && (
          <div className="flex items-center justify-between pt-2">
            <div className="flex flex-wrap gap-2">
              {nextTransitions.map((toStatus) => (
                <Button
                  key={toStatus}
                  size="sm"
                  variant={toStatus === "resolved" ? "default" : "outline"}
                  disabled={transitionLoading}
                  onClick={() => onTransition(toStatus)}
                >
                  {t(TRANSITION_LABELS[toStatus])}
                </Button>
              ))}
            </div>
            {isManager && (
              <Button size="sm" variant="outline" onClick={onOpenAssign}>
                {t("incidents.detail.assign")}
              </Button>
            )}
          </div>
        )}
      </CardContent>
    </Card>
  );
}
