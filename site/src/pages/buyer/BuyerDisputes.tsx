import { useState } from "react";
import { AlertTriangle, CheckCircle, Shield, Loader2, Bot, ChevronRight } from "lucide-react";
import { useTranslation } from "react-i18next";
import { useQuery, useQueries } from "@tanstack/react-query";
import { Link } from "react-router-dom";
import { api } from "@/lib/api-client";
import type { DisputeResponse } from "@/lib/api-client";
import { formatCurrency } from "@/lib/format";
import { DisputeEvidenceTimeline } from "@/components/dispute/DisputeEvidenceTimeline";

export default function BuyerDisputes() {
  const { t } = useTranslation();
  const [tab, setTab] = useState<"active" | "resolved">("active");

  const { data: ordersData, isLoading: loadingOrders } = useQuery({
    queryKey: ["orders", "buyer"],
    queryFn: () => api.listOrders("buyer"),
  });

  // Disputas continuam relevantes mesmo depois que o pedido sai de in_custody —
  // a resolução via IA/admin (refund/release) marca o pedido como completed.
  const candidateOrders = (ordersData?.orders ?? []).filter(
    (o) => o.status === "in_custody" || o.status === "completed",
  );

  // Fetch dispute for each candidate order — shares cache com AppOrderDetail
  const disputeQueries = useQueries({
    queries: candidateOrders.map((order) => ({
      queryKey: ["dispute", order.id],
      queryFn: async (): Promise<DisputeResponse | null> => {
        try {
          return await api.getDispute(order.id);
        } catch {
          return null;
        }
      },
      retry: false,
      staleTime: 30_000,
      refetchInterval: 15_000,
    })),
  });

  const isLoading =
    loadingOrders ||
    (candidateOrders.length > 0 && disputeQueries.some((q) => q.isLoading));

  type OrderWithDispute = {
    order: (typeof candidateOrders)[0];
    dispute: DisputeResponse;
  };
  const disputedPairs: OrderWithDispute[] = candidateOrders
    .map((order, i) => ({ order, dispute: disputeQueries[i]?.data ?? null }))
    .filter((p): p is OrderWithDispute => p.dispute != null);

  const activePairs = disputedPairs.filter(
    (p) => p.dispute.status === "open" || p.dispute.status === "under_review",
  );
  const resolvedPairs = disputedPairs.filter(
    (p) => p.dispute.status === "resolved" || p.dispute.status === "closed",
  );
  const openCount = disputedPairs.filter((p) => p.dispute.status === "open").length;
  const reviewCount = disputedPairs.filter((p) => p.dispute.status === "under_review").length;
  const visiblePairs = tab === "active" ? activePairs : resolvedPairs;

  const statusLabel: Record<string, string> = {
    open: t("order.disputeStatusOpen", "Aberta"),
    under_review: t("order.disputeStatusReview", "Em análise pela IA"),
    resolved: t("order.disputeStatusResolved", "Resolvida"),
    closed: t("order.disputeStatusClosed", "Encerrada"),
  };

  const statusColor: Record<string, string> = {
    open: "text-destructive bg-destructive/10",
    under_review: "text-amber-600 bg-amber-50",
    resolved: "text-secondary bg-secondary/10",
    closed: "text-muted-foreground bg-muted",
  };

  return (
    <div className="px-5 pt-6 space-y-5 md:px-0 md:pt-0">
      <h1 className="font-display text-2xl font-bold">{t("order.disputesTitle", "Minhas disputas")}</h1>

      <div className="grid grid-cols-3 gap-3">
        <div className="bg-destructive/5 border border-destructive/20 rounded-xl p-3 text-center">
          <p className="text-xl font-display font-bold text-destructive">{openCount}</p>
          <p className="text-[10px] text-muted-foreground font-medium uppercase tracking-wider">{t("status.OPEN")}</p>
        </div>
        <div className="bg-amber-50 border border-amber-200 rounded-xl p-3 text-center">
          <p className="text-xl font-display font-bold text-amber-600">{reviewCount}</p>
          <p className="text-[10px] text-muted-foreground font-medium uppercase tracking-wider">{t("status.IN_REVIEW")}</p>
        </div>
        <div className="bg-secondary/5 border border-secondary/20 rounded-xl p-3 text-center">
          <p className="text-xl font-display font-bold text-secondary">{resolvedPairs.length}</p>
          <p className="text-[10px] text-muted-foreground font-medium uppercase tracking-wider">{t("status.RESOLVED")}</p>
        </div>
      </div>

      <div className="flex gap-2">
        <button
          type="button"
          onClick={() => setTab("active")}
          className={`flex-1 text-xs font-semibold py-2 rounded-lg transition ${
            tab === "active" ? "bg-primary text-primary-foreground" : "bg-muted text-muted-foreground"
          }`}
        >
          {t("order.disputesActiveTab", "Ativas")} ({activePairs.length})
        </button>
        <button
          type="button"
          onClick={() => setTab("resolved")}
          className={`flex-1 text-xs font-semibold py-2 rounded-lg transition ${
            tab === "resolved" ? "bg-primary text-primary-foreground" : "bg-muted text-muted-foreground"
          }`}
        >
          {t("order.disputesResolvedTab", "Resolvidas")} ({resolvedPairs.length})
        </button>
      </div>

      <div className="space-y-3">
        {isLoading ? (
          <div className="flex justify-center py-8">
            <Loader2 className="h-5 w-5 animate-spin text-primary" />
          </div>
        ) : visiblePairs.length === 0 ? (
          <div className="flex flex-col items-center gap-3 py-12 text-center">
            <CheckCircle className="h-10 w-10 text-secondary" />
            <p className="font-semibold">{t("order.noDisputes", "Sem disputas")}</p>
            <p className="text-sm text-muted-foreground">
              {t("order.noDisputesDesc", "Nenhuma disputa em aberto ou resolvida.")}
            </p>
          </div>
        ) : (
          visiblePairs.map(({ order, dispute }) => {
            const verdictText =
              dispute.ai_verdict === "favor_buyer"
                ? t("order.aiVerdictBuyer", "IA decidiu a favor do comprador")
                : dispute.ai_verdict === "favor_seller"
                  ? t("order.aiVerdictSeller", "IA decidiu a favor do vendedor")
                  : dispute.ai_verdict === "inconclusive"
                    ? t("order.aiVerdictInconclusive", "IA inconclusiva — revisão manual")
                    : null;
            const verdictColor =
              dispute.ai_verdict === "favor_buyer"
                ? "text-secondary"
                : dispute.ai_verdict === "favor_seller"
                  ? "text-destructive"
                  : "text-amber-600";

            return (
              <div
                key={order.id}
                className="bg-card rounded-xl p-4 border border-border space-y-3"
              >
                <Link
                  to={`/buyer/orders/${order.id}`}
                  className="flex items-start justify-between gap-2 hover:opacity-80 transition"
                >
                  <div className="flex items-center gap-3 min-w-0">
                    <div className="h-10 w-10 rounded-xl bg-destructive/10 flex items-center justify-center flex-shrink-0">
                      <AlertTriangle className="h-5 w-5 text-destructive" />
                    </div>
                    <div className="min-w-0">
                      <p className="font-semibold text-sm truncate">{order.description ?? "Pedido"}</p>
                      <p className="text-xs text-muted-foreground">#{order.id.slice(0, 8)}</p>
                    </div>
                  </div>
                  <div className="flex items-center gap-1.5 flex-shrink-0">
                    <span className={`text-[10px] font-semibold px-2 py-0.5 rounded-full ${statusColor[dispute.status] ?? statusColor.closed}`}>
                      {statusLabel[dispute.status] ?? dispute.status}
                    </span>
                    <ChevronRight className="h-4 w-4 text-muted-foreground" />
                  </div>
                </Link>

                <div className="bg-muted/50 rounded-lg p-3">
                  <p className="text-xs font-medium text-muted-foreground mb-1">Valor em disputa</p>
                  <p className="text-sm font-semibold">{formatCurrency(parseFloat(order.amount))}</p>
                </div>

                {verdictText && (
                  <div className="flex items-start gap-2 bg-muted/40 rounded-lg px-3 py-2">
                    <Bot className={`h-4 w-4 shrink-0 mt-0.5 ${verdictColor}`} />
                    <div className="space-y-0.5">
                      <p className={`text-xs font-semibold ${verdictColor}`}>{verdictText}</p>
                      {dispute.ai_confidence != null && (
                        <p className="text-[10px] text-muted-foreground">
                          {t("order.aiConfidence", "Confiança")}: {Math.round(dispute.ai_confidence * 100)}%
                        </p>
                      )}
                    </div>
                  </div>
                )}

                {dispute.status === "under_review" && !dispute.ai_verdict && (
                  <div className="flex items-center gap-2 text-xs text-muted-foreground">
                    <Loader2 className="h-3.5 w-3.5 animate-spin" />
                    <span>{t("order.disputeStatusReview", "Em análise pela IA")}</span>
                  </div>
                )}

                {dispute.resolution_notes && (
                  <div className="text-xs text-muted-foreground border-t border-border pt-3">
                    <p className="font-semibold text-foreground mb-0.5">{t("order.resolutionNotes", "Notas da resolução")}</p>
                    <p>{dispute.resolution_notes}</p>
                  </div>
                )}

                <div className="border-t border-border pt-3">
                  <DisputeEvidenceTimeline evidence={dispute.evidence} currentParty="buyer" />
                </div>
              </div>
            );
          })
        )}
      </div>

      <div className="flex items-center justify-center gap-2 py-4 text-xs text-muted-foreground">
        <Shield className="h-3.5 w-3.5" />
        <span className="tracking-wider uppercase font-medium">{t("common.protectedPayment")}</span>
      </div>
    </div>
  );
}
