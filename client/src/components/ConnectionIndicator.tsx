import { Wifi, WifiOff, Loader2 } from "lucide-react";
import { cn } from "@/lib/utils";
import { t } from "@/lib/i18n";
import { useAuth } from "@/stores/auth";
import { type ConnectionStatus } from "@/stores/socket";

interface Props {
  status: ConnectionStatus;
}

export function ConnectionIndicator({ status }: Props) {
  const { language } = useAuth();
  void language;

  const config: Record<
    ConnectionStatus,
    { icon: typeof Wifi; label: string; className: string }
  > = {
    connected: {
      icon: Wifi,
      label: t("ws.connected"),
      className: "text-success",
    },
    connecting: {
      icon: Loader2,
      label: t("ws.connecting"),
      className: "text-warning animate-spin",
    },
    disconnected: {
      icon: WifiOff,
      label: t("ws.disconnected"),
      className: "text-destructive",
    },
  };

  const { icon: Icon, label, className } = config[status];
  return (
    <span
      className={cn(
        "inline-flex items-center gap-1.5 text-xs font-medium",
        className,
      )}
      title={label}
    >
      <Icon className="h-3.5 w-3.5" aria-hidden="true" />
      <span className="hidden sm:inline">{label}</span>
    </span>
  );
}
