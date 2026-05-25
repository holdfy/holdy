import { Shield, FileText, ChevronRight, Plus, ArrowDownLeft, ShieldCheck, Loader2 } from "lucide-react";
import { Link } from "react-router-dom";
import { useTranslation } from "react-i18next";
import { useQuery } from "@tanstack/react-query";
import { NotificationsDialog } from "@/components/app/NotificationsDialog";
import { WithdrawDialog } from "@/components/app/WithdrawDialog";
import { formatCurrency } from "@/lib/format";
import { api } from "@/lib/api-client";

const STATUS_BADGE: Record<string, string> = {
  in_custody: "IN_CUSTODY_BADGE",
  pending_funding: "PENDING_BADGE",
  completed: "COMPLETED_BADGE",
  cancelled: "CANCELLED_BADGE",
  failed: "FAILED_BADGE",
};

export default function AppHome() {
  const { t } = useTranslation();

  const { data: wallet, isLoading: walletLoading } = useQuery({
    queryKey: ["wallet"],
    queryFn: () => api.getWallet(),
    staleTime: 30_000,
  });

  const { data: ordersData } = useQuery({
    queryKey: ["orders", "buyer"],
    queryFn: () => api.listOrders("buyer"),
    staleTime: 30_000,
  });

  const totalBalance = wallet ? parseFloat(wallet.available_balance) + parseFloat(wallet.pending_balance) : null;
  const pending = wallet ? parseFloat(wallet.pending_balance) : null;
  const recentOrders = ordersData?.orders.slice(0, 3) ?? [];

  return (
    <div className="px-5 pt-6 space-y-5 md:px-0 md:pt-0">
      <div className="flex items-center justify-between md:hidden">
        <div className="flex items-center gap-3">
          <div className="h-10 w-10 rounded-full vault-card flex items-center justify-center">
            <Shield className="h-5 w-5 text-white" />
          </div>
          <span className="font-display font-bold text-lg">{t("common.holdfy")}</span>
        </div>
        <NotificationsDialog>
          <button
            type="button"
            className="h-10 w-10 rounded-full bg-muted flex items-center justify-center"
            aria-label={t("common.notifications")}
          >
            <span className="text-sm">🔔</span>
          </button>
        </NotificationsDialog>
      </div>

      <div className="vault-card rounded-2xl p-6 space-y-4">
        <p className="text-xs font-semibold tracking-[0.2em] text-white/60 uppercase">{t("buyer.totalBalance")}</p>
        <div>
          {walletLoading ? (
            <Loader2 className="h-8 w-8 animate-spin text-white/50" />
          ) : (
            <span className="text-3xl font-display font-bold text-white">
              {totalBalance != null ? formatCurrency(totalBalance) : "—"}
            </span>
          )}
          <span className="ml-2 text-secondary text-sm font-semibold">BRL</span>
        </div>
        <div className="flex items-center gap-2 text-xs text-secondary">
          <ShieldCheck className="h-4 w-4" />
          <span>{t("common.protectedPayment")}</span>
        </div>
        <div className="flex gap-3 pt-2">
          <Link
            to="/buyer/payment"
            className="flex items-center gap-2 px-5 py-2.5 rounded-xl bg-white/10 border border-white/20 text-white text-sm font-medium hover:bg-white/20 transition"
          >
            <Plus className="h-4 w-4" />
            {t("buyer.addFunds")}
          </Link>
          <WithdrawDialog>
            <button
              type="button"
              className="flex items-center gap-2 px-5 py-2.5 rounded-xl bg-white/20 text-white text-sm font-medium hover:bg-white/30 transition"
            >
              <ArrowDownLeft className="h-4 w-4" />
              {t("buyer.withdraw")}
            </button>
          </WithdrawDialog>
        </div>
      </div>

      <div className="grid grid-cols-1 md:grid-cols-2 gap-5">
        <div className="bg-card rounded-2xl p-5 border border-border">
          <div className="h-10 w-10 rounded-xl bg-muted flex items-center justify-center mb-3">
            <FileText className="h-5 w-5 text-muted-foreground" />
          </div>
          <p className="text-xs text-muted-foreground font-medium">{t("buyer.fundsInCustody")}</p>
          <p className="text-2xl font-display font-bold">
            {pending != null ? formatCurrency(pending) : "—"}
          </p>
          <p className="text-xs text-muted-foreground mt-1">
            {t("buyer.activeEscrows", { count: recentOrders.filter(o => o.status === "in_custody").length })}
          </p>
        </div>

        <div className="bg-secondary/10 rounded-2xl p-5 relative overflow-hidden">
          <div className="relative z-10">
            <div className="flex items-center gap-2 mb-2">
              <Shield className="h-5 w-5 text-secondary" />
              <span className="font-display font-bold">{t("common.protectedPayment")}</span>
            </div>
            <p className="text-sm text-muted-foreground leading-relaxed">{t("buyer.protectedDesc")}</p>
          </div>
          <div className="absolute bottom-2 right-2 opacity-10">
            <Shield className="h-20 w-20 text-secondary" />
          </div>
        </div>
      </div>

      <div>
        <div className="flex items-center justify-between mb-4">
          <h3 className="font-display font-bold text-lg">{t("buyer.recentOrders")}</h3>
          <Link to="/buyer/orders" className="text-xs font-semibold text-muted-foreground flex items-center gap-1">
            {t("common.viewAll")} <ChevronRight className="h-3 w-3" />
          </Link>
        </div>

        {recentOrders.length === 0 ? (
          <div className="text-center py-10 text-muted-foreground text-sm">
            {t("buyer.noOrders", "Nenhum pedido ainda.")}
          </div>
        ) : (
          <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-3">
            {recentOrders.map((order) => {
              const badgeKey = STATUS_BADGE[order.status] ?? "PENDING_BADGE";
              const initials = order.seller_id.slice(0, 2).toUpperCase();
              return (
                <Link
                  key={order.id}
                  to={`/buyer/orders/${order.id}`}
                  className="flex items-center gap-3 bg-card rounded-xl p-4 border border-border hover:border-primary/20 transition"
                >
                  <div className="h-11 w-11 rounded-full bg-muted flex items-center justify-center text-sm font-semibold text-muted-foreground flex-shrink-0">
                    {initials}
                  </div>
                  <div className="flex-1 min-w-0">
                    <p className="font-semibold text-sm truncate">{order.description ?? t("common.order")}</p>
                    <p className="text-xs text-muted-foreground truncate mt-0.5">
                      {formatCurrency(parseFloat(order.amount))}
                    </p>
                  </div>
                  <div className="text-right flex-shrink-0">
                    <span className="text-xs font-semibold text-secondary bg-secondary/10 px-2 py-0.5 rounded-full">
                      {t(`status.${badgeKey}`, order.status)}
                    </span>
                  </div>
                  <ChevronRight className="h-4 w-4 text-muted-foreground flex-shrink-0" />
                </Link>
              );
            })}
          </div>
        )}
      </div>

      <div className="h-4" />
    </div>
  );
}
