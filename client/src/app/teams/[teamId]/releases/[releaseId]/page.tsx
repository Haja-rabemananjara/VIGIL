import { ReleaseDetailClient } from "./client";

export function generateStaticParams() {
  return [{ teamId: "placeholder", releaseId: "placeholder" }];
}

export default function ReleaseDetailPage() {
  return <ReleaseDetailClient />;
}