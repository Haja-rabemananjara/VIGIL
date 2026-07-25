"use client";

import type { ReactNode } from "react";
import { RequireAuth } from "@/components/RequireAuth";
import { AppShell } from "@/components/AppShell";
import { VigilSocketProvider } from "@/stores/socket";

export default function TeamLayout({ children }: { children: ReactNode }) {
  return (
    <RequireAuth>
      <VigilSocketProvider>
        <AppShell>{children}</AppShell>
      </VigilSocketProvider>
    </RequireAuth>
  );
}
