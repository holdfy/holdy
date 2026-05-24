import { Shield, ChevronRight, Loader2 } from "lucide-react";
import { Link } from "react-router-dom";
import { useTranslation } from "react-i18next";
import { useQuery } from "@tanstack/react-query";
import { api } from "@/lib/api-client";
import { formatCurrency } from "@/lib/format";

function statusVariant(status: string) {
  return status === "completed" || status === "released"
    ? "text-secondary bg-secondary/10"
    : "text-primary bg-primary/10";
}

export default function AppOrders() {
  const { t } = useTranslation();
  const { data, isLoading, error } = useQuery({
    queryKey: ["orders", "buyer"],
    queryFn: () => api.listOrders("buyer"),
  });

  if (isLoading) {
    return (
      <div className="flex justify-center py-16">
        <Loader2 className="h-6 w-6 animate-spin text-primary" />
      </div>
    );
  }

  if (error) {
    return (
      <div className="px-5 pt-6">
        <p className="text-sm text-destructive">Erro ao carregar pedidos. Tente novamente.</p>
      </div>
    );
  }

  const orders = data?.orders ?? [];

  return (
    <div className="px-5 pt-6 space-y-5 md:px-0 md:pt-0">
      <h1 className="font-display text-2xl font-bold">{t("buyer.ordersTitle")}</h1>
      {orders.length === 0 ? (
        <p className="text-sm text-muted-foreground py-8 text-center">Nenhum pedido encontrado.</p>
      ) : (
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
                <p className="font-semibold text-sm">{order.description ?? "Pedido"}</p>
                <p className="text-xs text-muted-foreground mt-0.5">
                  #{order.id.slice(0, 8)}
                </p>
              </div>
              <div className="text-right flex-shrink-0">
                <p className="text-sm font-semibold">{formatCurrency(parseFloat(order.amount))}</p>
                <span className={`text-[10px] font-semibold px-2 py-0.5 rounded-full ${statusVariant(order.status)}`}>
                  {order.status === "in_custody" ? t("status.IN_CUSTODY_BADGE") : order.status.toUpperCase()}
                </span>
              </div>
              <ChevronRight className="h-4 w-4 text-muted-foreground flex-shrink-0" />
            </Link>
          ))}
        </div>
      )}
    </div>
  );
}
