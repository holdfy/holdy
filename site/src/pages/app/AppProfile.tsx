import { useMemo, useState } from "react";
import { Link } from "react-router-dom";
import { User, Shield, ChevronRight, LogOut, Loader2, AlertCircle } from "lucide-react";
import { useTranslation } from "react-i18next";
import { useMutation } from "@tanstack/react-query";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { LanguageSwitcher } from "@/components/LanguageSwitcher";
import { ReputationBadge } from "@/components/ReputationBadge";
import { tokenStore, api } from "@/lib/api-client";
import type { ApiError } from "@/lib/api-client";
import { maskPhone } from "@/lib/format";
import { toast } from "sonner";
import { useUserRole } from "@/contexts/UserRoleContext";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog";

export default function AppProfile() {
  const { t } = useTranslation();
  const { user } = useUserRole();
  const [openKey, setOpenKey] = useState<string | null>(null);
  const [phone, setPhone] = useState("");
  const [docInput, setDocInput] = useState("");
  const [docOpen, setDocOpen] = useState(false);
  const userId = user?.id ?? null;

  const phoneMutation = useMutation({
    mutationFn: () => api.updatePhone(phone.trim()),
    onSuccess: () => toast.success(t("profile.phoneSaved", "WhatsApp salvo com sucesso!")),
    onError: (err: ApiError) => toast.error(err?.error ?? t("profile.phoneError", "Erro ao salvar WhatsApp")),
  });

  const docMutation = useMutation({
    mutationFn: () => api.linkDocument(docInput.replace(/\D/g, "")),
    onSuccess: () => {
      toast.success(t("profile.docSaved", "Documento vinculado com sucesso!"));
      setDocOpen(false);
    },
    onError: (err: ApiError) => toast.error(err?.error ?? t("profile.docError", "Erro ao salvar documento")),
  });

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
            {user?.avatarUrl ? (
              <img
                src={user.avatarUrl}
                alt={user.name ?? "avatar"}
                className="h-16 w-16 rounded-full object-cover"
              />
            ) : (
              <div className="h-16 w-16 rounded-full vault-card flex items-center justify-center">
                <User className="h-8 w-8 text-white" />
              </div>
            )}
            <div>
              <p className="font-semibold text-lg">{user?.name ?? t("profile.displayName", "Nome")}</p>
              {user?.email && <p className="text-sm text-muted-foreground">{user.email}</p>}
              {userId && <ReputationBadge userId={userId} className="mt-2" />}
            </div>
          </div>

          {/* Banner: CPF/CNPJ pendente para usuários sociais */}
          {user && !user.hasDocument && (
            <button
              type="button"
              onClick={() => setDocOpen(true)}
              className="w-full flex items-start gap-3 bg-amber-500/10 border border-amber-500/30 rounded-2xl p-4 text-left hover:bg-amber-500/15 transition"
            >
              <AlertCircle className="h-5 w-5 text-amber-500 shrink-0 mt-0.5" />
              <div>
                <p className="text-sm font-semibold text-amber-700 dark:text-amber-400">
                  {t("auth.completeProfileTitle")}
                </p>
                <p className="text-xs text-amber-600 dark:text-amber-500 mt-0.5">
                  {t("profile.completeProfileBanner")}
                </p>
              </div>
            </button>
          )}
        </div>
        <div className="md:col-span-2 space-y-4">
          <div className="bg-card rounded-2xl p-5 border border-border space-y-3">
            <div className="space-y-1.5">
              <Label htmlFor="phone">
                {t("profile.whatsappPhone", "WhatsApp")}
              </Label>
              <div className="flex gap-2">
                <Input
                  id="phone"
                  placeholder={t("profile.phonePlaceholder", "+55 11 99999-9999")}
                  value={phone}
                  onChange={(e) => setPhone(maskPhone(e.target.value))}
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

      <Dialog open={docOpen} onOpenChange={setDocOpen}>
        <DialogContent>
          <DialogHeader>
            <DialogTitle>{t("auth.completeProfileTitle")}</DialogTitle>
            <DialogDescription>{t("auth.completeProfileDesc")}</DialogDescription>
          </DialogHeader>
          <div className="space-y-2 pt-2">
            <Label htmlFor="doc-input">{t("auth.cpfCnpjLabel", "CPF ou CNPJ")}</Label>
            <Input
              id="doc-input"
              inputMode="numeric"
              maxLength={18}
              placeholder="000.000.000-00"
              value={docInput}
              onChange={(e) => setDocInput(e.target.value)}
            />
          </div>
          <div className="flex justify-end gap-2 pt-2">
            <Button variant="outline" onClick={() => setDocOpen(false)}>{t("common.cancel")}</Button>
            <Button
              className="vault-card border-0 text-white hover:opacity-90"
              disabled={docMutation.isPending}
              onClick={() => docMutation.mutate()}
            >
              {docMutation.isPending ? <Loader2 className="h-4 w-4 animate-spin" /> : t("auth.completeProfileSave")}
            </Button>
          </div>
        </DialogContent>
      </Dialog>
    </div>
  );
}
