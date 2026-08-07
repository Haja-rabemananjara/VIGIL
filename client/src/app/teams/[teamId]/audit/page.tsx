import { AuditClient } from "./client";

export async function generateStaticParams() {
  return [{ teamId: "placeholder" }];
}

export default function AuditPage() {
  return <AuditClient />;
}
