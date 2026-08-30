"use client";

import { Button } from "@/components/ui/button";
import { t, TranslationKey } from "@/lib/i18n";

interface Member {
  user_id: string;
  display_name: string;
  role: string;
}

interface Props {
  open: boolean;
  eligibleMembers: Member[];
  loading: boolean;
  error: string;
  onAssign: (userId: string) => void;
  onClose: () => void;
}

export function AssignDialog({
  open,
  eligibleMembers,
  loading,
  error,
  onAssign,
  onClose,
}: Props) {
  if (!open) return null;

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/50">
      <div className="w-full max-w-sm space-y-4 rounded-lg border bg-card p-6 shadow-lg">
        <h3 className="font-semibold">{t("incidents.assign.dialogTitle")}</h3>
        <p className="text-sm text-muted-foreground">
          {t("incidents.assign.dialogDesc")}
        </p>
        {eligibleMembers.length === 0 ? (
          <p className="text-sm text-muted-foreground">
            {t("incidents.assign.noEligible")}
          </p>
        ) : (
          <div className="space-y-2">
            {eligibleMembers.map((m) => (
              <button
                key={m.user_id}
                onClick={() => onAssign(m.user_id)}
                disabled={loading}
                className="w-full rounded-md border px-4 py-2 text-left text-sm hover:bg-muted focus:outline-none focus-visible:ring-2 focus-visible:ring-ring"
              >
                {m.display_name}{" "}
                <span className="text-muted-foreground">
                  ({t(`members.role.${m.role}` as TranslationKey)})
                </span>
              </button>
            ))}
          </div>
        )}
        {error && <p className="text-sm text-destructive">{error}</p>}
        <div className="flex justify-end">
          <Button variant="outline" onClick={onClose}>
            {t("action.cancel")}
          </Button>
        </div>
      </div>
    </div>
  );
}
