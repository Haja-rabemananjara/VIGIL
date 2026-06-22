"use client";

import { RequireAuth } from "@/components/RequireAuth";
import { AppShell } from "@/components/AppShell";

export default function Home() {
  return (
    <RequireAuth>
      <AppShell>
        <div className="space-y-6">
          <h2 className="text-2xl font-semibold">Dashboard</h2>
          <p className="text-muted-foreground">You are signed in.</p>
        </div>
      </AppShell>
    </RequireAuth>
  );
}