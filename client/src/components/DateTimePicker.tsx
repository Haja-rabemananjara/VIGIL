"use client";

import { useState } from "react";
import { format } from "date-fns";
import { fr, enUS } from "date-fns/locale";
import { CalendarIcon } from "lucide-react";
import { cn } from "@/lib/utils";
import { useAuth } from "@/stores/auth";
import { Button } from "@/components/ui/button";
import { Calendar } from "@/components/ui/calendar";
import {
  Popover,
  PopoverContent,
  PopoverTrigger,
} from "@/components/ui/popover";

interface DateTimePickerProps {
  value: string;
  onChange: (value: string) => void;
  id?: string;
}

function parseValue(value: string): {
  date: Date | undefined;
  hour: string;
  minute: string;
} {
  if (!value) return { date: undefined, hour: "12", minute: "00" };
  const d = new Date(value);
  if (isNaN(d.getTime())) return { date: undefined, hour: "12", minute: "00" };
  return {
    date: d,
    hour: String(d.getHours()).padStart(2, "0"),
    minute: String(d.getMinutes()).padStart(2, "0"),
  };
}

function buildValue(
  date: Date | undefined,
  hour: string,
  minute: string,
): string {
  if (!date) return "";
  const d = new Date(date);
  d.setHours(parseInt(hour, 10) || 0);
  d.setMinutes(parseInt(minute, 10) || 0);
  d.setSeconds(0);
  d.setMilliseconds(0);
  const pad = (n: number) => String(n).padStart(2, "0");
  return `${d.getFullYear()}-${pad(d.getMonth() + 1)}-${pad(d.getDate())}T${pad(d.getHours())}:${pad(d.getMinutes())}`;
}

export function DateTimePicker({ value, onChange, id }: DateTimePickerProps) {
  const { language } = useAuth();
  const locale = language === "fr" ? fr : enUS;
  const parsed = parseValue(value);
  const [open, setOpen] = useState(false);

  const hours = Array.from({ length: 24 }, (_, i) =>
    String(i).padStart(2, "0"),
  );
  const minutes = Array.from({ length: 12 }, (_, i) =>
    String(i * 5).padStart(2, "0"),
  );

  function handleDateSelect(day: Date | undefined) {
    if (!day) return;
    const newValue = buildValue(day, parsed.hour, parsed.minute);
    onChange(newValue);
  }

  function handleHourChange(h: string) {
    const newValue = buildValue(parsed.date ?? new Date(), h, parsed.minute);
    onChange(newValue);
  }

  function handleMinuteChange(m: string) {
    const newValue = buildValue(parsed.date ?? new Date(), parsed.hour, m);
    onChange(newValue);
  }

  const displayText = parsed.date
    ? format(parsed.date, language === "fr" ? "dd/MM/yyyy" : "MM/dd/yyyy", {
        locale,
      }) + ` ${parsed.hour}:${parsed.minute}`
    : language === "fr"
      ? "Choisir une date"
      : "Pick a date";

  return (
    <Popover open={open} onOpenChange={setOpen}>
      <PopoverTrigger asChild>
        <Button
          id={id}
          variant="outline"
          className={cn(
            "w-full justify-start text-left font-normal",
            !parsed.date && "text-muted-foreground",
          )}
        >
          <CalendarIcon className="mr-2 h-4 w-4" />
          {displayText}
        </Button>
      </PopoverTrigger>
      <PopoverContent className="w-auto p-0" align="start">
        <Calendar
          mode="single"
          selected={parsed.date}
          onSelect={handleDateSelect}
          locale={locale}
          disabled={(date) => date < new Date()}
        />
        <div className="flex items-center gap-2 border-t px-3 py-2">
          <select
            value={parsed.hour}
            onChange={(e) => handleHourChange(e.target.value)}
            className="rounded-md border bg-transparent px-2 py-1 text-sm"
          >
            {hours.map((h) => (
              <option key={h} value={h}>
                {h}
              </option>
            ))}
          </select>
          <span className="text-sm font-medium">:</span>
          <select
            value={parsed.minute}
            onChange={(e) => handleMinuteChange(e.target.value)}
            className="rounded-md border bg-transparent px-2 py-1 text-sm"
          >
            {minutes.map((m) => (
              <option key={m} value={m}>
                {m}
              </option>
            ))}
          </select>
        </div>
      </PopoverContent>
    </Popover>
  );
}
