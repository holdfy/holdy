import { Shield, ChevronRight } from "lucide-react";
import { Link } from "react-router-dom";
import { useTranslation } from "react-i18next";
import { formatCurrency } from "@/lib/format";

const orders = [
  { id: "8921", product: "MacBook Pro M2", amount: 8500, status: "IN_CUSTODY" as const, date: "Apr 8" },
  { id: "8820", product: "iPhone 15 Pro", amount: 6999, status: "IN_CUSTODY" as const, date: "Apr 7" },
  { id: "8715", product: "Monitor 4K", amount: 2499, status: "COMPLETED" as const, date: "Apr 5" },
  { id: "8610", product: "Mechanical keyboard", amount: 890, status: "RELEASED" as const, date: "Apr 3" },
];

export default function AppOrders() {
  const { t } = useTranslation();

  return (
    <div className="px-5 pt-6 space-y-5 md:px-0 md:pt-0">
      <h1 className="font-display text-2xl font-bold">{t("buyer.ordersTitle")}</h1>
      <div className="grid grid-cols-1 md:grid-cols-2 gap-3">
        {orders.map((order) => (
          <Link
            key={order.id}
            to={`/buyer/orders/${order.id}`}
            className="flex items-center gap-3 bg-card rounded-xl p-4 border border-border hover:border-primary/20 transition"
          >
            <div className="h-11 w-11 rounded-xl bg-muted flex items-center justify-center flex-shrink-0">
              <Shield className="h-5 w-5 text-primary" />
            </div>
            <div className="flex-1 min-w-0">
              <p className="font-semibold text-sm">{order.product}</p>
              <p className="text-xs text-muted-foreground mt-0.5">{t("buyer.orderLabel", { id: order.id, date: order.date })}</p>
            </div>
            <div className="text-right flex-shrink-0">
              <p className="text-sm font-semibold">{formatCurrency(order.amount)}</p>
              <span
                className={`text-[10px] font-semibold px-2 py-0.5 rounded-full ${
                  order.status === "COMPLETED" || order.status === "RELEASED"
                    ? "text-secondary bg-secondary/10"
                    : "text-primary bg-primary/10"
                }`}
              >
                {order.status === "IN_CUSTODY" ? t("status.IN_CUSTODY_BADGE") : t(`status.${order.status}`)}
              </span>
            </div>
            <ChevronRight className="h-4 w-4 text-muted-foreground flex-shrink-0" />
          </Link>
        ))}
      </div>
    </div>
  );
}
