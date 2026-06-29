import { TeamClient } from "./client";

export async function generateStaticParams() {
  return [{ teamId: "placeholder" }];
}

export default function TeamPage() {
  return <TeamClient />;
}