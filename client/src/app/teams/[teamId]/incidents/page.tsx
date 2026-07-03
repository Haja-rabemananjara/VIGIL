import { IncidentsClient } from "./client";

export async function generateStaticParams() {
  return [{ teamId: "placeholder" }];
}

export default function IncidentsPage() {
  return <IncidentsClient />;
}
