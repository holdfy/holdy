import { ChevronRight } from "lucide-react";
import { Link } from "react-router-dom";
import { useMemo, useState } from "react";
import { useTranslation } from "react-i18next";
import { mockOrders } from "@/lib/mock-data";
import { getOrderDescription } from "@/lib/mock-i18n";
import { formatCurrency } from "@/lib/format";

const statusFilters = ["ALL", "PENDING", "IN_CUSTODY", "COMPLETED", "CANCELLED"] as const;

export default function SellerOrders() {
  const { t } = useTranslation();
  const [filter, setFilter] = useState<string>("ALL");
  const filtered = filter === "ALL" ? mockOrders : mockOrders.filter((o) => o.status === filter);

  const filterLabels = useMemo(
    () => ({
      ALL: t("seller.filterAll"),
      PENDING: t("seller.filterPending"),
      IN_CUSTODY: t("seller.filterInCustody"),
      COMPLETED: t("seller.filterCompleted"),
      CANCELLED: t("seller.filterCancelled"),
    }),
    [t],
  );

  return (
    <div className="px-5 pt-6 space-y-5 md:px-0 md:pt-0">
      <div className="flex items-center justify-between">
        <h1 className="font-display text-2xl font-bold">{t("seller.ordersTitle")}</h1>
        <span className="text-xs text-muted-foreground font-medium">{mockOrders.length}</span>
      </div>

      <div className="flex gap-2 overflow-x-auto pb-1 -mx-1 px-1">
        {statusFilters.map((s) => (
          <button
            key={s}
            onClick={() => setFilter(s)}
            className={`px-3 py-1.5 rounded-full text-xs font-semibold whitespace-nowrap transition ${
              filter === s ? "bg-primary text-primary-foreground" : "bg-muted text-muted-foreground hover:bg-muted/80"
            }`}
          >
            {filterLabels[s]}
          </button>
        ))}
      </div>

      <div className="grid grid-cols-1 md:grid-cols-2 gap-3">
        {filtered.map((order) => (
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
              <p className="text-xs text-muted-foreground mt-0.5">
                {getOrderDescription(order.id, t)} · {order.id}
              </p>
            </div>
            <div className="text-right flex-shrink-0">
              <p className="text-sm font-semibold">{formatCurrency(order.amount)}</p>
              <span
                className={`text-[10px] font-semibold px-2 py-0.5 rounded-full ${
                  order.status === "COMPLETED"
                    ? "text-secondary bg-secondary/10"
                    : order.status === "CANCELLED"
                      ? "text-destructive bg-destructive/10"
                      : order.status === "PENDING"
                        ? "text-amber-600 bg-amber-100"
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

      {filtered.length === 0 && (
        <div className="text-center py-12 text-muted-foreground">
          <p className="text-sm">{t("seller.noOrders")}</p>
        </div>
      )}
    </div>
  );
}
