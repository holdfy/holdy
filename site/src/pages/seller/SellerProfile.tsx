import { useMemo, useState } from "react";
import { Link } from "react-router-dom";
import { Shield, ChevronRight, Key, Bell, HelpCircle, Store, LogOut, Loader2 } from "lucide-react";
import { useTranslation } from "react-i18next";
import { useMutation } from "@tanstack/react-query";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { LanguageSwitcher } from "@/components/LanguageSwitcher";
import { ReputationBadge } from "@/components/ReputationBadge";
import { tokenStore, api } from "@/lib/api-client";
import type { ApiError } from "@/lib/api-client";
import { toast } from "sonner";
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

export default function SellerProfile() {
  const { t } = useTranslation();
  const [openKey, setOpenKey] = useState<string | null>(null);
  const [pixKey, setPixKey] = useState("");
  const [phone, setPhone] = useState("");
  const userId = getUserIdFromToken();

  const pixMutation = useMutation({
    mutationFn: () => api.updatePixKey(pixKey.trim()),
    onSuccess: () => toast.success(t("profile.pixKeySaved", "Chave PIX salva com sucesso!")),
    onError: (err: ApiError) => toast.error(err?.error ?? t("profile.pixKeyError", "Erro ao salvar chave PIX")),
  });

  const phoneMutation = useMutation({
    mutationFn: () => api.updatePhone(phone.trim()),
    onSuccess: () => toast.success(t("profile.phoneSaved", "WhatsApp salvo com sucesso!")),
    onError: (err: ApiError) => toast.error(err?.error ?? t("profile.phoneError", "Erro ao salvar WhatsApp")),
  });

  const rows = useMemo(
    () => [
      { key: "store", label: t("profile.storeInfo"), body: t("profile.payoutSettings"), icon: Store },
      { key: "payout", label: t("profile.payoutSettings"), body: t("wallet.depositDesc"), icon: Key },
      { key: "notifications", label: t("profile.notifications"), body: t("notifications.n3Body"), icon: Bell },
      { key: "support", label: t("common.support"), body: t("auth.supportDesc"), icon: HelpCircle },
    ],
    [t],
  );

  const section = rows.find((r) => r.key === openKey);

  return (
    <div className="px-5 pt-6 space-y-5 md:px-0 md:pt-0">
      <div className="flex items-center justify-between gap-3">
        <h1 className="font-display text-2xl font-bold">{t("profile.sellerTitle")}</h1>
        <LanguageSwitcher variant="app" />
      </div>
      <div className="grid grid-cols-1 md:grid-cols-3 gap-5">
        <div className="md:col-span-1">
          <div className="bg-card rounded-2xl p-5 border border-border flex flex-col items-center gap-4 text-center">
            <div className="h-16 w-16 rounded-full vault-card flex items-center justify-center">
              <Store className="h-8 w-8 text-white" />
            </div>
            <div>
              <p className="font-semibold text-lg">TechStore</p>
              <p className="text-sm text-muted-foreground">seller@holdfy.com</p>
              {userId && <ReputationBadge userId={userId} className="mt-2" />}
            </div>
          </div>
        </div>
        <div className="md:col-span-2 space-y-4">
          <div className="bg-card rounded-2xl p-5 border border-border space-y-4">
            <div className="space-y-1.5">
              <Label htmlFor="seller-pix-key">
                {t("seller.pixKey", "Chave PIX para Recebimento")}
              </Label>
              <div className="flex gap-2">
                <Input
                  id="seller-pix-key"
                  placeholder={t("seller.pixKeyPlaceholder", "CPF, e-mail, celular ou chave aleatória")}
                  value={pixKey}
                  onChange={(e) => setPixKey(e.target.value)}
                  autoComplete="off"
                  className="flex-1"
                />
                <Button
                  size="sm"
                  onClick={() => pixKey.trim() && pixMutation.mutate()}
                  disabled={!pixKey.trim() || pixMutation.isPending}
                  className="vault-card border-0 text-white hover:opacity-90"
                >
                  {pixMutation.isPending ? <Loader2 className="h-4 w-4 animate-spin" /> : t("common.save", "Salvar")}
                </Button>
              </div>
              <p className="text-xs text-muted-foreground">
                {t("seller.pixKeyHint", "PIX enviado automaticamente após confirmação do comprador.")}
              </p>
            </div>
            <div className="space-y-1.5">
              <Label htmlFor="seller-phone">
                {t("profile.whatsappPhone", "WhatsApp")}
              </Label>
              <div className="flex gap-2">
                <Input
                  id="seller-phone"
                  placeholder={t("profile.phonePlaceholder", "+55 11 99999-9999")}
                  value={phone}
                  onChange={(e) => setPhone(e.target.value)}
                  type="tel"
                  className="flex-1"
                />
                <Button
                  size="sm"
                  onClick={() => phone.trim() && phoneMutation.mutate()}
                  disabled={!phone.trim() || phoneMutation.isPending}
                  className="vault-card border-0 text-white hover:opacity-90"
                >
                  {phoneMutation.isPending ? <Loader2 className="h-4 w-4 animate-spin" /> : t("common.save", "Salvar")}
                </Button>
              </div>
              <p className="text-xs text-muted-foreground">
                {t("profile.phoneHint", "Receba notificações de pedidos pelo WhatsApp.")}
              </p>
            </div>
          </div>
          <div className="bg-card rounded-2xl border border-border divide-y">
            {rows.map((item) => (
              <button
                key={item.key}
                type="button"
                onClick={() => setOpenKey(item.key)}
                className="w-full flex items-center justify-between p-4 text-sm font-medium hover:bg-muted/50 transition first:rounded-t-2xl last:rounded-b-2xl text-left"
              >
                <div className="flex items-center gap-3">
                  <item.icon className="h-4 w-4 text-muted-foreground" />
                  {item.label}
                </div>
                <ChevronRight className="h-4 w-4 text-muted-foreground" />
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
