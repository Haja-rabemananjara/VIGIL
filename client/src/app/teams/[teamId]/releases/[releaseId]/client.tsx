"use client";

import { useEffect, useState } from "react";
import { useRouter } from "next/navigation";
import { useAuth } from "@/stores/auth";
import { api, ApiError } from "@/lib/api";
import { t } from "@/lib/i18n";
import {
  ReleaseStateBadge,
  type ReleaseState,
} from "@/components/ReleaseStateBadge";
import {
  ReleaseStepper,
  type ReleaseStep,
} from "@/components/ReleaseStepper";
import { ConfirmDialog } from "@/components/ConfirmDialog";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { useVigilSocket } from "@/stores/socket";
import { ShieldAlert } from "lucide-react";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog";
import { StateBadge, type IncidentState } from "@/components/StateBadge";
import { SeverityBadge, type Severity } from "@/components/SeverityBadge";
import { Link2, Unlink } from "lucide-react";
import { useRouteParams } from "@/lib/useRouteParams";

interface ReleaseDetail {
  id: string;
  team_id: string;
  title: string;
  body: string;
  status: ReleaseState;
  created_by: string;
  created_at: number;
  updated_at: number;
  started_at: number | null;
  completed_at: number | null;
  cancelled_at: number | null;
  steps: ReleaseStep[];
  progress: { completed: number; total: number };
  linked_incidents: LinkedIncident[];
}

interface IncidentRow {
  id: string;
  title: string;
  status: IncidentState;
  severity: Severity;
}

interface LinkedIncident {
  id: string;
  title: string;
  status: string;
  severity: string;
}

export function ReleaseDetailClient() {
  const { teamId, releaseId } = useRouteParams();
  const { token, user } = useAuth();
  const router = useRouter();

  const [release, setRelease] = useState<ReleaseDetail | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState("");
  const [actionError, setActionError] = useState("");
  const [role, setRole] = useState<string | null>(null);

  // Action loading states
  const [starting, setStarting] = useState(false);
  const [validating, setValidating] = useState(false);
  const [cancelOpen, setCancelOpen] = useState(false);

    // Link incident dialog
  const [linkOpen, setLinkOpen] = useState(false);
  const [teamIncidents, setTeamIncidents] = useState<IncidentRow[]>([]);
  const [linkLoading, setLinkLoading] = useState(false);
  const [unlinkLoading, setUnlinkLoading] = useState<string | null>(null);

  const { lastEvent, reconnectCount } = useVigilSocket();

  const isManager = role === "manager";
  const canValidate = role === "manager" || role === "responder";

  // Fetch release
  async function fetchRelease() {
    if (!token) return;
    setLoading(true);
    setError("");
    try {
      const data = await api<ReleaseDetail>(
        `/teams/${teamId}/releases/${releaseId}`,
        { token },
      );
      setRelease(data);
    } catch {
      setError(t("common.error"));
    } finally {
      setLoading(false);
    }
  }

  // Fetch role
  useEffect(() => {
    if (!token || !user) return;
    api<{ user_id: string; role: string }[]>(
      `/teams/${teamId}/members`,
      { token },
    )
      .then((members) => {
        const me = members.find((m) => m.user_id === user.id);
        setRole(me?.role ?? null);
      })
      .catch(() => {});
  }, [token, teamId, user]);

  useEffect(() => {
    // eslint-disable-next-line react-hooks/set-state-in-effect
    fetchRelease();
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [token, teamId, releaseId]);

  // Re-fetch after reconnect
  useEffect(() => {
    // eslint-disable-next-line react-hooks/set-state-in-effect
    if (reconnectCount > 0) fetchRelease();
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [reconnectCount]);

  // Real-time updates
  useEffect(() => {
    if (!lastEvent || !release) return;
    if (lastEvent.team_id !== teamId) return;

    if (
      lastEvent.type === "release_state_changed" &&
      lastEvent.release_id === releaseId
    ) {
      // Re-fetch to get full updated state

      // eslint-disable-next-line react-hooks/set-state-in-effect
      fetchRelease();
    }

    if (
      (lastEvent.type === "release_state_changed" ||
      lastEvent.type === "release_step_validated") &&
      lastEvent.release_id === releaseId
    ) {
      fetchRelease();
    }

    if (
      lastEvent.type === "release_incident_linked" ||
      lastEvent.type === "release_incident_unlinked"
    ) {
      const affectedReleaseId = lastEvent.release_id as string;
      if (affectedReleaseId === releaseId) {
        api<ReleaseDetail>(`/teams/${teamId}/releases/${releaseId}`, { token: token! })
          .then(setRelease)
          .catch(() => {});
      }
    }

  // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [lastEvent, teamId, releaseId]);

  // Actions
  async function handleStart() {
    setStarting(true);
    setActionError("");
    try {
      const data = await api<ReleaseDetail>(
        `/teams/${teamId}/releases/${releaseId}/start`,
        { method: "POST", token },
      );
      setRelease(data);
    } catch (e) {
      setActionError(e instanceof ApiError ? e.message : t("common.error"));
    } finally {
      setStarting(false);
    }
  }

  async function handleValidateStep(stepId: string) {
    setValidating(true);
    setActionError("");
    try {
      const data = await api<ReleaseDetail>(
        `/teams/${teamId}/releases/${releaseId}/steps/${stepId}/validate`,
        { method: "POST", token },
      );
      setRelease(data);
    } catch (e) {
      setActionError(e instanceof ApiError ? e.message : t("common.error"));
    } finally {
      setValidating(false);
    }
  }

  async function handleCancel() {
    setActionError("");
    try {
      const data = await api<ReleaseDetail>(
        `/teams/${teamId}/releases/${releaseId}/cancel`,
        { method: "POST", token },
      );
      setRelease(data);
      setCancelOpen(false);
    } catch (e) {
      setActionError(e instanceof ApiError ? e.message : t("common.error"));
    }
  }

   // Link/Unlink
  async function openLinkDialog() {
    setLinkOpen(true);
    try {
      const data = await api<{ incidents: IncidentRow[] }>(
        `/teams/${teamId}/incidents`,
        { token },
      );
      // Filter out already-linked incidents
      const linkedIds = new Set(
        release?.linked_incidents?.map((li: LinkedIncident) => li.id) ?? [],
      );
      setTeamIncidents(
        data.incidents.filter((inc) => !linkedIds.has(inc.id)),
      );
    } catch {
      /* keep dialog open with empty list */
    }
  }

  async function handleLinkIncident(incidentId: string) {
    setLinkLoading(true);
    try {
      const data = await api<ReleaseDetail>(
        `/teams/${teamId}/releases/${releaseId}/link`,
        { method: "POST", token, body: { incident_id: incidentId } },
      );
      setRelease(data);
      setLinkOpen(false);
    } catch (e) {
      setActionError(e instanceof ApiError ? e.message : t("common.error"));
    } finally {
      setLinkLoading(false);
    }
  }

  async function handleUnlinkIncident(incidentId: string) {
    setUnlinkLoading(incidentId);
    setActionError("");
    try {
      const data = await api<ReleaseDetail>(
        `/teams/${teamId}/releases/${releaseId}/unlink`,
        { method: "POST", token, body: { incident_id: incidentId } },
      );
      setRelease(data);
    } catch (e) {
      setActionError(e instanceof ApiError ? e.message : t("common.error"));
    } finally {
      setUnlinkLoading(null);
    }
  }

  // Render
  if (loading) {
    return (
      <div className="p-6 text-muted-foreground">{t("common.loading")}</div>
    );
  }

  if (error || !release) {
    return <div className="p-6 text-destructive">{error || "Not found"}</div>;
  }

  const isTerminal =
    release.status === "completed" || release.status === "cancelled";

  return (
    <>
      <div className="space-y-4 p-6">
        {/* Back link */}
        <button
          onClick={() => router.push(`/teams/${teamId}/releases`)}
          className="text-sm text-muted-foreground hover:text-foreground"
        >
          {t("releases.title")}
        </button>

        {/* Header */}
        <div className="flex items-start justify-between gap-4">
          <div>
            <h1 className="text-2xl font-semibold">{release.title}</h1>
            {release.body && (
              <p className="mt-1 text-sm text-muted-foreground">
                {release.body}
              </p>
            )}
          </div>
          <div className="flex shrink-0 items-center gap-2">
            <ReleaseStateBadge state={release.status} />
            <span className="text-sm text-muted-foreground">
              {release.progress.completed}/{release.progress.total}
            </span>
          </div>
        </div>

        {/* Blocked banner */}
        {release.status === "blocked" && (
          <div className="rounded-lg border border-destructive/50 bg-destructive/5 px-4 py-3">
            <div className="flex items-center gap-3">
              <ShieldAlert className="h-5 w-5 text-destructive" />
              <p className="text-sm font-medium text-destructive">
                {t("release.blocked.banner")}
              </p>
            </div>
            {release.linked_incidents?.length > 0 && (
              <div className="mt-2 space-y-1 pl-8">
                {release.linked_incidents
                  .filter((li: LinkedIncident) => li.status !== "resolved")
                  .map((li: LinkedIncident) => (
                    <button
                      key={li.id}
                      onClick={() =>
                        router.push(`/teams/${teamId}/incidents/${li.id}`)
                      }
                      className="text-sm text-destructive underline hover:no-underline"
                    >
                      {li.title}
                    </button>
                  ))}
              </div>
            )}
          </div>
        )}

        {/* Linked incidents */}
        {release.linked_incidents?.length > 0 && release.status !== "blocked" && (
          <Card>
            <CardHeader>
              <CardTitle className="flex items-center gap-2">
                <Link2 className="h-4 w-4" />
                {t("release.linked.title")}
              </CardTitle>
            </CardHeader>
            <CardContent className="space-y-2">
              {release.linked_incidents.map((li: LinkedIncident) => (
                <div
                  key={li.id}
                  className="flex items-center justify-between rounded-md border px-3 py-2"
                >
                  <button
                    onClick={() =>
                      router.push(`/teams/${teamId}/incidents/${li.id}`)
                    }
                    className="text-sm font-medium hover:underline"
                  >
                    {li.title}
                  </button>
                  <div className="flex items-center gap-2">
                    <StateBadge state={li.status as IncidentState} />
                    <SeverityBadge severity={li.severity as Severity} />
                    {isManager && (
                      <Button
                        variant="ghost"
                        size="sm"
                        onClick={() => handleUnlinkIncident(li.id)}
                        disabled={unlinkLoading === li.id}
                        className="relative group"
                      >
                        <Unlink className="h-3.5 w-3.5" />
                        <span className="pointer-events-none absolute -top-8 left-1/2 -translate-x-1/2 scale-0 rounded bg-gray-800 px-2 py-1 text-xs text-white transition-all group-hover:scale-100 z-10">
                          {t("release.unlink")}
                        </span>
                      </Button>
                    )}
                  </div>
                </div>
              ))}
            </CardContent>
          </Card>
        )}

        {/* Action error */}
        {actionError && (
          <p className="text-sm text-destructive">{actionError}</p>
        )}

        {/* Actions */}
        <div className="flex gap-2">
          {isManager && release.status === "created" && (
            <Button onClick={handleStart} disabled={starting}>
              {starting ? "…" : t("release.actions.start")}
            </Button>
          )}
          {isManager && !isTerminal && (
            <Button variant="destructive" onClick={() => setCancelOpen(true)}>
              {t("release.actions.cancel")}
            </Button>
          )}
          {isManager && !isTerminal && (
            <Button variant="outline" onClick={openLinkDialog}>
              <Link2 className="mr-2 h-4 w-4" />
              {t("release.actions.link")}
            </Button>
          )}
        </div>

        {/* Steps stepper */}
        <Card>
          <CardHeader>
            <CardTitle>{t("release.steps.title")}</CardTitle>
          </CardHeader>
          <CardContent>
            <ReleaseStepper
              steps={release.steps}
              releaseStatus={release.status}
              onValidate={canValidate ? handleValidateStep : undefined}
              validating={validating}
            />
          </CardContent>
        </Card>

        {/* Timestamps */}
        <Card>
          <CardContent className="space-y-1 py-4 text-sm text-muted-foreground">
            <p>
              {t("release.info.created")}{" "}
              {new Date(release.created_at * 1000).toLocaleString()}
            </p>
            {release.started_at && (
              <p>
                {t("release.info.started")}{" "}
                {new Date(release.started_at * 1000).toLocaleString()}
              </p>
            )}
            {release.completed_at && (
              <p>
                {t("release.info.completed")}{" "}
                {new Date(release.completed_at * 1000).toLocaleString()}
              </p>
            )}
            {release.cancelled_at && (
              <p>
                {t("release.info.cancelled")}{" "}
                {new Date(release.cancelled_at * 1000).toLocaleString()}
              </p>
            )}
          </CardContent>
        </Card>
      </div>

      {/* Cancel confirmation */}
      <ConfirmDialog
        open={cancelOpen}
        onOpenChange={setCancelOpen}
        title={t("release.cancel.title")}
        description={t("release.cancel.desc").replace(
          "{name}",
          release.title,
        )}
        confirmLabel={t("release.cancel.confirm")}
        destructive
        onConfirm={handleCancel}
      />

      {/* Link incident dialog */}
      <Dialog open={linkOpen} onOpenChange={setLinkOpen}>
        <DialogContent>
          <DialogHeader>
            <DialogTitle>{t("release.link.dialogTitle")}</DialogTitle>
            <DialogDescription>
              {t("release.link.dialogDesc")}
            </DialogDescription>
          </DialogHeader>
          <div className="max-h-64 space-y-1 overflow-auto">
            {teamIncidents.length === 0 ? (
              <p className="py-4 text-center text-sm text-muted-foreground">
                {t("release.link.noIncidents")}
              </p>
            ) : (
              teamIncidents.map((inc) => (
                <button
                  key={inc.id}
                  onClick={() => handleLinkIncident(inc.id)}
                  disabled={linkLoading}
                  className="flex w-full items-center justify-between rounded-md border px-3 py-2 text-left transition-colors hover:bg-muted/50 disabled:opacity-50"
                >
                  <span className="text-sm font-medium truncate">
                    {inc.title}
                  </span>
                  <div className="flex shrink-0 items-center gap-2">
                    <StateBadge state={inc.status} />
                    <SeverityBadge severity={inc.severity} />
                  </div>
                </button>
              ))
            )}
          </div>
          <DialogFooter>
            <Button variant="outline" onClick={() => setLinkOpen(false)}>
              {t("close")}
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>
    </>
  );
}