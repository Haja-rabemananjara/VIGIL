import { ReleasesClient } from "./client";

export function generateStaticParams() {
  return [{ teamId: "placeholder" }];
}

export default function ReleasesPage() {
  return <ReleasesClient />;
}
