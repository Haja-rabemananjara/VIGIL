"use client";

import type { ReactNode } from "react";
import { UserMenu } from "./UserMenu";
import { t } from "@/lib/i18n";

interface AppShellProps {
  children: ReactNode;
}

export function AppShell({ children }: AppShellProps) {
  return (
    <div className="flex h-screen flex-col">
      <header className="flex h-14 items-center justify-between border-b px-6">
        <h1 className="text-lg font-semibold">{t("app.name")}</h1>
        <UserMenu />
      </header>
      <div className="flex flex-1 overflow-hidden">
        <aside className="w-60 border-r bg-muted/30 p-4">
          {/* Teams sidebar will be populated in VGL-020 */}
          <p className="text-sm text-muted-foreground">
            {t("app.shell.noTeamsYet")}
          </p>
        </aside>
        <main className="flex-1 overflow-auto p-6">{children}</main>
      </div>
    </div>
  );
}
