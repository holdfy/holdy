import { useState } from "react";
import { Shield, CheckCircle2, ArrowRight, FileText } from "lucide-react";
import { Link } from "react-router-dom";
import { useTranslation } from "react-i18next";
import { Button } from "@/components/ui/button";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog";
import { formatCurrency } from "@/lib/format";

export default function AppTransactionComplete() {
  const { t } = useTranslation();
  const [receiptOpen, setReceiptOpen] = useState(false);

  return (
    <div className="px-5 pt-6 space-y-5">
      <div className="flex items-center justify-between">
        <div className="flex items-center gap-3">
          <div className="h-10 w-10 rounded-full vault-card flex items-center justify-center">
            <Shield className="h-5 w-5 text-white" />
          </div>
          <span className="font-display font-bold text-lg">{t("common.holdfy")}</span>
        </div>
        <Link
          to="/buyer/profile"
          className="h-10 w-10 rounded-full bg-muted flex items-center justify-center"
          aria-label={t("common.profile")}
        >
          <span className="text-sm">👤</span>
        </Link>
      </div>

      <div className="text-center pt-8 pb-4">
        <div className="mx-auto h-20 w-20 rounded-full bg-secondary/15 flex items-center justify-center mb-5">
          <CheckCircle2 className="h-10 w-10 text-secondary" />
        </div>
        <p className="text-xs font-semibold tracking-wider text-secondary uppercase mb-2">{t("status.RELEASED")}</p>
        <h1 className="font-display text-3xl font-bold">{t("transactionComplete.title")}</h1>
        <p className="text-sm text-muted-foreground mt-3 max-w-xs mx-auto leading-relaxed">{t("transactionComplete.subtitle")}</p>
      </div>

      <div className="bg-card rounded-2xl p-5 border border-border">
        <div className="min-w-0">
          <p className="text-xs font-semibold tracking-wider text-muted-foreground uppercase">{t("order.stepReleased")}</p>
          <p className="text-3xl font-display font-bold mt-1">{formatCurrency(1240)}</p>
          <div className="flex items-center gap-1.5 mt-2 text-xs text-muted-foreground">
            <Shield className="h-3 w-3 flex-shrink-0" />
            <span className="truncate">{t("common.protectedPayment")} · PIX</span>
          </div>
        </div>
      </div>

      <div className="bg-secondary/10 rounded-2xl p-5 border border-secondary/20">
        <div className="flex items-center gap-2 mb-3">
          <CheckCircle2 className="h-5 w-5 text-secondary" />
          <span className="font-display font-bold">{t("order.protectedPayment")}</span>
        </div>
        <p className="text-sm text-muted-foreground leading-relaxed">{t("transactionComplete.subtitle")}</p>
        <div className="flex items-center gap-6 mt-4">
          <div>
            <p className="text-[10px] font-semibold tracking-wider text-secondary uppercase">{t("order.paymentStatus")}</p>
            <span className="text-xs font-semibold bg-secondary text-white px-2.5 py-0.5 rounded-full mt-1 inline-block">
              {t("status.RELEASED")}
            </span>
          </div>
          <div>
            <p className="text-[10px] font-semibold tracking-wider text-muted-foreground uppercase">Ref</p>
            <p className="text-sm font-semibold mt-1 font-mono">#8921-X</p>
          </div>
        </div>
      </div>

      <Button asChild className="w-full h-14 rounded-xl vault-card border-0 text-white font-semibold text-base hover:opacity-90">
        <Link to="/buyer/wallet">
          {t("transactionComplete.backHome")} <ArrowRight className="ml-2 h-4 w-4" />
        </Link>
      </Button>

      <button
        type="button"
        className="w-full flex items-center justify-center gap-2 py-3 text-sm text-muted-foreground font-medium hover:text-foreground transition"
        onClick={() => setReceiptOpen(true)}
      >
        <FileText className="h-4 w-4" />
        {t("transactionComplete.viewReceipt")}
      </button>

      <div className="h-4" />

      <Dialog open={receiptOpen} onOpenChange={setReceiptOpen}>
        <DialogContent>
          <DialogHeader>
            <DialogTitle>{t("transactionComplete.receiptTitle")}</DialogTitle>
            <DialogDescription>{t("transactionComplete.subtitle")}</DialogDescription>
          </DialogHeader>
          <dl className="space-y-2 text-sm">
            <div className="flex justify-between gap-4">
              <dt className="text-muted-foreground">{t("common.product")}</dt>
              <dd className="font-medium">TechStore Ltd.</dd>
            </div>
            <div className="flex justify-between gap-4">
              <dt className="text-muted-foreground">{t("navBuyer.orders")}</dt>
              <dd className="font-mono">#8921-X</dd>
            </div>
            <div className="flex justify-between gap-4">
              <dt className="text-muted-foreground">{t("payment.amount")}</dt>
              <dd className="font-semibold">{formatCurrency(1240)}</dd>
            </div>
          </dl>
        </DialogContent>
      </Dialog>
    </div>
  );
}
