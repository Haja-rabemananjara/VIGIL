"use client";

import { Button } from "@/components/ui/button";
import { t } from "@/lib/i18n";

interface Props {
  value: string;
  loading: boolean;
  onChange: (value: string) => void;
  onSubmit: () => void;
}

export function TimelineComposer({
  value,
  loading,
  onChange,
  onSubmit,
}: Props) {
  return (
    <div className="flex gap-2 pt-2">
      <textarea
        value={value}
        onChange={(e) => onChange(e.target.value)}
        placeholder={t("timeline.composer.placeholder")}
        rows={2}
        onKeyDown={(e) => {
          if (e.key === "Enter" && e.ctrlKey && !loading) {
            onSubmit();
          }
        }}
        className="flex-1 rounded-md border bg-background px-3 py-2 text-sm focus:outline-none focus-visible:ring-2 focus-visible:ring-ring"
      />
      <Button onClick={onSubmit} disabled={loading || !value.trim()}>
        {t("timeline.composer.submit")}
      </Button>
    </div>
  );
}
