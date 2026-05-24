import { Shield, TrendingUp, FileText, ChevronRight, Plus, ArrowDownLeft, ShieldCheck } from "lucide-react";
import { Link } from "react-router-dom";
import { useTranslation } from "react-i18next";
import { NotificationsDialog } from "@/components/app/NotificationsDialog";
import { WithdrawDialog } from "@/components/app/WithdrawDialog";
import { formatCurrency } from "@/lib/format";

const orders = [
  { id: "#8821", name: "Ricardo Mendes", desc: "Payment for Services", amount: 1200, status: "IN_CUSTODY" as const, avatar: "RM" },
  { id: "#7712", name: "Juliana Silva", desc: "Raw Material Import", amount: 450, status: "IN_CUSTODY" as const, avatar: "JS" },
  { id: "#6543", name: "Marcos Oliveira", desc: "Equipment Purchase", amount: 3800, status: "IN_CUSTODY" as const, avatar: "MO" },
];

export default function AppHome() {
  const { t } = useTranslation();

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
          <span className="text-3xl font-display font-bold text-white">{formatCurrency(142850)}</span>
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
          <div className="flex items-center justify-between mb-3">
            <div className="h-10 w-10 rounded-xl bg-muted flex items-center justify-center">
              <TrendingUp className="h-5 w-5 text-muted-foreground" />
            </div>
            <span className="text-xs font-semibold text-muted-foreground bg-muted px-2.5 py-1 rounded-full">{t("buyer.last30Days")}</span>
          </div>
          <p className="text-xs text-muted-foreground font-medium">{t("buyer.pixAdded")}</p>
          <p className="text-2xl font-display font-bold">{formatCurrency(1240.12)}</p>
          <div className="flex items-end gap-1.5 mt-3 h-10">
            {[40, 50, 45, 55, 50, 60, 55, 65, 60, 70, 75, 90].map((h, i) => (
              <div
                key={i}
                className="flex-1 rounded-sm"
                style={{ height: `${h}%`, background: i === 11 ? "hsl(217 90% 15%)" : "hsl(165 55% 42% / 0.3)" }}
              />
            ))}
          </div>
        </div>

        <div className="bg-card rounded-2xl p-5 border border-border">
          <div className="h-10 w-10 rounded-xl bg-muted flex items-center justify-center mb-3">
            <FileText className="h-5 w-5 text-muted-foreground" />
          </div>
          <p className="text-xs text-muted-foreground font-medium">{t("buyer.fundsInCustody")}</p>
          <p className="text-2xl font-display font-bold">{formatCurrency(45000)}</p>
          <p className="text-xs text-muted-foreground mt-1">{t("buyer.activeEscrows", { count: 3 })}</p>
        </div>
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

      <div>
        <div className="flex items-center justify-between mb-4">
          <h3 className="font-display font-bold text-lg">{t("buyer.recentOrders")}</h3>
          <Link to="/buyer/orders" className="text-xs font-semibold text-muted-foreground flex items-center gap-1">
            {t("common.viewAll")} <ChevronRight className="h-3 w-3" />
          </Link>
        </div>
        <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-3">
          {orders.map((order) => (
            <Link
              key={order.id}
              to={`/buyer/orders/${order.id.replace("#", "")}`}
              className="flex items-center gap-3 bg-card rounded-xl p-4 border border-border hover:border-primary/20 transition"
            >
              <div className="h-11 w-11 rounded-full bg-muted flex items-center justify-center text-sm font-semibold text-muted-foreground flex-shrink-0">
                {order.avatar}
              </div>
              <div className="flex-1 min-w-0">
                <p className="font-semibold text-sm truncate">{order.name}</p>
                <div className="flex items-center gap-1.5 mt-0.5">
                  <Shield className="h-3 w-3 text-primary" />
                  <span className="text-xs text-muted-foreground">
                    {order.desc} {order.id}
                  </span>
                </div>
              </div>
              <div className="text-right flex-shrink-0">
                <span className="text-xs font-semibold text-secondary bg-secondary/10 px-2 py-0.5 rounded-full">
                  {t("status.IN_CUSTODY_BADGE")}
                </span>
              </div>
              <ChevronRight className="h-4 w-4 text-muted-foreground flex-shrink-0" />
            </Link>
          ))}
        </div>
      </div>

      <div className="h-4" />
    </div>
  );
}
