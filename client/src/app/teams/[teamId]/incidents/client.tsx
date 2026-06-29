"use client";

import { useEffect, useState } from "react";
import { useParams, useRouter } from "next/navigation";
import { useAuth } from "@/stores/auth";
import { api, ApiError } from "@/lib/api";
import { t } from "@/lib/i18n";
import { StateBadge, type IncidentState } from "@/components/StateBadge";
import { SeverityBadge, type Severity } from "@/components/SeverityBadge";
import { Button } from "@/components/ui/button";
import { Card, CardContent } from "@/components/ui/card";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { saveLastTeam } from "@/lib/navigation";

interface IncidentRow {
  id: string;
  title: string;
  status: IncidentState;
  severity: Severity;
  created_at: number;
}

// Helpers
function formatDate(ts: number): string {
  return new Date(ts * 1000).toLocaleDateString(undefined, {
    month: "short",
    day: "numeric",
    hour: "2-digit",
    minute: "2-digit",
  });
}

const STATUS_OPTIONS: IncidentState[] = ["open", "acknowledged", "escalated", "resolved"];
const SEVERITY_OPTIONS: Severity[] = ["low", "medium", "high", "critical"];

// COMPONENTS
export function IncidentsClient() {
  const { teamId } = useParams<{ teamId: string }>();
  const { token, user } = useAuth();
  const router = useRouter();

  // Data
  const [incidents, setIncidents] = useState<IncidentRow[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState("");

  // Role — fetched from team membership
  const [role, setRole] = useState<string | null>(null);

  // Filters
  const [statusFilter, setStatusFilter] = useState<string>("");
  const [severityFilter, setSeverityFilter] = useState<string>("");

  // Create incident dialog
  const [createOpen, setCreateOpen] = useState(false);
  const [createTitle, setCreateTitle] = useState("");
  const [createBody, setCreateBody] = useState("");
  const [createSeverity, setCreateSeverity] = useState<Severity>("low");
  const [createError, setCreateError] = useState("");
  const [createLoading, setCreateLoading] = useState(false);

  // Fetch incidents
  async function fetchIncidents() {
    if (!token) return;
    setLoading(true);
    setError("");
    try {
      const params = new URLSearchParams();
      if (statusFilter) params.set("status", statusFilter);
      if (severityFilter) params.set("severity", severityFilter);
      const query = params.toString() ? `?${params.toString()}` : "";

      const data = await api<{ incidents: IncidentRow[] }>(
        `/teams/${teamId}/incidents${query}`,
        { token }
      );
      setIncidents(data.incidents);
    } catch {
      setError(t("common.error"));
    } finally {
      setLoading(false);
    }
  }
  
  // Fetch current user's role in this team
  useEffect(() => {
    if (!token || !user) return;
    api<{ user_id: string; display_name: string; role: string }[]>(
      `/teams/${teamId}/members`,
      { token }
    )
      .then((members) => {
      const me = members.find((m) => m.user_id === user.id);
      setRole(me?.role ?? null);
      })
      .catch(() => {});
  }, [token, teamId, user]);

  // Fetch incidents
  useEffect(() => {
    // eslint-disable-next-line react-hooks/set-state-in-effect
    fetchIncidents(); 
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [token, teamId, statusFilter, severityFilter]);

  // Use memory of active team
  useEffect(() => {
  if (teamId) saveLastTeam(teamId);
}, [teamId]);


  // CREATE INCIDENT

  function handleCreateOpenChange(next: boolean) {
    setCreateOpen(next);
    if (!next) {
      setCreateTitle("");
      setCreateBody("");
      setCreateSeverity("low");
      setCreateError("");
    }
  }

  async function handleCreate() {
    const trimmed = createTitle.trim();
    if (!trimmed) {
      setCreateError(t("incidents.create.error.emptyTitle"));
      return;
    }
    setCreateLoading(true);
    setCreateError("");
    try {
      const incident = await api<IncidentRow>(
        `/teams/${teamId}/incidents`,
        {
          method: "POST",
          token,
          body: { title: trimmed, body: createBody, severity: createSeverity },
        }
      );
      setIncidents((prev) => [incident, ...prev]);
      handleCreateOpenChange(false);
      // Navigate directly to the new incident
      router.push(`/teams/${teamId}/incidents/${incident.id}`);
    } catch (e) {
      setCreateError(e instanceof ApiError ? e.message : t("common.error"));
    } finally {
      setCreateLoading(false);
    }
  }

  // RENDER

  return (
    <>
      <div className="space-y-4 p-6">
        {/* Header */}
        <div className="flex items-center justify-between">
          <h1 className="text-2xl font-semibold">{t("incidents.title")}</h1>
          {role === "manager" && (
            <Button onClick={() => setCreateOpen(true)}>
              {t("incidents.new")}
            </Button>
          )}
        </div>

        {/* Filters */}
        <div className="flex gap-3">
          <div className="flex items-center gap-2">
            <Label htmlFor="filter-status">{t("incidents.filter.status")}</Label>
            <select
              id="filter-status"
              value={statusFilter}
              onChange={(e) => setStatusFilter(e.target.value)}
              className="rounded-md border bg-background px-2 py-1 text-sm"
            >
              <option value="">{t("incidents.filter.all")}</option>
              {STATUS_OPTIONS.map((s) => (
                <option key={s} value={s}>{s}</option>
              ))}
            </select>
          </div>
          <div className="flex items-center gap-2">
            <Label htmlFor="filter-severity">{t("incidents.filter.severity")}</Label>
            <select
              id="filter-severity"
              value={severityFilter}
              onChange={(e) => setSeverityFilter(e.target.value)}
              className="rounded-md border bg-background px-2 py-1 text-sm"
            >
              <option value="">{t("incidents.filter.all")}</option>
              {SEVERITY_OPTIONS.map((s) => (
                <option key={s} value={s}>{s}</option>
              ))}
            </select>
          </div>
        </div>

        {/* Content */}
        {loading ? (
          <p className="text-muted-foreground">{t("common.loading")}</p>
        ) : error ? (
          <p className="text-destructive">{error}</p>
        ) : incidents.length === 0 ? (
          <Card>
            <CardContent className="py-12 text-center text-muted-foreground">
              {t("incidents.empty")}
            </CardContent>
          </Card>
        ) : (
          <div className="space-y-2">
            {incidents.map((incident) => (
              <button
                key={incident.id}
                onClick={() =>
                  router.push(`/teams/${teamId}/incidents/${incident.id}`)
                }
                className="w-full rounded-lg border bg-card px-4 py-3 text-left transition-colors hover:bg-muted/50 focus:outline-none focus-visible:ring-2 focus-visible:ring-ring"
              >
                <div className="flex items-center justify-between gap-4">
                  <span className="flex-1 font-medium truncate">
                    {incident.title}
                  </span>
                  <div className="flex shrink-0 items-center gap-2">
                    <StateBadge state={incident.status} />
                    <SeverityBadge severity={incident.severity} />
                    <span className="text-xs text-muted-foreground">
                      {formatDate(incident.created_at)}
                    </span>
                  </div>
                </div>
              </button>
            ))}
          </div>
        )}
      </div>

       {/* Create incident dialog */}
      <Dialog open={createOpen} onOpenChange={handleCreateOpenChange}>
        <DialogContent>
          <DialogHeader>
            <DialogTitle>{t("incidents.create.dialogTitle")}</DialogTitle>
            <DialogDescription>{t("incidents.create.dialogDesc")}</DialogDescription>
          </DialogHeader>
          <div className="space-y-4">
            <div className="space-y-2">
              <Label htmlFor="inc-title">{t("incidents.create.titleLabel")}</Label>
              <Input
                id="inc-title"
                value={createTitle}
                onChange={(e) => setCreateTitle(e.target.value)}
                placeholder={t("incidents.create.titlePlaceholder")}
                onKeyDown={(e) => {
                  if (e.key === "Enter" && !createLoading) handleCreate();
                }}
              />
            </div>
            <div className="space-y-2">
              <Label htmlFor="inc-body">{t("incidents.create.bodyLabel")}</Label>
              <textarea
                id="inc-body"
                value={createBody}
                onChange={(e) => setCreateBody(e.target.value)}
                placeholder={t("incidents.create.bodyPlaceholder")}
                rows={3}
                className="w-full rounded-md border bg-background px-3 py-2 text-sm focus:outline-none focus-visible:ring-2 focus-visible:ring-ring"
              />
            </div>
            <div className="space-y-2">
              <Label htmlFor="inc-severity">{t("incidents.create.severityLabel")}</Label>
              <div className="flex gap-2">
                {SEVERITY_OPTIONS.map((s) => (
                  <button
                    key={s}
                    type="button"
                    onClick={() => setCreateSeverity(s)}
                    className="focus:outline-none focus-visible:ring-2 focus-visible:ring-ring rounded-full"
                  >
                    <SeverityBadge
                      severity={s}
                      className={
                        createSeverity === s
                          ? "ring-2 ring-ring ring-offset-1"
                          : "opacity-50"
                      }
                    />
                  </button>
                ))}
              </div>
            </div>
            {createError && (
              <p className="text-sm text-destructive">{createError}</p>
            )}
          </div>
          <DialogFooter>
            <Button
              variant="outline"
              onClick={() => handleCreateOpenChange(false)}
            >
              {t("incidents.create.cancel")}
            </Button>
            <Button onClick={handleCreate} disabled={createLoading}>
              {createLoading ? "…" : t("incidents.create.submit")}
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>
    </>
  );
}