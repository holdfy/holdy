import { useMemo, useState } from "react";
import { Link } from "react-router-dom";
import { User, Shield, ChevronRight, LogOut } from "lucide-react";
import { useTranslation } from "react-i18next";
import { Button } from "@/components/ui/button";
import { LanguageSwitcher } from "@/components/LanguageSwitcher";
import { ReputationBadge } from "@/components/ReputationBadge";
import { tokenStore } from "@/lib/api-client";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog";

function getUserIdFromToken(): string | null {
  const token = tokenStore.getAccess();
  if (!token) return null;
  try {
    const payload = JSON.parse(atob(token.split(".")[1]));
    return payload.sub ?? null;
  } catch {
    return null;
  }
}

export default function AppProfile() {
  const { t } = useTranslation();
  const [openKey, setOpenKey] = useState<string | null>(null);
  const userId = getUserIdFromToken();

  const sections = useMemo(
    () => [
      { key: "security", label: t("profile.security"), body: t("profile.twoFactor") },
      { key: "notifications", label: t("profile.notifications"), body: t("notifications.n1Body") },
      { key: "language", label: t("profile.language"), body: t("lang.label") },
    ],
    [t],
  );

  const section = sections.find((s) => s.key === openKey);

  return (
    <div className="px-5 pt-6 space-y-5 md:px-0 md:pt-0">
      <div className="flex items-center justify-between gap-3">
        <h1 className="font-display text-2xl font-bold">{t("profile.buyerTitle")}</h1>
        <LanguageSwitcher variant="app" />
      </div>
      <div className="grid grid-cols-1 md:grid-cols-3 gap-5">
        <div className="md:col-span-1">
          <div className="bg-card rounded-2xl p-5 border border-border flex flex-col items-center gap-4 text-center">
            <div className="h-16 w-16 rounded-full vault-card flex items-center justify-center">
              <User className="h-8 w-8 text-white" />
            </div>
            <div>
              <p className="font-semibold text-lg">Demo User</p>
              <p className="text-sm text-muted-foreground">demo@holdfy.com</p>
              {userId && <ReputationBadge userId={userId} className="mt-2" />}
            </div>
          </div>
        </div>
        <div className="md:col-span-2">
          <div className="bg-card rounded-2xl border border-border divide-y">
            {sections.map((item) => (
              <button
                key={item.key}
                type="button"
                onClick={() => setOpenKey(item.key)}
                className="w-full flex items-center justify-between p-4 text-sm font-medium hover:bg-muted/50 transition first:rounded-t-2xl last:rounded-b-2xl text-left"
              >
                {item.label}
                <ChevronRight className="h-4 w-4 text-muted-foreground flex-shrink-0" />
              </button>
            ))}
          </div>
        </div>
      </div>

      <div className="md:hidden">
        <Button
          variant="outline"
          className="h-12 w-full rounded-xl border-destructive/25 text-destructive hover:bg-destructive/5 hover:text-destructive"
          asChild
        >
          <Link to="/login" className="flex items-center justify-center gap-2">
            <LogOut className="h-4 w-4" />
            {t("common.logout")}
          </Link>
        </Button>
      </div>

      <div className="flex items-center justify-center gap-2 py-4 text-xs text-muted-foreground">
        <Shield className="h-3.5 w-3.5" />
        <span className="tracking-wider uppercase font-medium">{t("common.protectedPayment")}</span>
      </div>

      <Dialog open={!!openKey} onOpenChange={(o) => !o && setOpenKey(null)}>
        <DialogContent>
          <DialogHeader>
            <DialogTitle>{section?.label}</DialogTitle>
            <DialogDescription className="text-left leading-relaxed pt-2">{section?.body}</DialogDescription>
          </DialogHeader>
        </DialogContent>
      </Dialog>
    </div>
  );
}
