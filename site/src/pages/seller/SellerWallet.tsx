import { ArrowUpRight, ArrowDownLeft, Shield, Clock } from "lucide-react";
import { useTranslation } from "react-i18next";
import { WithdrawDialog } from "@/components/app/WithdrawDialog";
import { formatCurrency } from "@/lib/format";

export default function SellerWallet() {
  const { t } = useTranslation();

  const transactions = [
    { id: 1, type: "in" as const, desc: "ORD-002", amount: 349.9, date: "Apr 6" },
    { id: 2, type: "in" as const, desc: "ORD-006", amount: 3200, date: "Apr 3" },
    { id: 3, type: "out" as const, desc: "Withdraw", amount: 2000, date: "Apr 2" },
  ];

  return (
    <div className="space-y-5 px-5 pt-6 md:px-0 md:pt-0">
      <h1 className="font-display text-2xl font-bold">{t("seller.walletTitle")}</h1>

      <div className="rounded-2xl border border-dashed border-border bg-muted/30 p-5">
        <p className="text-sm text-muted-foreground">{t("buyer.protectedDesc")}</p>
      </div>

      <div className="vault-card space-y-3 rounded-2xl p-6">
        <p className="text-xs font-semibold uppercase tracking-[0.2em] text-white/60">{t("seller.availableBalance")}</p>
        <p className="font-display text-3xl font-bold text-white">{formatCurrency(1592.05)}</p>
        <div className="flex gap-3 pt-2">
          <WithdrawDialog>
            <button
              type="button"
              className="flex items-center gap-2 rounded-xl border border-white/20 bg-white/10 px-5 py-2.5 text-sm font-medium text-white transition hover:bg-white/20"
            >
              <ArrowUpRight className="h-4 w-4" />
              {t("buyer.withdraw")}
            </button>
          </WithdrawDialog>
        </div>
      </div>

      <div className="flex items-center gap-3 rounded-2xl border border-primary/20 bg-primary/5 p-4">
        <div className="flex h-10 w-10 shrink-0 items-center justify-center rounded-xl bg-primary/10">
          <Clock className="h-5 w-5 text-primary" />
        </div>
        <div>
          <p className="text-sm font-semibold">{formatCurrency(3889.8)} {t("seller.custodyBalance").toLowerCase()}</p>
          <p className="text-xs text-muted-foreground">{t("order.paidPix")}</p>
        </div>
      </div>

      <div>
        <h3 className="mb-3 font-display text-lg font-bold">{t("seller.transactionHistory")}</h3>
        <div className="space-y-2">
          {transactions.map((tx) => (
            <div key={tx.id} className="flex items-center gap-3 rounded-xl border border-border bg-card p-4">
              <div
                className={`flex h-10 w-10 shrink-0 items-center justify-center rounded-xl ${
                  tx.type === "in" ? "bg-secondary/10" : "bg-muted"
                }`}
              >
                {tx.type === "in" ? (
                  <ArrowDownLeft className="h-5 w-5 text-secondary" />
                ) : (
                  <ArrowUpRight className="h-5 w-5 text-muted-foreground" />
                )}
              </div>
              <div className="min-w-0 flex-1">
                <p className="truncate text-sm font-medium">{tx.desc}</p>
                <p className="text-xs text-muted-foreground">{tx.date}</p>
              </div>
              <span className={`text-sm font-semibold ${tx.type === "in" ? "text-secondary" : "text-foreground"}`}>
                {tx.type === "in" ? "+" : "-"}
                {formatCurrency(tx.amount)}
              </span>
            </div>
          ))}
        </div>
      </div>

      <div className="flex items-center justify-center gap-2 py-4 text-center text-xs text-muted-foreground">
        <Shield className="h-3.5 w-3.5 shrink-0" />
        <span>{t("common.protectedPayment")}</span>
      </div>
    </div>
  );
}
