"use client";

import { LogOut, User as UserIcon, Languages } from "lucide-react";
import { useAuth } from "@/stores/auth";
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuLabel,
  DropdownMenuSeparator,
  DropdownMenuTrigger,
} from "@/components/ui/dropdown-menu";
import { t } from "@/lib/i18n";
import Link from "next/link";
import { UserAvatar } from "./UserAvatar";

export function UserMenu() {
  const { user, signout } = useAuth();

  if (!user) return null;

  // eslint-disable-next-line react-hooks/rules-of-hooks
  const { language, changeLanguage } = useAuth();

  return (
    <DropdownMenu>
      <DropdownMenuTrigger className="outline-none focus-visible:ring-2 focus-visible:ring-ring rounded-full">
        <UserAvatar
          seed={user.avatar_seed}
          displayName={user.display_name}
          size={36}
          className="cursor-pointer"
        />
      </DropdownMenuTrigger>
      <DropdownMenuContent align="end" className="w-56">
        <DropdownMenuLabel className="font-normal">
          <div className="flex flex-col space-y-1">
            <p className="text-sm font-medium leading-none">
              {user.display_name}
            </p>
            <p className="text-xs leading-none text-muted-foreground">
              {user.email}
            </p>
          </div>
        </DropdownMenuLabel>
        <DropdownMenuSeparator />
        <DropdownMenuItem asChild>
          <Link href="/settings/profile">
            <UserIcon className="mr-2 h-4 w-4" aria-hidden="true" />
            <span>{t("user.profile")}</span>
          </Link>
        </DropdownMenuItem>
        <DropdownMenuItem
          onClick={() => changeLanguage(language === "en" ? "fr" : "en")}
        >
          <Languages className="mr-2 h-4 w-4" aria-hidden="true" />
          <span>{language === "en" ? "Français" : "English"}</span>
        </DropdownMenuItem>
        <DropdownMenuSeparator />
        <DropdownMenuItem onClick={() => signout()}>
          <LogOut className="mr-2 h-4 w-4" aria-hidden="true" />
          <span>{t("auth.signout.label")}</span>
        </DropdownMenuItem>
      </DropdownMenuContent>
    </DropdownMenu>
  );
}