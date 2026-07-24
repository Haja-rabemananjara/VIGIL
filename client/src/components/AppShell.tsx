"use client";

import type { ReactNode } from "react";
import { UserMenu } from "./UserMenu";
import { t } from "@/lib/i18n";
import { useEffect, useState } from "react";
import Link from "next/link";
import { usePathname } from "next/navigation";
import { useAuth } from "@/stores/auth";
import { api } from "@/lib/api";
import { cn } from "@/lib/utils";
import { useVigilSocket } from "@/stores/socket";
import { ConnectionIndicator } from "./ConnectionIndicator"
import { useNotifications } from "@/lib/useNotifications";
// eslint-disable-next-line @typescript-eslint/no-unused-vars
import { useRouter } from "next/router";

interface AppShellProps {
  children: ReactNode;
}

interface TeamView {
  id: string;
  name: string;
  role: string;
  created_at: string;
}

export function AppShell({ children }: AppShellProps) {
  const { token } = useAuth();
  useNotifications();

  const pathname = usePathname();
  const [teams, setTeams] = useState<TeamView[]>([]);
  const { status, lastEvent } = useVigilSocket();
  const { user } = useAuth();


  const activeTeamId = pathname?.startsWith("/teams/")
    ? pathname.split("/")[2]
    : null;

  useEffect(() => {
    if (!token) return;
    api<TeamView[]>("/teams", { token })
      .then(setTeams)
      .catch(() => { });
  }, [token]);

  useEffect(() => {
    if (!lastEvent) return;

    if (
      (lastEvent.type === "member_kicked" || lastEvent.type === "member_banned") &&
      (lastEvent.user_id as string) === user?.id
    ) {
      const removedTeamId = lastEvent.team_id as string;

      // eslint-disable-next-line react-hooks/set-state-in-effect
      setTeams((prev) => {
        const remaining = prev.filter((t) => t.id !== removedTeamId);

        if (activeTeamId === removedTeamId) {
          if (remaining.length > 0) {
            window.location.href = `/teams/${remaining[0].id}/incidents`;
          } else {
            window.location.href = "/onboarding";
          }
        }

        return remaining;
      });
    }
  }, [lastEvent, user?.id, activeTeamId]);

  return (
    <div className="flex h-screen flex-col">
      <header className="flex h-14 items-center justify-between border-b px-6">
        <h1 className="text-lg font-semibold">{t("app.name")}</h1>
        <div className="flex items-center gap-3">
          <ConnectionIndicator status={status} />
          <UserMenu />
        </div>
      </header>
      <div className="flex flex-1 overflow-hidden">
        <aside className="flex w-60 flex-col border-r bg-muted/30 p-4">
          <p className="mb-2 px-2 text-xs font-medium uppercase tracking-wide text-muted-foreground">
            {t("onboarding.myTeams")}
          </p>
          <nav className="flex-1 space-y-1 overflow-auto">
            {teams.length === 0 ? (
              <p className="px-2 text-sm text-muted-foreground">
                {t("app.shell.noTeamsYet")}
              </p>
            ) : (
              teams.map((team) => (
                <div key={team.id} className="space-y-0.5">
                  <Link
                    key={team.id}
                    href={`/teams/${team.id}/incidents`}
                    className={cn(
                      "block rounded-md px-3 py-2 text-sm transition-colors",
                      team.id === activeTeamId
                        ? "bg-primary/10 font-medium text-primary"
                        : "hover:bg-muted",
                    )}
                  >
                    {team.name}
                  </Link>
                  {team.id === activeTeamId && (
                    <>
                      <Link
                        href={`/teams/${team.id}/members`}
                        className={cn(
                          "block rounded-md px-3 py-1.5 pl-6 text-xs transition-colors",
                          pathname?.includes("/members")
                            ? "font-medium text-primary"
                            : "text-muted-foreground hover:bg-muted",
                        )}
                      >
                        {t("app.shell.members")}
                      </Link>
                      <Link
                        href={`/teams/${team.id}/releases`}
                        className={cn(
                          "block rounded-md px-3 py-1.5 pl-6 text-xs transition-colors",
                          pathname?.includes("/releases")
                            ? "font-medium text-primary"
                            : "text-muted-foreground hover:bg-muted",
                        )}
                      >
                        {t("app.shell.releases")}
                      </Link>
                      <Link
                        href={`/teams/${team.id}/rules`}
                        className={cn(
                          "block rounded-md px-3 py-1.5 pl-6 text-xs transition-colors",
                          pathname?.includes("/rules")
                            ? "font-medium text-primary"
                            : "text-muted-foreground hover:bg-muted",
                        )}
                      >
                        {t("app.shell.rules")}
                      </Link>
                    </>
                  )}
                </div>
              ))
            )}
          </nav>
          <Link
            href="/onboarding"
            className="mt-2 rounded-md border px-3 py-2 text-center text-sm hover:bg-muted"
          >
            {t("app.shell.addTeam")}
          </Link>
        </aside>
        <main className="flex-1 overflow-auto">{children}</main>
      </div>
    </div>
  );
}
