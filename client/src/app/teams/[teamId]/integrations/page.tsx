import { IntegrationsClient } from "./client";

export async function generateStaticParams() {
  return [{ teamId: "placeholder" }];
}

export default function IntegrationsPage() {
  return <IntegrationsClient />;
}
