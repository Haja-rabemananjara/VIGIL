"use client";

import { useState } from "react";
import { useAuth } from "@/stores/auth";
import { api, ApiError } from "@/lib/api";
import { t, type TranslationKey } from "@/lib/i18n";
import type { Language } from "@/lib/i18n";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import type { User } from "@/stores/auth";
import { Eye, EyeOff, Check, ArrowLeft } from "lucide-react";
import { useRouter } from "next/navigation";
import { UserAvatar } from "@/components/UserAvatar";

const AVATAR_SEEDS = [
  "avatar-01",
  "avatar-02",
  "avatar-03",
  "avatar-04",
  "avatar-05",
  "avatar-06",
  "avatar-07",
  "avatar-08",
  "avatar-09",
  "avatar-10",
  "avatar-11",
  "avatar-12",
  "avatar-13",
  "avatar-14",
  "avatar-15",
  "avatar-16",
  "avatar-17",
  "avatar-18",
  "avatar-19",
  "avatar-20",
  "avatar-21",
  "avatar-22",
  "avatar-23",
  "avatar-24",
  "avatar-25",
  "avatar-26",
  "avatar-27",
  "avatar-28",
  "avatar-29",
  "avatar-30",
  "avatar-31",
  "avatar-32",
  "avatar-33",
  "avatar-34",
  "avatar-35",
  "avatar-36",
  "avatar-37",
  "avatar-38",
  "avatar-39",
  "avatar-40",
  "avatar-41",
  "avatar-42",
];

export function ProfileClient() {
  const { user, token, language, changeLanguage, updateUser } = useAuth();

  const [displayName, setDisplayName] = useState(user?.display_name ?? "");
  const [nameSuccess, setNameSuccess] = useState(false);
  const [nameError, setNameError] = useState("");
  const [namePending, setNamePending] = useState(false);

  const [newPassword, setNewPassword] = useState("");
  const [showPassword, setShowPassword] = useState(false);
  const [pwSuccess, setPwSuccess] = useState(false);
  const [pwError, setPwError] = useState("");
  const [pwPending, setPwPending] = useState(false);

  const [langPending, setLangPending] = useState(false);

  const [selectedSeed, setSelectedSeed] = useState<string | null>(
    user?.avatar_seed ?? null,
  );
  const [avatarPending, setAvatarPending] = useState(false);
  const [avatarSuccess, setAvatarSuccess] = useState(false);

  if (!user) return null;

  // eslint-disable-next-line react-hooks/rules-of-hooks
  const router = useRouter();

  async function handleNameSave() {
    const trimmed = displayName.trim();
    if (!trimmed) {
      setNameError(t("teams.create.error.empty" as TranslationKey));
      return;
    }
    setNameError("");
    setNameSuccess(false);
    setNamePending(true);
    try {
      const updated = await api<User>("/me", {
        method: "PATCH",
        token,
        body: { display_name: trimmed },
      });
      setDisplayName(updated.display_name);
      updateUser({ display_name: updated.display_name });
      setNameSuccess(true);
      setTimeout(() => setNameSuccess(false), 2000);
    } catch (e) {
      setNameError(e instanceof ApiError ? e.message : t("common.error"));
    } finally {
      setNamePending(false);
    }
  }

  async function handlePasswordSave() {
    if (newPassword.length < 8) {
      setPwError(t("auth.error.passwordTooShort"));
      return;
    }
    setPwError("");
    setPwSuccess(false);
    setPwPending(true);
    try {
      await api<User>("/me", {
        method: "PATCH",
        token,
        body: { password: newPassword },
      });
      setNewPassword("");
      setPwSuccess(true);
      setTimeout(() => setPwSuccess(false), 2000);
    } catch (e) {
      setPwError(e instanceof ApiError ? e.message : t("common.error"));
    } finally {
      setPwPending(false);
    }
  }

  async function handleAvatarSave() {
    setAvatarPending(true);
    setAvatarSuccess(false);
    try {
      await api<User>("/me", {
        method: "PATCH",
        token,
        body: { avatar_seed: selectedSeed },
      });
      updateUser({ avatar_seed: selectedSeed });
      setAvatarSuccess(true);
      setTimeout(() => setAvatarSuccess(false), 2000);
    } catch {
      // silent
    } finally {
      setAvatarPending(false);
    }
  }

  async function handleLanguageChange(lang: Language) {
    setLangPending(true);
    try {
      await api<User>("/me", {
        method: "PATCH",
        token,
        body: { language: lang },
      });
      changeLanguage(lang);
    } catch {
    } finally {
      setLangPending(false);
    }
  }

  return (
    <div className="mx-auto max-w-2xl space-y-4 p-6">
      <Button
        variant="ghost"
        size="sm"
        onClick={() => router.back()}
        className="gap-1.5"
      >
        <ArrowLeft className="h-4 w-4" />
        {t("action.back")}
      </Button>
      <div>
        <h1 className="text-2xl font-semibold">{t("user.profile")}</h1>
        <p className="text-sm text-muted-foreground">{user.email}</p>
      </div>

      {/* Avatar */}
      <Card>
        <CardHeader>
          <CardTitle>Avatar</CardTitle>
        </CardHeader>
        <CardContent className="space-y-4">
          <div className="flex items-center gap-4">
            <UserAvatar
              seed={selectedSeed}
              displayName={user.display_name}
              size={64}
            />
            <div className="text-sm text-muted-foreground">
              {selectedSeed
                ? t("profile.avatarSelected" as TranslationKey)
                : t("profile.avatarDefault" as TranslationKey)}
            </div>
          </div>

          <div className="grid grid-cols-7 gap-2">
            {AVATAR_SEEDS.map((seed) => (
              <button
                key={seed}
                type="button"
                onClick={() => setSelectedSeed(seed)}
                className={`rounded-full p-0.5 transition-all ${
                  selectedSeed === seed
                    ? "ring-2 ring-primary ring-offset-2"
                    : "hover:ring-2 hover:ring-muted-foreground hover:ring-offset-1"
                }`}
              >
                <UserAvatar seed={seed} displayName="" size={40} />
              </button>
            ))}
          </div>

          <div className="flex gap-2">
            <Button
              onClick={handleAvatarSave}
              disabled={avatarPending || selectedSeed === user.avatar_seed}
            >
              {avatarSuccess ? <Check className="h-4 w-4" /> : t("action.save")}
            </Button>
            {selectedSeed && (
              <Button variant="outline" onClick={() => setSelectedSeed(null)}>
                {t("profile.removeAvatar" as TranslationKey)}
              </Button>
            )}
          </div>
        </CardContent>
      </Card>

      {/* Display name */}
      <Card>
        <CardHeader>
          <CardTitle>{t("auth.signup.displayName")}</CardTitle>
        </CardHeader>
        <CardContent className="space-y-3">
          <div className="space-y-2">
            <Label htmlFor="display-name">{t("auth.signup.displayName")}</Label>
            <div className="flex gap-2">
              <Input
                id="display-name"
                value={displayName}
                onChange={(e) => setDisplayName(e.target.value)}
              />
              <Button
                onClick={handleNameSave}
                disabled={
                  namePending || displayName.trim() === user.display_name
                }
              >
                {nameSuccess ? <Check className="h-4 w-4" /> : t("action.save")}
              </Button>
            </div>
          </div>
          {nameError && <p className="text-sm text-destructive">{nameError}</p>}
        </CardContent>
      </Card>

      {/* Language */}
      <Card>
        <CardHeader>
          <CardTitle>{t("user.language")}</CardTitle>
        </CardHeader>
        <CardContent>
          <div className="flex gap-2">
            <Button
              variant={language === "en" ? "default" : "outline"}
              onClick={() => handleLanguageChange("en")}
              disabled={langPending || language === "en"}
            >
              English
            </Button>
            <Button
              variant={language === "fr" ? "default" : "outline"}
              onClick={() => handleLanguageChange("fr")}
              disabled={langPending || language === "fr"}
            >
              Français
            </Button>
          </div>
        </CardContent>
      </Card>

      {/* Password */}
      <Card>
        <CardHeader>
          <CardTitle>{t("auth.signin.password")}</CardTitle>
        </CardHeader>
        <CardContent className="space-y-3">
          <div className="space-y-2">
            <Label htmlFor="new-password">
              {t("profile.newPassword" as TranslationKey)}
            </Label>
            <div className="flex gap-2">
              <div className="relative flex-1">
                <Input
                  id="new-password"
                  type={showPassword ? "text" : "password"}
                  value={newPassword}
                  onChange={(e) => setNewPassword(e.target.value)}
                  className="pr-10"
                />
                <button
                  type="button"
                  onClick={() => setShowPassword(!showPassword)}
                  className="absolute right-2 top-1/2 -translate-y-1/2 text-muted-foreground hover:text-foreground"
                  aria-label={
                    showPassword
                      ? t("auth.hidePassword")
                      : t("auth.showPassword")
                  }
                >
                  {showPassword ? (
                    <EyeOff className="h-4 w-4" />
                  ) : (
                    <Eye className="h-4 w-4" />
                  )}
                </button>
              </div>
              <Button
                onClick={handlePasswordSave}
                disabled={pwPending || !newPassword}
              >
                {pwSuccess ? <Check className="h-4 w-4" /> : t("action.save")}
              </Button>
            </div>
          </div>
          {pwError && <p className="text-sm text-destructive">{pwError}</p>}
        </CardContent>
      </Card>
    </div>
  );
}
