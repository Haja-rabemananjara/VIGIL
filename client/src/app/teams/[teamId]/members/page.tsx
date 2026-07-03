import { MembersClient } from "./client";

export async function generateStaticParams() {
  return [{ teamId: "placeholder" }];
}

export default function MembersPage() {
  return <MembersClient />;
}
