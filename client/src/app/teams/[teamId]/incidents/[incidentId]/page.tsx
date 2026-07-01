import { IncidentDetailClient } from "./client";

export async function generateStaticParams() {
  return [{ teamId: "placeholder", incidentId: "placeholder" }];
}

export default function IncidentDetailPage() {
  return <IncidentDetailClient />;
}
