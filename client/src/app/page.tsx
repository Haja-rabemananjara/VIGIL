"use client";

import { AppShell } from "@/components/AppShell";
import { StateBadge } from "@/components/StateBadge";
import { SeverityBadge } from "@/components/SeverityBadge";

export default function Home() {
  return (
    <AppShell>
      <div className="space-y-6">
        <h2 className="text-2xl font-semibold">UI Foundation Preview</h2>
        <section>
          <h3 className="mb-2 text-sm font-medium text-muted-foreground">Incident states</h3>
          <div className="flex flex-wrap gap-2">
            <StateBadge state="open" />
            <StateBadge state="acknowledged" />
            <StateBadge state="escalated" />
            <StateBadge state="resolved" />
          </div>
        </section>
        <section>
          <h3 className="mb-2 text-sm font-medium text-muted-foreground">Severities</h3>
          <div className="flex flex-wrap gap-2">
            <SeverityBadge severity="low" />
            <SeverityBadge severity="medium" />
            <SeverityBadge severity="high" />
            <SeverityBadge severity="critical" />
          </div>
        </section>
      </div>
    </AppShell>
  );
}