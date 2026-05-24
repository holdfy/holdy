import { useState } from "react";
import { AlertTriangle, MessageSquare, CheckCircle, Clock, Shield } from "lucide-react";
import { useTranslation } from "react-i18next";
import { useQuery } from "@tanstack/react-query";
import { api } from "@/lib/api-client";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog";
import { Button } from "@/components/ui/button";
import { Label } from "@/components/ui/label";
import { Textarea } from "@/components/ui/textarea";
import { toast } from "sonner";
import { formatCurrency } from "@/lib/format";

// A "dispute" here is an order that had the dispute endpoint called.
// We surface orders in the seller's list that have been disputed (status = "failed" indicates dispute-closed).
const DISPUTED_STATUSES = ["failed", "cancelled"];

export default function SellerDisputes() {
  const { t } = useTranslation();
  const [respondId, setRespondId] = useState<string | null>(null);
  const [message, setMessage] = useState("");

  const { data: ordersData, isLoading } = useQuery({
    queryKey: ["orders", "seller"],
    queryFn: () => api.listOrders("seller"),
  });

  const disputedOrders = (ordersData?.orders ?? []).filter((o) =>
    DISPUTED_STATUSES.includes(o.status)
  );

  const sendResponse = () => {
    if (!message.trim()) {
      toast.error(t("seller.toastRespondEmpty"));
      return;
    }
    toast.success(t("seller.toastRespondSent"));
    setRespondId(null);
    setMessage("");
  };

  return (
    <div className="px-5 pt-6 space-y-5 md:px-0 md:pt-0">
      <h1 className="font-display text-2xl font-bold">{t("seller.disputesTitle")}</h1>

      <div className="grid grid-cols-3 gap-3">
        <div className="bg-destructive/5 border border-destructive/20 rounded-xl p-3 text-center">
          <p className="text-xl font-display font-bold text-destructive">{disputedOrders.length}</p>
          <p className="text-[10px] text-muted-foreground font-medium uppercase tracking-wider">{t("status.OPEN")}</p>
        </div>
        <div className="bg-amber-50 border border-amber-200 rounded-xl p-3 text-center">
          <p className="text-xl font-display font-bold text-amber-600">0</p>
          <p className="text-[10px] text-muted-foreground font-medium uppercase tracking-wider">{t("status.IN_REVIEW")}</p>
        </div>
        <div className="bg-secondary/5 border border-secondary/20 rounded-xl p-3 text-center">
          <p className="text-xl font-display font-bold text-secondary">0</p>
          <p className="text-[10px] text-muted-foreground font-medium uppercase tracking-wider">{t("status.RESOLVED")}</p>
        </div>
      </div>

      <div className="space-y-3">
        {isLoading ? (
          <p className="text-sm text-muted-foreground text-center py-8">Carregando...</p>
        ) : disputedOrders.length === 0 ? (
          <div className="flex flex-col items-center gap-3 py-12 text-center">
            <CheckCircle className="h-10 w-10 text-secondary" />
            <p className="font-semibold">{t("seller.noDisputes")}</p>
            <p className="text-sm text-muted-foreground">{t("seller.noDisputesDesc")}</p>
          </div>
        ) : (
          disputedOrders.map((order) => (
            <div key={order.id} className="bg-card rounded-xl p-4 border border-border space-y-3">
              <div className="flex items-start justify-between gap-2">
                <div className="flex items-center gap-3 min-w-0">
                  <div className="h-10 w-10 rounded-xl bg-destructive/10 flex items-center justify-center flex-shrink-0">
                    <AlertTriangle className="h-5 w-5 text-destructive" />
                  </div>
                  <div className="min-w-0">
                    <p className="font-semibold text-sm truncate">{order.description ?? "Pedido"}</p>
                    <p className="text-xs text-muted-foreground">
                      #{order.id.slice(0, 8)}
                    </p>
                  </div>
                </div>
                <span className="text-[10px] font-semibold px-2 py-0.5 rounded-full flex-shrink-0 text-destructive bg-destructive/10">
                  {t("status.OPEN")}
                </span>
              </div>
              <div className="bg-muted/50 rounded-lg p-3">
                <p className="text-xs font-medium text-muted-foreground mb-1">Valor em disputa</p>
                <p className="text-sm font-semibold">{formatCurrency(parseFloat(order.amount))}</p>
              </div>
              <div className="flex gap-2">
                <button
                  type="button"
                  className="flex-1 flex items-center justify-center gap-2 px-4 py-2.5 rounded-xl bg-primary text-primary-foreground text-sm font-semibold hover:opacity-90 transition"
                  onClick={() => {
                    setMessage("");
                    setRespondId(order.id);
                  }}
                >
                  <MessageSquare className="h-4 w-4" />
                  {t("seller.respond")}
                </button>
              </div>
            </div>
          ))
        )}
      </div>

      <div className="flex items-center justify-center gap-2 py-4 text-xs text-muted-foreground">
        <Shield className="h-3.5 w-3.5" />
        <span className="tracking-wider uppercase font-medium">{t("common.protectedPayment")}</span>
      </div>

      <Dialog open={!!respondId} onOpenChange={(o) => !o && setRespondId(null)}>
        <DialogContent>
          <DialogHeader>
            <DialogTitle>{t("seller.respondTitle")}</DialogTitle>
            <DialogDescription>{t("seller.respondDesc")}</DialogDescription>
          </DialogHeader>
          <div className="space-y-2">
            <Label htmlFor="dispute-reply">{t("common.message")}</Label>
            <Textarea
              id="dispute-reply"
              placeholder={t("seller.respondPlaceholder")}
              value={message}
              onChange={(e) => setMessage(e.target.value)}
            />
          </div>
          <DialogFooter className="gap-2">
            <Button type="button" variant="outline" onClick={() => setRespondId(null)}>
              {t("common.cancel")}
            </Button>
            <Button type="button" onClick={sendResponse}>
              {t("seller.respond")}
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>
    </div>
  );
}
