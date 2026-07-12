import { useState } from "react";
import { AlertTriangle, CheckCircle, Shield, Loader2, Paperclip, Bot } from "lucide-react";
import { useTranslation } from "react-i18next";
import { useQuery, useQueries, useQueryClient } from "@tanstack/react-query";
import { api } from "@/lib/api-client";
import type { ApiError, DisputeResponse } from "@/lib/api-client";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { toast } from "sonner";
import { formatCurrency } from "@/lib/format";
import { DisputeEvidenceTimeline } from "@/components/dispute/DisputeEvidenceTimeline";

function fileToBase64(file: File): Promise<string> {
  return new Promise((resolve, reject) => {
    const reader = new FileReader();
    reader.onload = () => {
      const result = reader.result as string;
      resolve(result.split(",")[1] ?? result);
    };
    reader.onerror = reject;
    reader.readAsDataURL(file);
  });
}

export default function SellerDisputes() {
  const { t } = useTranslation();
  const queryClient = useQueryClient();
  const [tab, setTab] = useState<"active" | "resolved">("active");
  const [uploadingId, setUploadingId] = useState<string | null>(null);
  const [analyzingId, setAnalyzingId] = useState<string | null>(null);
  const [messageInputs, setMessageInputs] = useState<Record<string, string>>({});

  const { data: ordersData, isLoading: loadingOrders } = useQuery({
    queryKey: ["orders", "seller"],
    queryFn: () => api.listOrders("seller"),
  });

  // Disputas continuam relevantes mesmo depois que o pedido sai de in_custody —
  // a resolução via IA/admin (refund/release) marca o pedido como completed.
  const candidateOrders = (ordersData?.orders ?? []).filter(
    (o) => o.status === "in_custody" || o.status === "completed",
  );

  // Fetch dispute for each candidate order — shares cache with AppOrderDetail
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

  // Pairs where dispute actually exists
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
  const resolvedCount = resolvedPairs.length;
  const visiblePairs = tab === "active" ? activePairs : resolvedPairs;

  const refetchDispute = (orderId: string) =>
    queryClient.invalidateQueries({ queryKey: ["dispute", orderId] });

  const handleFiles = async (
    orderId: string,
    evidenceCount: number,
    e: React.ChangeEvent<HTMLInputElement>,
  ) => {
    const files = Array.from(e.target.files ?? []).slice(0, Math.max(0, 10 - evidenceCount));
    e.target.value = "";
    if (!files.length) return;
    setUploadingId(orderId);
    try {
      for (const file of files) {
        const base64 = await fileToBase64(file);
        const kind = file.type.startsWith("video/") ? "video" : "photo";
        const ext = file.name.split(".").pop();
        await api.addDisputeEvidence(orderId, kind, base64, ext);
      }
      toast.success(t("order.toastEvidenceAdded", "Evidência adicionada."));
      refetchDispute(orderId);
    } catch (err: unknown) {
      const apiErr = err as ApiError;
      toast.error(apiErr?.error ?? "Erro ao adicionar evidência");
    } finally {
      setUploadingId(null);
    }
  };

  const addMessage = async (orderId: string) => {
    const value = (messageInputs[orderId] ?? "").trim();
    if (!value) return;
    setUploadingId(orderId);
    try {
      await api.addDisputeEvidence(orderId, "message", value);
      setMessageInputs((prev) => ({ ...prev, [orderId]: "" }));
      toast.success(t("order.toastEvidenceAdded", "Evidência adicionada."));
      refetchDispute(orderId);
    } catch (err: unknown) {
      const apiErr = err as ApiError;
      toast.error(apiErr?.error ?? "Erro ao adicionar evidência");
    } finally {
      setUploadingId(null);
    }
  };

  const finishAndAnalyze = async (orderId: string) => {
    setAnalyzingId(orderId);
    try {
      await api.analyzeDispute(orderId);
      toast.success(
        t("order.toastAnalysisStarted", "Enviado para análise. Você será avisado quando sair um resultado."),
      );
      refetchDispute(orderId);
    } catch (err: unknown) {
      const apiErr = err as ApiError;
      toast.error(apiErr?.error ?? t("order.toastAnalysisError", "Erro ao enviar para análise"));
    } finally {
      setAnalyzingId(null);
    }
  };

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
      <h1 className="font-display text-2xl font-bold">{t("seller.disputesTitle")}</h1>

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
          <p className="text-xl font-display font-bold text-secondary">{resolvedCount}</p>
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
            <p className="font-semibold">{t("seller.noDisputes")}</p>
            <p className="text-sm text-muted-foreground">{t("seller.noDisputesDesc")}</p>
          </div>
        ) : (
          visiblePairs.map(({ order, dispute }) => {
            const canRespond = dispute.status === "open" || dispute.status === "under_review";
            const verdictText =
              dispute.ai_verdict === "favor_buyer"
                ? t("order.aiVerdictBuyer", "IA decidiu a favor do comprador")
                : dispute.ai_verdict === "favor_seller"
                  ? t("order.aiVerdictSeller", "IA decidiu a favor do vendedor")
                  : dispute.ai_verdict === "inconclusive"
                    ? t("order.aiVerdictInconclusive", "IA inconclusiva — revisão manual")
                    : null;
            const verdictColor =
              dispute.ai_verdict === "favor_seller"
                ? "text-secondary"
                : dispute.ai_verdict === "favor_buyer"
                  ? "text-destructive"
                  : "text-amber-600";
            const isBusy = uploadingId === order.id;

            return (
              <div key={order.id} className="bg-card rounded-xl p-4 border border-border space-y-3">
                <div className="flex items-start justify-between gap-2">
                  <div className="flex items-center gap-3 min-w-0">
                    <div className="h-10 w-10 rounded-xl bg-destructive/10 flex items-center justify-center flex-shrink-0">
                      <AlertTriangle className="h-5 w-5 text-destructive" />
                    </div>
                    <div className="min-w-0">
                      <p className="font-semibold text-sm truncate">{order.description ?? "Pedido"}</p>
                      <p className="text-xs text-muted-foreground">#{order.id.slice(0, 8)}</p>
                    </div>
                  </div>
                  <span className={`text-[10px] font-semibold px-2 py-0.5 rounded-full flex-shrink-0 ${statusColor[dispute.status] ?? statusColor.closed}`}>
                    {statusLabel[dispute.status] ?? dispute.status}
                  </span>
                </div>

                <div className="bg-muted/50 rounded-lg p-3">
                  <p className="text-xs font-medium text-muted-foreground mb-1">Valor em disputa</p>
                  <p className="text-sm font-semibold">{formatCurrency(parseFloat(order.amount))}</p>
                </div>

                {order.risk_score != null && (
                  <div className="flex items-center justify-between bg-muted/30 rounded-lg px-3 py-2">
                    <span className="text-xs text-muted-foreground">Score de risco</span>
                    <span className={`text-xs font-semibold px-2 py-0.5 rounded-full ${
                      order.risk_decision === "approve"
                        ? "text-secondary bg-secondary/10"
                        : order.risk_decision === "block"
                          ? "text-destructive bg-destructive/10"
                          : "text-amber-600 bg-amber-50"
                    }`}>
                      {order.risk_score} / 1000
                    </span>
                  </div>
                )}

                {/* Veredito da IA */}
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
                  <DisputeEvidenceTimeline evidence={dispute.evidence} currentParty="seller" />
                </div>

                {canRespond && (
                  <div className="border-t border-border pt-3 space-y-2">
                    <p className="text-xs font-semibold text-foreground">{t("seller.respond")}</p>
                    <p className="text-[11px] text-muted-foreground">
                      {t("order.addEvidenceHint", "Envie fotos, vídeos ou o código de rastreio. Você pode enviar aos poucos.")}
                    </p>
                    <label className="flex items-center justify-center gap-2 border-2 border-dashed border-border rounded-xl p-3 cursor-pointer hover:bg-muted/50 transition text-xs text-muted-foreground">
                      <Paperclip className="h-3.5 w-3.5" />
                      <span>Nota fiscal, comprovante de envio, fotos</span>
                      <input
                        type="file"
                        accept="image/*,video/*"
                        multiple
                        className="hidden"
                        onChange={(e) => handleFiles(order.id, dispute.evidence.length, e)}
                        disabled={isBusy || dispute.evidence.length >= 10}
                      />
                    </label>
                    <div className="flex gap-2">
                      <Input
                        value={messageInputs[order.id] ?? ""}
                        onChange={(e) =>
                          setMessageInputs((prev) => ({ ...prev, [order.id]: e.target.value }))
                        }
                        placeholder={t("seller.respondPlaceholder")}
                        className="h-9 text-xs"
                        disabled={isBusy}
                      />
                      <Button
                        type="button"
                        size="sm"
                        variant="outline"
                        onClick={() => addMessage(order.id)}
                        disabled={isBusy || !(messageInputs[order.id] ?? "").trim()}
                      >
                        +
                      </Button>
                    </div>
                    <Button
                      type="button"
                      className="w-full h-10 rounded-xl vault-card border-0 text-white text-xs font-semibold hover:opacity-90"
                      onClick={() => finishAndAnalyze(order.id)}
                      disabled={analyzingId === order.id || isBusy}
                    >
                      {analyzingId === order.id ? <Loader2 className="mr-2 h-3.5 w-3.5 animate-spin" /> : null}
                      {t("order.finishAndAnalyze", "Concluir contestação e enviar para análise")}
                    </Button>
                  </div>
                )}
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
