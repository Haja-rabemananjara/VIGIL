"use client";

import { UserMenu } from "@/components/UserMenu";
import { t } from "@/lib/i18n";
import { RequireAuth } from "@/components/RequireAuth";

export default function SettingsLayout({
  children,
}: {
  children: React.ReactNode;
}) {
  return (
    <RequireAuth>
      <div className="flex h-screen flex-col">
        <header className="flex h-14 items-center justify-between border-b px-6">
          <h1 className="text-lg font-semibold">{t("app.name")}</h1>
          <div className="flex items-center gap-3">
            <UserMenu />
          </div>
        </header>
        <main className="flex-1 overflow-auto">{children}</main>
      </div>
    </RequireAuth>
  );
}
