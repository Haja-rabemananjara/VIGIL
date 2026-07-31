"use client";

import { useEffect, useState } from "react";
import { useRouter } from "next/navigation";
import { ArrowLeft } from "lucide-react";
import { RequireAuth } from "@/components/RequireAuth";
import { UserMenu } from "@/components/UserMenu";
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

export default function OnboardingPage() {
  const { user, token } = useAuth();
  const router = useRouter();

  const [hasTeams, setHasTeams] = useState(false);
  const [firstTeamId, setFirstTeamId] = useState<string | null>(null);

  const [createOpen, setCreateOpen] = useState(false);
  const [teamName, setTeamName] = useState("");
  const [createError, setCreateError] = useState("");
  const [createLoading, setCreateLoading] = useState(false);

  const [joinOpen, setJoinOpen] = useState(false);
  const [joinCode, setJoinCode] = useState("");
  const [joinError, setJoinError] = useState("");
  const [joinLoading, setJoinLoading] = useState(false);

  useEffect(() => {
    if (!token) return;
    api<TeamView[]>("/teams", { token })
      .then((teams) => {
        if (teams.length > 0) {
          setHasTeams(true);
          const lastTeamId = localStorage.getItem("vigil_last_team");
          const match = teams.find((t) => t.id === lastTeamId);
          setFirstTeamId(match?.id ?? teams[0].id);
        }
      })
      .catch(() => {});
  }, [token]);

  function handleBack() {
    if (firstTeamId) {
      router.push(`/teams/${firstTeamId}/incidents`);
    }
  }

  function handleCreateOpenChange(next: boolean) {
    setCreateOpen(next);
    if (!next) {
      setTeamName("");
      setCreateError("");
    }
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
        method: "POST",
        token,
        body: { name: trimmed },
      });
      handleCreateOpenChange(false);
      router.push(`/teams/${team.id}/incidents`);
    } catch (e) {
      setCreateError(e instanceof ApiError ? e.message : t("common.error"));
    } finally {
      setCreateLoading(false);
    }
  }

  function handleJoinOpenChange(next: boolean) {
    setJoinOpen(next);
    if (!next) {
      setJoinCode("");
      setJoinError("");
    }
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
      const result = await api<{
        team_id: string;
        team_name: string;
        role: string;
      }>("/teams/join", { method: "POST", token, body: { code: trimmed } });
      handleJoinOpenChange(false);
      router.push(`/teams/${result.team_id}/incidents`);
    } catch (e) {
      setJoinError(e instanceof ApiError ? e.message : t("common.error"));
    } finally {
      setJoinLoading(false);
    }
  }

  return (
    <RequireAuth>
      <header className="flex h-14 items-center justify-between border-b px-6">
        <div className="flex items-center gap-3">
          {hasTeams && (
            <Button
              variant="ghost"
              size="sm"
              onClick={handleBack}
              className="gap-2"
            >
              <ArrowLeft className="h-4 w-4" />
              {t("action.back")}
            </Button>
          )}
          <h1 className="text-lg font-semibold">{t("app.name")}</h1>
        </div>
        <UserMenu />
      </header>

      <main className="flex min-h-[calc(100vh-3.5rem)] items-center justify-center p-6">
        <div className="w-full max-w-md space-y-6">
          <div className="text-center">
            <h1 className="text-2xl font-semibold">
              {t("onboarding.welcome")}, {user?.display_name}
            </h1>
            <p className="mt-2 text-muted-foreground">
              {t("onboarding.subtitle")}
            </p>
          </div>

          <div className="grid gap-4">
            <Card>
              <CardHeader>
                <CardTitle>{t("onboarding.create.title")}</CardTitle>
                <CardDescription>{t("onboarding.create.desc")}</CardDescription>
              </CardHeader>
              <CardContent>
                <Button className="w-full" onClick={() => setCreateOpen(true)}>
                  {t("onboarding.create.action")}
                </Button>
              </CardContent>
            </Card>

            <Card>
              <CardHeader>
                <CardTitle>{t("onboarding.join.title")}</CardTitle>
                <CardDescription>{t("onboarding.join.desc")}</CardDescription>
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

      <Dialog open={createOpen} onOpenChange={handleCreateOpenChange}>
        <DialogContent>
          <DialogHeader>
            <DialogTitle>{t("teams.create.dialogTitle")}</DialogTitle>
            <DialogDescription>
              {t("teams.create.dialogDesc")}
            </DialogDescription>
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
            {createError && (
              <p className="text-sm text-destructive">{createError}</p>
            )}
          </div>
          <DialogFooter>
            <Button
              variant="outline"
              onClick={() => handleCreateOpenChange(false)}
            >
              {t("action.cancel")}
            </Button>
            <Button onClick={handleCreate} disabled={createLoading}>
              {createLoading ? "..." : t("action.create")}
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>

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
            {joinError && (
              <p className="text-sm text-destructive">{joinError}</p>
            )}
          </div>
          <DialogFooter>
            <Button
              variant="outline"
              onClick={() => handleJoinOpenChange(false)}
            >
              {t("action.cancel")}
            </Button>
            <Button onClick={handleJoin} disabled={joinLoading}>
              {joinLoading ? "..." : t("teams.join.submit")}
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>
    </RequireAuth>
  );
}
