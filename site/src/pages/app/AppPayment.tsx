import { useState } from "react";
import { Shield, ArrowLeft, Copy, Clock, Lock, Zap, HelpCircle } from "lucide-react";
import { Link } from "react-router-dom";
import { useTranslation } from "react-i18next";
import { Button } from "@/components/ui/button";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog";
import { formatCurrency } from "@/lib/format";
import { toast } from "sonner";

export default function AppPayment() {
  const { t } = useTranslation();
  const [helpOpen, setHelpOpen] = useState(false);

  return (
    <div className="px-5 pt-6 space-y-5">
      <div className="flex items-center justify-between">
        <div className="flex items-center gap-3">
          <Link to="/buyer" className="h-10 w-10 rounded-full bg-muted flex items-center justify-center">
            <ArrowLeft className="h-5 w-5" />
          </Link>
          <span className="font-display font-bold text-lg">{t("common.holdfy")}</span>
        </div>
        <div className="h-10 w-10 rounded-full vault-card flex items-center justify-center">
          <Shield className="h-5 w-5 text-white" />
        </div>
      </div>

      <div>
        <p className="text-xs font-semibold tracking-wider text-muted-foreground uppercase mb-1">{t("payment.title")}</p>
        <h1 className="font-display text-2xl font-bold">{t("payment.title")}</h1>
        <p className="text-sm text-muted-foreground mt-1">{t("payment.helpDesc")}</p>
      </div>

      <div className="bg-card rounded-2xl p-6 border border-border text-center space-y-5">
        <div>
          <p className="text-xs text-muted-foreground font-medium uppercase tracking-wider">{t("payment.amount")}</p>
          <p className="text-4xl font-display font-bold mt-1">{formatCurrency(250)}</p>
        </div>

        <div className="mx-auto w-48 h-48 rounded-2xl vault-card flex items-center justify-center">
          <div className="text-center text-white/60">
            <div className="grid grid-cols-3 gap-1.5 mx-auto w-20">
              {Array.from({ length: 9 }).map((_, i) => (
                <div
                  key={i}
                  className={`h-5 w-5 rounded-sm ${i % 3 === 0 ? "bg-white/80" : i % 2 === 0 ? "bg-white/40" : "bg-white/20"}`}
                />
              ))}
            </div>
            <p className="text-[10px] mt-2 font-mono">PIX</p>
          </div>
        </div>

        <Button
          className="w-full h-12 rounded-xl vault-card border-0 text-white font-semibold hover:opacity-90"
          onClick={() => toast.success(t("payment.copied"))}
        >
          <Copy className="mr-2 h-4 w-4" />
          {t("payment.copyPaste")}
        </Button>
      </div>

      <div className="bg-card rounded-2xl p-4 border border-border flex items-center gap-3">
        <div className="h-10 w-10 rounded-full bg-muted flex items-center justify-center flex-shrink-0">
          <Clock className="h-5 w-5 text-muted-foreground animate-pulse" />
        </div>
        <div>
          <p className="font-semibold text-sm">{t("payment.confirm")}</p>
          <p className="text-xs text-muted-foreground">{t("payment.helpDesc")}</p>
        </div>
      </div>

      <div className="bg-card rounded-2xl p-4 border border-border flex items-start gap-3">
        <div className="h-10 w-10 rounded-xl bg-secondary/10 flex items-center justify-center flex-shrink-0">
          <Lock className="h-5 w-5 text-secondary" />
        </div>
        <div>
          <p className="font-semibold text-sm">{t("order.inCustody")}</p>
          <p className="text-xs text-muted-foreground mt-0.5 leading-relaxed">{t("buyer.protectedDesc")}</p>
        </div>
      </div>

      <div className="flex items-center justify-center gap-2 py-2 text-xs text-muted-foreground">
        <button type="button" className="inline-flex items-center gap-2 hover:text-foreground transition" onClick={() => setHelpOpen(true)}>
          <HelpCircle className="h-3.5 w-3.5" />
          <span>{t("payment.help")}</span>
        </button>
      </div>

      <div className="h-4" />

      <Dialog open={helpOpen} onOpenChange={setHelpOpen}>
        <DialogContent>
          <DialogHeader>
            <DialogTitle>{t("payment.helpTitle")}</DialogTitle>
            <DialogDescription>{t("payment.helpDesc")}</DialogDescription>
          </DialogHeader>
          <DialogFooter>
            <Button type="button" onClick={() => setHelpOpen(false)}>
              {t("common.close")}
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>
    </div>
  );
}
