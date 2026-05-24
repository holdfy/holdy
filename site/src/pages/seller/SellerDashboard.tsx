import { Shield, TrendingUp, FileText, AlertTriangle, ChevronRight, DollarSign, ShieldCheck, Loader2 } from "lucide-react";
import { Link } from "react-router-dom";
import { useTranslation } from "react-i18next";
import { useQuery } from "@tanstack/react-query";
import { NotificationsDialog } from "@/components/app/NotificationsDialog";
import { api } from "@/lib/api-client";
import { formatCurrency } from "@/lib/format";

export default function SellerDashboard() {
  const { t } = useTranslation();

  const { data: dash, isLoading: dashLoading } = useQuery({
    queryKey: ["seller", "dashboard"],
    queryFn: () => api.getSellerDashboard(),
  });

  const { data: ordersData, isLoading: ordersLoading } = useQuery({
    queryKey: ["orders", "seller"],
    queryFn: () => api.listOrders("seller"),
  });

  const isLoading = dashLoading || ordersLoading;
  const recentOrders = ordersData?.orders.slice(0, 3) ?? [];

  const totalVolume = parseFloat(dash?.total_volume_brl ?? "0");
  const completedVolume = parseFloat(dash?.completed_volume_brl ?? "0");
  const inCustodyOrders = dash?.in_custody_orders ?? 0;
  const totalOrders = dash?.total_orders ?? 0;

  return (
    <div className="px-5 pt-6 space-y-5 md:px-0 md:pt-0">
      <div className="flex items-center justify-between md:hidden">
        <div className="flex items-center gap-3">
          <div className="h-10 w-10 rounded-full vault-card flex items-center justify-center">
            <Shield className="h-5 w-5 text-white" />
          </div>
          <div>
            <span className="font-display font-bold text-lg">{t("common.holdfy")}</span>
            <span className="ml-2 text-xs bg-primary/10 text-primary px-2 py-0.5 rounded-full font-semibold">{t("common.seller")}</span>
          </div>
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

      {/* Revenue card */}
      <div className="vault-card rounded-2xl p-6 space-y-4">
        <p className="text-xs font-semibold tracking-[0.2em] text-white/60 uppercase">{t("seller.totalRevenue")}</p>
        <div>
          {isLoading ? (
            <Loader2 className="h-6 w-6 animate-spin text-white/60" />
          ) : (
            <span className="text-3xl font-display font-bold text-white">{formatCurrency(totalVolume)}</span>
          )}
        </div>
        <div className="flex items-center gap-2 text-xs text-secondary">
          <ShieldCheck className="h-4 w-4" />
          <span>{t("common.protectedPayment")}</span>
        </div>
      </div>

      {/* KPI grid */}
      <div className="grid grid-cols-2 md:grid-cols-4 gap-3">
        <div className="bg-card rounded-2xl p-4 border border-border">
          <div className="h-9 w-9 rounded-xl bg-primary/10 flex items-center justify-center mb-2">
            <DollarSign className="h-4 w-4 text-primary" />
          </div>
          <p className="text-[10px] text-muted-foreground font-medium uppercase tracking-wider">{t("seller.inCustody")}</p>
          <p className="text-lg font-display font-bold">{inCustodyOrders} pedidos</p>
        </div>
        <div className="bg-card rounded-2xl p-4 border border-border">
          <div className="h-9 w-9 rounded-xl bg-secondary/10 flex items-center justify-center mb-2">
            <TrendingUp className="h-4 w-4 text-secondary" />
          </div>
          <p className="text-[10px] text-muted-foreground font-medium uppercase tracking-wider">{t("seller.released")}</p>
          <p className="text-lg font-display font-bold">{formatCurrency(completedVolume)}</p>
        </div>
        <div className="bg-card rounded-2xl p-4 border border-border">
          <div className="h-9 w-9 rounded-xl bg-muted flex items-center justify-center mb-2">
            <FileText className="h-4 w-4 text-muted-foreground" />
          </div>
          <p className="text-[10px] text-muted-foreground font-medium uppercase tracking-wider">{t("seller.activeOrders")}</p>
          <p className="text-lg font-display font-bold">{totalOrders}</p>
        </div>
        <div className="bg-card rounded-2xl p-4 border border-border">
          <div className="h-9 w-9 rounded-xl bg-destructive/10 flex items-center justify-center mb-2">
            <AlertTriangle className="h-4 w-4 text-destructive" />
          </div>
          <p className="text-[10px] text-muted-foreground font-medium uppercase tracking-wider">{t("seller.disputes")}</p>
          <p className="text-lg font-display font-bold">0</p>
        </div>
      </div>

      {/* Recent orders */}
      <div>
        <div className="flex items-center justify-between mb-4">
          <h3 className="font-display font-bold text-lg">{t("seller.recentOrders")}</h3>
          <Link to="/seller/orders" className="text-xs font-semibold text-muted-foreground flex items-center gap-1">
            {t("common.viewAll")} <ChevronRight className="h-3 w-3" />
          </Link>
        </div>
        {ordersLoading ? (
          <div className="flex justify-center py-8">
            <Loader2 className="h-5 w-5 animate-spin text-primary" />
          </div>
        ) : recentOrders.length === 0 ? (
          <p className="text-sm text-muted-foreground text-center py-6">Nenhum pedido ainda.</p>
        ) : (
          <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-3">
            {recentOrders.map((order) => (
              <Link
                key={order.id}
                to={`/seller/orders/${order.id}`}
                className="flex items-center gap-3 bg-card rounded-xl p-4 border border-border hover:border-primary/20 transition"
              >
                <div className="h-11 w-11 rounded-full bg-muted flex items-center justify-center text-sm font-semibold text-muted-foreground flex-shrink-0">
                  {order.id.slice(0, 2).toUpperCase()}
                </div>
                <div className="flex-1 min-w-0">
                  <p className="font-semibold text-sm truncate">{order.description ?? "Pedido"}</p>
                  <p className="text-xs text-muted-foreground mt-0.5">#{order.id.slice(0, 8)}</p>
                </div>
                <div className="text-right flex-shrink-0">
                  <p className="text-sm font-semibold">{formatCurrency(parseFloat(order.amount))}</p>
                  <span
                    className={`text-[10px] font-semibold px-2 py-0.5 rounded-full ${
                      order.status === "completed"
                        ? "text-secondary bg-secondary/10"
                        : order.status === "cancelled"
                          ? "text-destructive bg-destructive/10"
                          : "text-primary bg-primary/10"
                    }`}
                  >
                    {order.status === "in_custody" ? t("status.IN_CUSTODY_BADGE") : order.status.toUpperCase()}
                  </span>
                </div>
              </Link>
            ))}
          </div>
        )}
      </div>

      <div className="h-4" />
    </div>
  );
}
