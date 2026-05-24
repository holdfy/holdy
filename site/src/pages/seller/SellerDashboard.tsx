import { Shield, TrendingUp, FileText, AlertTriangle, ChevronRight, DollarSign, ShieldCheck } from "lucide-react";
import { Link } from "react-router-dom";
import { useTranslation } from "react-i18next";
import { NotificationsDialog } from "@/components/app/NotificationsDialog";
import { mockOrders, mockDisputes, mockMetrics } from "@/lib/mock-data";
import { getOrderDescription } from "@/lib/mock-i18n";
import { formatCurrency } from "@/lib/format";

export default function SellerDashboard() {
  const { t } = useTranslation();
  const totalSales = mockOrders.reduce((sum, o) => sum + o.amount, 0);
  const inCustody = mockOrders.filter((o) => o.status === "IN_CUSTODY").reduce((sum, o) => sum + o.amount, 0);
  const released = mockOrders.filter((o) => o.status === "COMPLETED").reduce((sum, o) => sum + o.amount, 0);
  const openDisputes = mockDisputes.filter((d) => d.status === "OPEN" || d.status === "IN_REVIEW").length;

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

      <div className="vault-card rounded-2xl p-6 space-y-4">
        <p className="text-xs font-semibold tracking-[0.2em] text-white/60 uppercase">{t("seller.totalRevenue")}</p>
        <div>
          <span className="text-3xl font-display font-bold text-white">{formatCurrency(totalSales)}</span>
        </div>
        <div className="flex items-center gap-2 text-xs text-secondary">
          <ShieldCheck className="h-4 w-4" />
          <span>{t("common.protectedPayment")}</span>
        </div>
      </div>

      <div className="grid grid-cols-2 md:grid-cols-4 gap-3">
        <div className="bg-card rounded-2xl p-4 border border-border">
          <div className="h-9 w-9 rounded-xl bg-primary/10 flex items-center justify-center mb-2">
            <DollarSign className="h-4 w-4 text-primary" />
          </div>
          <p className="text-[10px] text-muted-foreground font-medium uppercase tracking-wider">{t("seller.inCustody")}</p>
          <p className="text-lg font-display font-bold">{formatCurrency(inCustody)}</p>
        </div>
        <div className="bg-card rounded-2xl p-4 border border-border">
          <div className="h-9 w-9 rounded-xl bg-secondary/10 flex items-center justify-center mb-2">
            <TrendingUp className="h-4 w-4 text-secondary" />
          </div>
          <p className="text-[10px] text-muted-foreground font-medium uppercase tracking-wider">{t("seller.released")}</p>
          <p className="text-lg font-display font-bold">{formatCurrency(released)}</p>
        </div>
        <div className="bg-card rounded-2xl p-4 border border-border">
          <div className="h-9 w-9 rounded-xl bg-muted flex items-center justify-center mb-2">
            <FileText className="h-4 w-4 text-muted-foreground" />
          </div>
          <p className="text-[10px] text-muted-foreground font-medium uppercase tracking-wider">{t("seller.activeOrders")}</p>
          <p className="text-lg font-display font-bold">{mockOrders.filter((o) => o.status !== "CANCELLED").length}</p>
        </div>
        <div className="bg-card rounded-2xl p-4 border border-border">
          <div className="h-9 w-9 rounded-xl bg-destructive/10 flex items-center justify-center mb-2">
            <AlertTriangle className="h-4 w-4 text-destructive" />
          </div>
          <p className="text-[10px] text-muted-foreground font-medium uppercase tracking-wider">{t("seller.disputes")}</p>
          <p className="text-lg font-display font-bold">{openDisputes}</p>
        </div>
      </div>

      <div className="bg-card rounded-2xl p-5 border border-border">
        <h3 className="font-display font-bold mb-4">{t("seller.salesVolume")}</h3>
        <div className="flex items-end gap-1.5 h-32">
          {mockMetrics.transactionsChart.map((item, i) => {
            const maxVal = Math.max(...mockMetrics.transactionsChart.map((chartItem) => chartItem.value));
            const height = (item.value / maxVal) * 100;
            return (
              <div key={i} className="flex-1 flex flex-col items-center gap-1">
                <div
                  className="w-full rounded-t-md bg-primary/20 hover:bg-primary/40 transition-colors"
                  style={{ height: `${height}%` }}
                />
                <span className="text-[9px] text-muted-foreground">{item.date}</span>
              </div>
            );
          })}
        </div>
      </div>

      <div>
        <div className="flex items-center justify-between mb-4">
          <h3 className="font-display font-bold text-lg">{t("seller.recentOrders")}</h3>
          <Link to="/seller/orders" className="text-xs font-semibold text-muted-foreground flex items-center gap-1">
            {t("common.viewAll")} <ChevronRight className="h-3 w-3" />
          </Link>
        </div>
        <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-3">
          {mockOrders.slice(0, 3).map((order) => (
            <Link
              key={order.id}
              to={`/seller/orders/${order.id}`}
              className="flex items-center gap-3 bg-card rounded-xl p-4 border border-border hover:border-primary/20 transition"
            >
              <div className="h-11 w-11 rounded-full bg-muted flex items-center justify-center text-sm font-semibold text-muted-foreground flex-shrink-0">
                {order.buyer.split(" ").map((n) => n[0]).join("")}
              </div>
              <div className="flex-1 min-w-0">
                <p className="font-semibold text-sm truncate">{order.buyer}</p>
                <p className="text-xs text-muted-foreground mt-0.5">{getOrderDescription(order.id, t)}</p>
              </div>
              <div className="text-right flex-shrink-0">
                <p className="text-sm font-semibold">{formatCurrency(order.amount)}</p>
                <span
                  className={`text-[10px] font-semibold px-2 py-0.5 rounded-full ${
                    order.status === "COMPLETED"
                      ? "text-secondary bg-secondary/10"
                      : order.status === "CANCELLED"
                        ? "text-destructive bg-destructive/10"
                        : "text-primary bg-primary/10"
                  }`}
                >
                  {order.status === "IN_CUSTODY" ? t("status.IN_CUSTODY_BADGE") : t(`status.${order.status}`)}
                </span>
              </div>
            </Link>
          ))}
        </div>
      </div>

      {openDisputes > 0 && (
        <Link
          to="/seller/disputes"
          className="flex items-center gap-3 bg-destructive/5 border border-destructive/20 rounded-2xl p-4 hover:bg-destructive/10 transition"
        >
          <div className="h-10 w-10 rounded-xl bg-destructive/10 flex items-center justify-center flex-shrink-0">
            <AlertTriangle className="h-5 w-5 text-destructive" />
          </div>
          <div className="flex-1">
            <p className="font-semibold text-sm">{t("seller.openDisputes", { count: openDisputes })}</p>
            <p className="text-xs text-muted-foreground">{t("seller.requiresAttention")}</p>
          </div>
          <ChevronRight className="h-4 w-4 text-muted-foreground" />
        </Link>
      )}

      <div className="h-4" />
    </div>
  );
}
