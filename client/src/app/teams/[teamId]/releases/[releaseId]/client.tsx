"use client";

import { useEffect, useState } from "react";
import { useParams, useRouter } from "next/navigation";
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
}

export function ReleaseDetailClient() {
  const { teamId, releaseId } = useParams<{
    teamId: string;
    releaseId: string;
  }>();
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
          <div className="flex items-center gap-3 rounded-lg border border-destructive/50 bg-destructive/5 px-4 py-3">
            <ShieldAlert className="h-5 w-5 text-destructive" />
            <p className="text-sm font-medium text-destructive">
              {t("release.blocked.banner")}
            </p>
          </div>
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
    </>
  );
}