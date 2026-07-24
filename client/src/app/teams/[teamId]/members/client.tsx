"use client";

import { useEffect, useState } from "react";
import { useRouter } from "next/navigation";
import { useAuth } from "@/stores/auth";
import { api, ApiError } from "@/lib/api";
import { t } from "@/lib/i18n";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog";
import { Input } from "@/components/ui/input";
import { ConfirmDialog } from "@/components/ConfirmDialog";
import { useVigilSocket } from "@/stores/socket";
import { useRouteParams } from "@/lib/useRouteParams";

interface Member {
  user_id: string;
  display_name: string;
  email: string;
  role: string;
  joined_at: string;
}

// COMPONENTS
export function MembersClient() {
  const { teamId } = useRouteParams();
  const { token, user } = useAuth();
  const router = useRouter();

  // Data
  const [members, setMembers] = useState<Member[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState("");
  const [actionError, setActionError] = useState("");

  // My role
  const myMember = members.find((m) => m.user_id === user?.id);
  const isManager = myMember?.role === "manager";

  // Transfer dialog
  const [transferTarget, setTransferTarget] = useState<Member | null>(null);
  // eslint-disable-next-line @typescript-eslint/no-unused-vars
  const [transferLoading, setTransferLoading] = useState(false);

  // Leave dialog
  const [leaveOpen, setLeaveOpen] = useState(false);
  // eslint-disable-next-line @typescript-eslint/no-unused-vars
  const [leaveLoading, setLeaveLoading] = useState(false);

  // Invite dialog
  const [inviteOpen, setInviteOpen] = useState(false);
  const [inviteCode, setInviteCode] = useState<string | null>(null);
  const [inviteLoading, setInviteLoading] = useState(false);
  const [copied, setCopied] = useState(false);

  // Kick dialog
  const [kickTarget, setKickTarget] = useState<Member | null>(null);
  // eslint-disable-next-line @typescript-eslint/no-unused-vars
  const [kickLoading, setKickLoading] = useState(false);

  // Ban dialog
  const [banTarget, setBanTarget] = useState<Member | null>(null);
  const [banDuration, setBanDuration] = useState<"7d" | "30d" | "90d" | "permanent" | "custom">("7d");
  const [banCustomDate, setBanCustomDate] = useState("");
  const [banReason, setBanReason] = useState("");
  const [banLoading, setBanLoading] = useState(false);

  const { lastEvent } = useVigilSocket();

  // Fetch
  async function fetchMembers() {
    if (!token) return;
    setLoading(true);
    try {
      const data = await api<Member[]>(`/teams/${teamId}/members`, { token });
      setMembers(data);
    } catch {
      setError(t("common.error"));
    } finally {
      setLoading(false);
    }
  }

  useEffect(() => {
    // eslint-disable-next-line react-hooks/set-state-in-effect
    fetchMembers();
  // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [token, teamId]);

  useEffect(() => {
  if (!lastEvent) return;

  if (
    lastEvent.type === "member_role_changed" &&
    lastEvent.team_id === teamId
  ) {
    const changedUserId = lastEvent.user_id as string;
    const newRole = lastEvent.new_role as string;

    // eslint-disable-next-line react-hooks/set-state-in-effect
    setMembers((prev) =>
      prev.map((m) =>
        m.user_id === changedUserId ? { ...m, role: newRole } : m,
      ),
    );
  }

  if (
      lastEvent &&
      lastEvent.type === "member_joined" &&
      lastEvent.team_id === teamId
    ) {
      const newMember: Member = {
        user_id: lastEvent.user_id as string,
        display_name: lastEvent.display_name as string,
        email: "",
        role: lastEvent.role as string,
        joined_at: new Date().toISOString(),
      };
      setMembers((prev) => {
        // Don't add if already present (e.g. double event from Strict Mode)
        if (prev.some((m) => m.user_id === newMember.user_id)) return prev;
        return [...prev, newMember];
      });
    }

  if (
    lastEvent &&
    (lastEvent.type === "member_kicked" || lastEvent.type === "member_banned") &&
    lastEvent.team_id === teamId
  ) {
      const kickedUserId = lastEvent.user_id as string;
      setMembers((prev) => prev.filter((m) => m.user_id !== kickedUserId));
    }
}, [lastEvent, teamId]);

  // Role change
  async function handleRoleChange(targetUserId: string, newRole: string) {
    setActionError("");
    try {
      await api(`/teams/${teamId}/members/${targetUserId}/role`, {
        method: "PATCH",
        token,
        body: { role: newRole },
      });
      setMembers((prev) =>
        prev.map((m) =>
          m.user_id === targetUserId ? { ...m, role: newRole } : m,
        ),
      );
    } catch (e) {
      setActionError(e instanceof ApiError ? e.message : t("common.error"));
    }
  }

  // Transfer Manager
  async function handleTransfer() {
    if (!transferTarget) return;
    setTransferLoading(true);
    setActionError("");
    try {
      await api(`/teams/${teamId}/transfer-manager`, {
        method: "POST",
        token,
        body: { target_user_id: transferTarget.user_id },
      });
      // Roles update via WS 
      setTransferTarget(null);
    } catch (e) {
      setActionError(e instanceof ApiError ? e.message : t("common.error"));
    } finally {
      setTransferLoading(false);
    }
  }

  // Leave
  async function handleLeave() {
    setLeaveLoading(true);
    setActionError("");
    try {
      await api(`/teams/${teamId}/leave`, {
        method: "POST",
        token,
      });
      router.push("/onboarding");
    } catch (e) {
      if (e instanceof ApiError && e.status === 409) {
        setActionError(t("members.leave.managerError"));
      } else {
        setActionError(e instanceof ApiError ? e.message : t("common.error"));
      }
      setLeaveOpen(false);
    } finally {
      setLeaveLoading(false);
    }
  }

  // Invite
  function handleInviteOpenChange(next: boolean) {
    setInviteOpen(next);
    if (!next) {
      setInviteCode(null);
      setCopied(false);
    }
  }

  async function handleGenerateCode() {
    setInviteLoading(true);
    try {
      const res = await api<{ code: string }>(`/teams/${teamId}/invitations`, {
        method: "POST",
        token,
      });
      setInviteCode(res.code);
    } catch {
      // retry possible
    } finally {
      setInviteLoading(false);
    }
  }

  async function handleCopyCode() {
    if (!inviteCode) return;
    await navigator.clipboard.writeText(inviteCode);
    setCopied(true);
    setTimeout(() => setCopied(false), 2000);
  }

  // Kick
  async function handleKick() {
    if (!kickTarget) return;
    setKickLoading(true);
    setActionError("");
    try {
      await api(`/teams/${teamId}/members/${kickTarget.user_id}/kick`, {
        method: "POST",
        token,
      });
      setKickTarget(null);
    } catch (e) {
      setActionError(e instanceof ApiError ? e.message : t("common.error"));
    } finally {
      setKickLoading(false);
    }
  }

  // Ban
  function computeExpiresAt(): number | null {
    const now = Math.floor(Date.now() / 1000);
    switch (banDuration) {
      case "7d":  return now + 7 * 86400;
      case "30d": return now + 30 * 86400;
      case "90d": return now + 90 * 86400;
      case "permanent": return null;
      case "custom": {
        if (!banCustomDate) return null;
        const ts = Math.floor(new Date(banCustomDate).getTime() / 1000);
        return isNaN(ts) ? null : ts;
      }
    }
  }

  async function handleBan() {
    if (!banTarget) return;
    setBanLoading(true);
    setActionError("");
    try {
      const expires_at = computeExpiresAt();

      // custom date without input, or in the past
      if (banDuration === "custom") {
        if (!expires_at || expires_at <= Math.floor(Date.now() / 1000)) {
          setActionError(t("members.ban.error.pastDate"));
          setBanLoading(false);
          return;
        }
      }

      await api(`/teams/${teamId}/members/${banTarget.user_id}/ban`, {
        method: "POST",
        token,
        body: {
          expires_at,
          reason: banReason.trim() || null,
        },
      });
      setBanTarget(null);
      setBanDuration("7d");
      setBanCustomDate("");
      setBanReason("");
    } catch (e) {
      setActionError(e instanceof ApiError ? e.message : t("common.error"));
    } finally {
      setBanLoading(false);
    }
  }

  // Render
  if (loading) {
    return (
      <div className="p-6 text-muted-foreground">{t("common.loading")}</div>
    );
  }

  if (error) {
    return <div className="p-6 text-destructive">{error}</div>;
  }

  return (
    <>
      <div className="space-y-4 p-6">
        {/* Header */}
        <div className="flex items-center justify-between">
          <h1 className="text-2xl font-semibold">{t("members.title")}</h1>
          <div className="flex gap-2">
            {isManager && (
              <Button onClick={() => setInviteOpen(true)}>
                {t("members.invite")}
              </Button>
            )}
            <Button variant="outline" onClick={() => setLeaveOpen(true)}>
              {t("members.leave")}
            </Button>
          </div>
        </div>

        {/* Action error */}
        {actionError && (
          <p className="text-sm text-destructive">{actionError}</p>
        )}

        {/* Members list */}
        <Card>
          <CardHeader>
            <CardTitle>{t("members.title")}</CardTitle>
          </CardHeader>
          <CardContent className="space-y-2">
            {members.map((member) => {
              const isMe = member.user_id === user?.id;
              const isMemberManager = member.role === "manager";

              return (
                <div
                  key={member.user_id}
                  className="flex items-center justify-between rounded-md border px-4 py-3"
                >
                  {/* Left: name + role */}
                  <div>
                    <span className="font-medium">
                      {member.display_name}
                      {isMe && (
                        <span className="ml-1 text-sm text-muted-foreground">
                          {t("members.you")}
                        </span>
                      )}
                    </span>
                    <span className="ml-2 text-sm text-muted-foreground">
                      {t(`members.role.${member.role}`)}
                    </span>
                  </div>

                  {/* Right: actions (Manager only, not on self, not on other Managers) */}
                  {isManager && !isMe && !isMemberManager && (
                    <div className="flex gap-2">
                      {member.role === "observer" ? (
                        <Button
                          size="sm"
                          variant="outline"
                          onClick={() =>
                            handleRoleChange(member.user_id, "responder")
                          }
                        >
                          {t("members.promote")}
                        </Button>
                      ) : (
                        <Button
                          size="sm"
                          variant="outline"
                          onClick={() =>
                            handleRoleChange(member.user_id, "observer")
                          }
                        >
                          {t("members.demote")}
                        </Button>
                      )}
                      <Button
                        size="sm"
                        variant="outline"
                        onClick={() => setTransferTarget(member)}
                      >
                        {t("members.transfer")}
                      </Button>
                      <Button
                        size="sm"
                        variant="outline"
                        onClick={() => setKickTarget(member)}
                      >
                        {t("members.kick")}
                      </Button>
                      <Button
                        size="sm"
                        variant="destructive"
                        onClick={() => setBanTarget(member)}
                      >
                        {t("members.ban")}
                      </Button>
                    </div>
                  )}
                </div>
              );
            })}
          </CardContent>
        </Card>
      </div>

      {/* Transfer confirm */}
      <ConfirmDialog
        open={!!transferTarget}
        onOpenChange={(open) => {
          if (!open) setTransferTarget(null);
        }}
        title={t("members.transfer.title")}
        description={t("members.transfer.desc").replace(
          "{name}",
          transferTarget?.display_name ?? "",
        )}
        confirmLabel={t("members.transfer.confirm")}
        destructive
        onConfirm={handleTransfer}
      />

      {/* Kick confirm */}
      <ConfirmDialog
        open={!!kickTarget}
        onOpenChange={(open) => {
          if (!open) setKickTarget(null);
        }}
        title={t("members.kick.title")}
        description={t("members.kick.desc").replace(
          "{name}",
          kickTarget?.display_name ?? "",
        )}
        confirmLabel={t("members.kick.confirm")}
        destructive
        onConfirm={handleKick}
      />

      {/* Leave confirm */}
      <ConfirmDialog
        open={leaveOpen}
        onOpenChange={setLeaveOpen}
        title={t("members.leave.title")}
        description={t("members.leave.desc")}
        confirmLabel={t("members.leave.confirm")}
        destructive
        onConfirm={handleLeave}
      />

      {/* Invite dialog */}
      <Dialog open={inviteOpen} onOpenChange={handleInviteOpenChange}>
        <DialogContent>
          <DialogHeader>
            <DialogTitle>{t("teams.invite.dialogTitle")}</DialogTitle>
            <DialogDescription>
              {t("teams.invite.dialogDesc")}
            </DialogDescription>
          </DialogHeader>
          <div className="space-y-4">
            {!inviteCode ? (
              <Button
                onClick={handleGenerateCode}
                disabled={inviteLoading}
                className="w-full"
              >
                {inviteLoading ? "…" : t("teams.invite.generate")}
              </Button>
            ) : (
              <div className="flex items-center gap-2">
                <Input
                  value={inviteCode}
                  readOnly
                  className="font-mono text-lg tracking-widest"
                />
                <Button variant="outline" onClick={handleCopyCode}>
                  {copied ? t("action.copied") : t("action.copy")}
                </Button>
              </div>
            )}
          </div>
          <DialogFooter>
            <Button
              variant="outline"
              onClick={() => handleInviteOpenChange(false)}
            >
              {t("action.close")}
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>

      {/* Ban dialog */}
      <Dialog
        open={!!banTarget}
        onOpenChange={(open) => {
          if (!open) {
            setBanTarget(null);
            setBanDuration("7d");
            setBanCustomDate("");
            setBanReason("");
          }
        }}
      >
        <DialogContent>
          <DialogHeader>
            <DialogTitle>
              {t("members.ban.title").replace(
                "{name}",
                banTarget?.display_name ?? "",
              )}
            </DialogTitle>
            <DialogDescription>{t("members.ban.desc")}</DialogDescription>
          </DialogHeader>

          <div className="space-y-4">
            {/* Duration presets */}
            <div>
              <label className="mb-2 block text-sm font-medium">
                {t("members.ban.duration")}
              </label>
              <div className="flex flex-wrap gap-2">
                {(["7d", "30d", "90d", "permanent", "custom"] as const).map((d) => (
                  <Button
                    key={d}
                    type="button"
                    variant={banDuration === d ? "default" : "outline"}
                    size="sm"
                    onClick={() => setBanDuration(d)}
                  >
                    {t(`members.ban.duration.${d}`)}
                  </Button>
                ))}
              </div>
            </div>

            {/* Custom date input */}
            {banDuration === "custom" && (
              <div>
                <label
                  htmlFor="ban-custom-date"
                  className="mb-2 block text-sm font-medium"
                >
                  {t("members.ban.customDate")}
                </label>
                <Input
                  id="ban-custom-date"
                  type="datetime-local"
                  value={banCustomDate}
                  onChange={(e) => setBanCustomDate(e.target.value)}
                />
              </div>
            )}

            {/* Optional reason */}
            <div>
              <label
                htmlFor="ban-reason"
                className="mb-2 block text-sm font-medium"
              >
                {t("members.ban.reason")}
              </label>
              <Input
                id="ban-reason"
                value={banReason}
                onChange={(e) => setBanReason(e.target.value)}
                placeholder={t("members.ban.reason.placeholder")}
              />
            </div>
          </div>

          <DialogFooter>
            <Button
              variant="outline"
              onClick={() => setBanTarget(null)}
              disabled={banLoading}
            >
              {t("action.cancel")}
            </Button>
            <Button
              variant="destructive"
              onClick={handleBan}
              disabled={banLoading}
            >
              {banLoading ? "…" : t("members.ban.confirm")}
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>
    </>
  );
}
