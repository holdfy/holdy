import { Wallet, ArrowRight, Smartphone, Loader2 } from "lucide-react";
import { Link } from "react-router-dom";
import { useTranslation } from "react-i18next";
import { useQuery } from "@tanstack/react-query";
import { api } from "@/lib/api-client";
import { formatCurrency } from "@/lib/format";

export default function AppWallet() {
  const { t } = useTranslation();
  const { data, isLoading } = useQuery({
    queryKey: ["wallet"],
    queryFn: () => api.getWallet(),
  });

  return (
    <div className="space-y-5 px-5 pt-6 md:px-0 md:pt-0">
      <h1 className="font-display text-2xl font-bold">{t("wallet.buyerTitle")}</h1>

      {/* Balance card */}
      <div className="rounded-2xl border border-border bg-card p-5">
        {isLoading ? (
          <div className="flex justify-center py-4">
            <Loader2 className="h-5 w-5 animate-spin text-primary" />
          </div>
        ) : (
          <div className="space-y-3">
            <div className="flex justify-between items-center">
              <span className="text-sm text-muted-foreground">Saldo disponível</span>
              <span className="text-xl font-bold text-secondary">
                {formatCurrency(parseFloat(data?.available_balance ?? "0"))}
              </span>
            </div>
            <div className="flex justify-between items-center">
              <span className="text-sm text-muted-foreground">Em custódia</span>
              <span className="text-sm font-semibold text-primary">
                {formatCurrency(parseFloat(data?.pending_balance ?? "0"))}
              </span>
            </div>
            <p className="text-xs text-muted-foreground">Moeda: {data?.currency ?? "BRL"}</p>
          </div>
        )}
      </div>

      <div className="rounded-2xl border border-border bg-card p-5">
        <div className="flex items-start gap-3">
          <div className="flex h-11 w-11 shrink-0 items-center justify-center rounded-xl bg-primary/10">
            <Smartphone className="h-5 w-5 text-primary" />
          </div>
          <div>
            <h2 className="font-display text-lg font-bold leading-tight">{t("wallet.depositPix")}</h2>
            <p className="mt-1 text-sm leading-relaxed text-muted-foreground">{t("wallet.depositDesc")}</p>
            <Link
              to="/buyer/payment"
              className="mt-3 inline-flex items-center gap-2 rounded-xl bg-primary px-4 py-2.5 text-sm font-semibold text-primary-foreground hover:opacity-90"
            >
              {t("wallet.depositPix")} <ArrowRight className="h-4 w-4" />
            </Link>
          </div>
        </div>
      </div>

      <div className="rounded-2xl border border-border bg-card p-5 text-center">
        <Wallet className="mx-auto mb-3 h-10 w-10 text-muted-foreground" />
        <p className="text-sm text-muted-foreground">{t("buyer.protectedDesc")}</p>
        <Link
          to="/buyer/transaction-complete"
          className="mt-4 inline-flex items-center gap-2 text-sm font-semibold text-secondary"
        >
          {t("transactionComplete.viewReceipt")} <ArrowRight className="h-4 w-4" />
        </Link>
      </div>
    </div>
  );
}
