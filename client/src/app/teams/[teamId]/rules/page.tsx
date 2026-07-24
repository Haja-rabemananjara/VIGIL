import { RulesClient } from "./client";

export function generateStaticParams() {
  return [{ teamId: "placeholder" }];
}

export default function RulesPage() {
  return <RulesClient />;
}
