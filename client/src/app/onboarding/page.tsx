"use client";

import { RequireAuth } from "@/components/RequireAuth";
import { useAuth } from "@/stores/auth";
import { t } from "@/lib/i18n";
import { Button } from "@/components/ui/button";
import { useState } from "react";
import { useRouter } from "next/navigation";
import { api, ApiError } from "@/lib/api";
import { saveLastTeam } from "@/lib/navigation";
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

  const [open, setOpen] = useState(false);
  const [teamName, setTeamName] = useState("");
  const [error, setError] = useState("");
  const [loading, setLoading] = useState(false);

  function handleOpenChange(next: boolean) {
    setOpen(next);
    if (!next) {
      setTeamName("");
      setError("");
    }
  }

  async function handleCreate() {
    const trimmed = teamName.trim();
    if (!trimmed) {
      setError(t("teams.create.error.empty"));
      return;
    }

    setLoading(true);
    setError("");

    try {
      const team = await api<TeamView>("/teams", {
        method: "POST",
        token,
        body: { name: trimmed },
      });

      saveLastTeam(team.id);

      router.replace("/onboarding");
    } catch (e) {
      if (e instanceof ApiError) {
        setError(e.message);
      } else {
        setError("Something went wrong");
      }
    } finally {
      setLoading(false);
    }
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

          <div className="grid gap-4">
            {/* Create a team (ACTIVE) */}
            <Card>
              <CardHeader>
                <CardTitle>{t("onboarding.create.title")}</CardTitle>
                <CardDescription>{t("onboarding.create.desc")}</CardDescription>
              </CardHeader>
              <CardContent>
                <Button className="w-full" onClick={() => setOpen(true)}>
                  {t("onboarding.create.action")}
                </Button>
              </CardContent>
            </Card>

            {/* Join a team (disabled) */}
            <Card className="opacity-60">
              <CardHeader>
                <CardTitle>{t("onboarding.join.title")}</CardTitle>
                <CardDescription>{t("onboarding.join.desc")}</CardDescription>
              </CardHeader>
              <CardContent>
                <Button className="w-full" variant="outline" disabled>
                  {t("onboarding.join.action")}
                </Button>
              </CardContent>
            </Card>
          </div>
        </div>
      </main>

      {/* Create team dialog */}
      <Dialog open={open} onOpenChange={handleOpenChange}>
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
                if (e.key === "Enter" && !loading) handleCreate();
              }}
            />
            {error && <p className="text-sm text-destructive">{error}</p>}
          </div>

          <DialogFooter>
            <Button variant="outline" onClick={() => handleOpenChange(false)}>
              {t("teams.create.cancel")}
            </Button>
            <Button onClick={handleCreate} disabled={loading}>
              {loading ? "…" : t("teams.create.submit")}
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>
    </RequireAuth>
  );
}
