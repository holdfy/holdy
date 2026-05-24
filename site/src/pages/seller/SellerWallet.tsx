import { ArrowUpRight, ArrowDownLeft, Shield, Clock, Loader2 } from "lucide-react";
import { useTranslation } from "react-i18next";
import { useQuery } from "@tanstack/react-query";
import { WithdrawDialog } from "@/components/app/WithdrawDialog";
import { api } from "@/lib/api-client";
import { formatCurrency } from "@/lib/format";

export default function SellerWallet() {
  const { t } = useTranslation();

  const { data: wallet, isLoading } = useQuery({
    queryKey: ["seller", "wallet"],
    queryFn: () => api.getWallet(),
  });

  const { data: ordersData } = useQuery({
    queryKey: ["orders", "seller"],
    queryFn: () => api.listOrders("seller"),
  });

  const completedOrders = ordersData?.orders.filter((o) => o.status === "completed") ?? [];
  const custodyOrders = ordersData?.orders.filter((o) => o.status === "in_custody") ?? [];

  const available = parseFloat(wallet?.available_balance ?? "0");
  const pending = parseFloat(wallet?.pending_balance ?? "0");

  return (
    <div className="space-y-5 px-5 pt-6 md:px-0 md:pt-0">
      <h1 className="font-display text-2xl font-bold">{t("seller.walletTitle")}</h1>

      <div className="vault-card space-y-3 rounded-2xl p-6">
        <p className="text-xs font-semibold uppercase tracking-[0.2em] text-white/60">{t("seller.availableBalance")}</p>
        {isLoading ? (
          <Loader2 className="h-6 w-6 animate-spin text-white/60" />
        ) : (
          <p className="font-display text-3xl font-bold text-white">{formatCurrency(available)}</p>
        )}
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

      {pending > 0 && (
        <div className="flex items-center gap-3 rounded-2xl border border-primary/20 bg-primary/5 p-4">
          <div className="flex h-10 w-10 shrink-0 items-center justify-center rounded-xl bg-primary/10">
            <Clock className="h-5 w-5 text-primary" />
          </div>
          <div>
            <p className="text-sm font-semibold">{formatCurrency(pending)} {t("seller.custodyBalance").toLowerCase()}</p>
            <p className="text-xs text-muted-foreground">{t("order.paidPix")}</p>
          </div>
        </div>
      )}

      {/* Recent completed orders as "transaction history" */}
      {(completedOrders.length > 0 || custodyOrders.length > 0) && (
        <div>
          <h3 className="mb-3 font-display text-lg font-bold">{t("seller.transactionHistory")}</h3>
          <div className="space-y-2">
            {[...custodyOrders, ...completedOrders].slice(0, 10).map((order) => (
              <div key={order.id} className="flex items-center gap-3 rounded-xl border border-border bg-card p-4">
                <div
                  className={`flex h-10 w-10 shrink-0 items-center justify-center rounded-xl ${
                    order.status === "completed" ? "bg-secondary/10" : "bg-primary/10"
                  }`}
                >
                  {order.status === "completed" ? (
                    <ArrowDownLeft className="h-5 w-5 text-secondary" />
                  ) : (
                    <Clock className="h-5 w-5 text-primary" />
                  )}
                </div>
                <div className="min-w-0 flex-1">
                  <p className="truncate text-sm font-medium">{order.description ?? `Pedido #${order.id.slice(0, 8)}`}</p>
                  <p className="text-xs text-muted-foreground">
                    {order.status === "completed" ? "Liberado" : "Em custódia"}
                  </p>
                </div>
                <span className={`text-sm font-semibold ${order.status === "completed" ? "text-secondary" : "text-primary"}`}>
                  +{formatCurrency(parseFloat(order.amount))}
                </span>
              </div>
            ))}
          </div>
        </div>
      )}

      <div className="flex items-center justify-center gap-2 py-4 text-center text-xs text-muted-foreground">
        <Shield className="h-3.5 w-3.5 shrink-0" />
        <span>{t("common.protectedPayment")}</span>
      </div>
    </div>
  );
}
