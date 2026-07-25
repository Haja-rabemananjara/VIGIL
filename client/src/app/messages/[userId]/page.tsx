import { ConversationClient } from "./client";

export function generateStaticParams() {
  return [{ userId: "placeholder" }];
}

export default function ConversationPage() {
  return <ConversationClient />;
}
