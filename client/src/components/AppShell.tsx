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
  const pathname = usePathname();
  const [teams, setTeams] = useState<TeamView[]>([]);

  // Active team id read from the URL: /teams/{id}/...
  const activeTeamId =
    pathname?.startsWith("/teams/") ? pathname.split("/")[2] : null;

  useEffect(() => {
    if (!token) return;
    api<TeamView[]>("/teams", { token })
      .then(setTeams)
      .catch(() => {});
  }, [token]);

  return (
    <div className="flex h-screen flex-col">
      <header className="flex h-14 items-center justify-between border-b px-6">
        <h1 className="text-lg font-semibold">{t("app.name")}</h1>
        <UserMenu />
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