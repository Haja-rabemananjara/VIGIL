"use client";

import { useEffect, useState } from "react";
import { RequireAuth } from "@/components/RequireAuth";
import { useAuth } from "@/stores/auth";
import { api, ApiError } from "@/lib/api";
import { t } from "@/lib/i18n";
import { Button } from "@/components/ui/button";
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "@/components/ui/card";
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

interface TeamView {
  id: string;
  name: string;
  role: string;
  created_at: string;
}

interface InvitationView {
  id: string;
  code: string;
  expires_at: string | null;
  max_uses: number | null;
  uses: number;
}


export default function OnboardingPage() {
  const { user, token } = useAuth();


  const [teams, setTeams] = useState<TeamView[]>([]);

  const [createOpen, setCreateOpen] = useState(false);
  const [teamName, setTeamName] = useState("");
  const [createError, setCreateError] = useState("");
  const [createLoading, setCreateLoading] = useState(false);

  const [joinOpen, setJoinOpen] = useState(false);
  const [joinCode, setJoinCode] = useState("");
  const [joinError, setJoinError] = useState("");
  const [joinLoading, setJoinLoading] = useState(false);

  const [inviteOpen, setInviteOpen] = useState(false);
  const [inviteTeamId, setInviteTeamId] = useState<string | null>(null);
  const [inviteCode, setInviteCode] = useState<string | null>(null);
  const [inviteLoading, setInviteLoading] = useState(false);
  const [copied, setCopied] = useState(false);



  // Fetch existing teams on mount.
  useEffect(() => {
    if (!token) return;
    api<TeamView[]>("/teams", { token })
      .then(setTeams)
      .catch(() => {});
  }, [token]);

  // Create team
  function handleCreateOpenChange(next: boolean) {
    setCreateOpen(next);
    if (!next) { setTeamName(""); setCreateError(""); }
  }

  async function handleCreate() {
    const trimmed = teamName.trim();
    if (!trimmed) {
      setCreateError(t("teams.create.error.empty"));
      return;
    }
    setCreateLoading(true);
    setCreateError("");
    try {
      const team = await api<TeamView>("/teams", {
        method: "POST", token, body: { name: trimmed },
      });
      setTeams((prev) => [...prev, team]);
      handleCreateOpenChange(false);
    } catch (e) {
      setCreateError(e instanceof ApiError ? e.message : "Something went wrong");
    } finally {
      setCreateLoading(false);
    }
  }

  // Join team
  function handleJoinOpenChange(next: boolean) {
    setJoinOpen(next);
    if (!next) { setJoinCode(""); setJoinError(""); }
  }

  async function handleJoin() {
    const trimmed = joinCode.trim();
    if (!trimmed) {
      setJoinError(t("teams.join.error.empty"));
      return;
    }
    setJoinLoading(true);
    setJoinError("");
    try {
      const result = await api<{ team_id: string; team_name: string; role: string }>(
        "/teams/join",
        { method: "POST", token, body: { code: trimmed } },
      );
      // Add the new team to the local list
      setTeams((prev) => [
        ...prev,
        {
          id: result.team_id,
          name: result.team_name,
          role: result.role,
          created_at: new Date().toISOString(),
        },
      ]);
      handleJoinOpenChange(false);
    } catch (e) {
      if (e instanceof ApiError) {
        setJoinError(e.message);
      } else {
        setJoinError("Something went wrong");
      }
    } finally {
      setJoinLoading(false);
    }
  }

  // Invite (generate code)
  function handleInviteOpenChange(next: boolean) {
    setInviteOpen(next);
    if (!next) {
      setInviteCode(null);
      setInviteTeamId(null);
      setCopied(false);
    }
  }

  function openInviteDialog(teamId: string) {
    setInviteTeamId(teamId);
    setInviteOpen(true);
  }

  async function handleGenerateCode() {
    if (!inviteTeamId) return;
    setInviteLoading(true);
    try {
      const invitation = await api<InvitationView>(
        `/teams/${inviteTeamId}/invitations`,
        { method: "POST", token },
      );
      setInviteCode(invitation.code);
    } catch {
      // If it fails, the user can retry
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

  return (
    <RequireAuth>
      <main className="flex min-h-screen items-center justify-center p-4">
        <div className="w-full max-w-md space-y-6">
          <div className="text-center">
            <h1 className="text-2xl font-semibold">
              {t("onboarding.welcome")}, {user?.display_name}
            </h1>
            <p className="mt-2 text-muted-foreground">
              {t("onboarding.subtitle")}
            </p>
          </div>

          {/* Existing teams */}
          {teams.length > 0 && (
            <Card>
              <CardHeader>
                <CardTitle>{t("onboarding.myTeams")}</CardTitle>
                <CardDescription>
                  {t("onboarding.myTeams.desc")}
                </CardDescription>
              </CardHeader>
              <CardContent className="space-y-2">
                {teams.map((team) => (
                  <div
                    key={team.id}
                    className="flex items-center justify-between rounded-md border px-4 py-2"
                  >
                    <div>
                      <span className="font-medium">{team.name}</span>
                      <span className="ml-2 text-sm text-muted-foreground">
                        {team.role}
                      </span>
                    </div>
                    {team.role === "manager" && (
                      <Button
                        variant="outline"
                        size="sm"
                        onClick={() => openInviteDialog(team.id)}
                      >
                        {t("teams.invite.button")}
                      </Button>
                    )}
                  </div>
                ))}
              </CardContent>
            </Card>
          )}

          <div className="grid gap-4">
            {/* Create a team */}
            <Card>
              <CardHeader>
                <CardTitle>{t("onboarding.create.title")}</CardTitle>
                <CardDescription>
                  {t("onboarding.create.desc")}
                </CardDescription>
              </CardHeader>
              <CardContent>
                <Button className="w-full" onClick={() => setCreateOpen(true)}>
                  {t("onboarding.create.action")}
                </Button>
              </CardContent>
            </Card>

            {/* Join a team */}
            <Card>
              <CardHeader>
                <CardTitle>{t("onboarding.join.title")}</CardTitle>
                <CardDescription>
                  {t("onboarding.join.desc")}
                </CardDescription>
              </CardHeader>
              <CardContent>
                <Button
                  className="w-full"
                  variant="outline"
                  onClick={() => setJoinOpen(true)}
                >
                  {t("onboarding.join.action")}
                </Button>
              </CardContent>
            </Card>
          </div>
        </div>
      </main>

      {/* Create team dialog */}
      <Dialog open={createOpen} onOpenChange={handleCreateOpenChange}>
        <DialogContent>
          <DialogHeader>
            <DialogTitle>{t("teams.create.dialogTitle")}</DialogTitle>
            <DialogDescription>{t("teams.create.dialogDesc")}</DialogDescription>
          </DialogHeader>
          <div className="space-y-2">
            <Label htmlFor="team-name">{t("teams.create.nameLabel")}</Label>
            <Input
              id="team-name"
              value={teamName}
              onChange={(e) => setTeamName(e.target.value)}
              placeholder={t("teams.create.namePlaceholder")}
              onKeyDown={(e) => {
                if (e.key === "Enter" && !createLoading) handleCreate();
              }}
            />
            {createError && <p className="text-sm text-destructive">{createError}</p>}
          </div>
          <DialogFooter>
            <Button variant="outline" onClick={() => handleCreateOpenChange(false)}>
              {t("teams.create.cancel")}
            </Button>
            <Button onClick={handleCreate} disabled={createLoading}>
              {createLoading ? "…" : t("teams.create.submit")}
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>

      {/* Join team dialog */}
      <Dialog open={joinOpen} onOpenChange={handleJoinOpenChange}>
        <DialogContent>
          <DialogHeader>
            <DialogTitle>{t("teams.join.dialogTitle")}</DialogTitle>
            <DialogDescription>{t("teams.join.dialogDesc")}</DialogDescription>
          </DialogHeader>
          <div className="space-y-2">
            <Label htmlFor="join-code">{t("teams.join.codeLabel")}</Label>
            <Input
              id="join-code"
              value={joinCode}
              onChange={(e) => setJoinCode(e.target.value.toUpperCase())}
              placeholder={t("teams.join.codePlaceholder")}
              onKeyDown={(e) => {
                if (e.key === "Enter" && !joinLoading) handleJoin();
              }}
            />
            {joinError && <p className="text-sm text-destructive">{joinError}</p>}
          </div>
          <DialogFooter>
            <Button variant="outline" onClick={() => handleJoinOpenChange(false)}>
              {t("teams.join.cancel")}
            </Button>
            <Button onClick={handleJoin} disabled={joinLoading}>
              {joinLoading ? "…" : t("teams.join.submit")}
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>

      {/* Invite dialog */}
      <Dialog open={inviteOpen} onOpenChange={handleInviteOpenChange}>
        <DialogContent>
          <DialogHeader>
            <DialogTitle>{t("teams.invite.dialogTitle")}</DialogTitle>
            <DialogDescription>{t("teams.invite.dialogDesc")}</DialogDescription>
          </DialogHeader>
          <div className="space-y-4">
            {!inviteCode ? (
              <Button onClick={handleGenerateCode} disabled={inviteLoading} className="w-full">
                {inviteLoading ? "…" : t("teams.invite.generate")}
              </Button>
            ) : (
              <div className="flex items-center gap-2">
                <Input value={inviteCode} readOnly className="font-mono text-lg tracking-widest" />
                <Button variant="outline" onClick={handleCopyCode}>
                  {copied ? t("teams.invite.copied") : t("teams.invite.copy")}
                </Button>
              </div>
            )}
          </div>
          <DialogFooter>
            <Button variant="outline" onClick={() => handleInviteOpenChange(false)}>
              {t("teams.invite.close")}
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>
    </RequireAuth>
  );
}