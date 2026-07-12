import { useMemo, useState, useEffect, useRef } from "react";
import { Shield, ArrowLeft, CheckCircle2, HelpCircle, CheckCheck, Loader2, Paperclip, AlertTriangle, PackageSearch, Bot, User } from "lucide-react";
import { Link, useLocation, useNavigate, useParams } from "react-router-dom";
import { useTranslation } from "react-i18next";
import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog";
import {
  AlertDialog,
  AlertDialogAction,
  AlertDialogCancel,
  AlertDialogContent,
  AlertDialogDescription,
  AlertDialogFooter,
  AlertDialogHeader,
  AlertDialogTitle,
} from "@/components/ui/alert-dialog";
import { Textarea } from "@/components/ui/textarea";
import { Label } from "@/components/ui/label";
import { formatCurrency } from "@/lib/format";
import { toast } from "sonner";
import { api } from "@/lib/api-client";
import { useUserRole } from "@/contexts/UserRoleContext";
import type { ApiError, TrackingInfo, DisputeResponse } from "@/lib/api-client";
import { TrackingCard } from "@/components/TrackingCard";
import { DisputeEvidenceTimeline } from "@/components/dispute/DisputeEvidenceTimeline";

// Steps: 0=Pago 1=Retido(custódia) 2=Enviado 3=Entregue 4=Liberado.
// 2 e 3 vêm do status real da transportadora (simulador de rastreio), não só do escrow.
function orderStatusStep(status: string, trackingStatus?: string | null, hasTrackingCode?: boolean): number {
  if (status === "completed") return 4;
  if (status !== "in_custody") return 0;
  if (trackingStatus === "delivered") return 3;
  if (hasTrackingCode) return 2;
  return 1;
}

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

function DisputeCard({ dispute, isSeller, orderId }: { dispute: DisputeResponse; isSeller: boolean; orderId: string }) {
  const { t } = useTranslation();
  const queryClient = useQueryClient();
  const [uploading, setUploading] = useState(false);
  const [analyzing, setAnalyzing] = useState(false);
  const [trackingInput, setTrackingInput] = useState("");
  const currentParty: "buyer" | "seller" = isSeller ? "seller" : "buyer";
  const canAddEvidence = dispute.status === "open";

  const refetchDispute = () => queryClient.invalidateQueries({ queryKey: ["dispute", orderId] });

  const handleFiles = async (e: React.ChangeEvent<HTMLInputElement>) => {
    const files = Array.from(e.target.files ?? []).slice(0, Math.max(0, 5 - dispute.evidence.length));
    e.target.value = "";
    if (!files.length) return;
    setUploading(true);
    try {
      for (const file of files) {
        const base64 = await fileToBase64(file);
        const kind = file.type.startsWith("video/") ? "video" : "photo";
        const ext = file.name.split(".").pop();
        await api.addDisputeEvidence(orderId, kind, base64, ext);
      }
      toast.success(t("order.toastEvidenceAdded", "Evidência adicionada."));
      refetchDispute();
    } catch (err: unknown) {
      const apiErr = err as ApiError;
      toast.error(apiErr?.error ?? "Erro ao adicionar evidência");
    } finally {
      setUploading(false);
    }
  };

  const addTracking = async () => {
    if (!trackingInput.trim()) return;
    setUploading(true);
    try {
      await api.addDisputeEvidence(orderId, "tracking_code", trackingInput.trim());
      setTrackingInput("");
      toast.success(t("order.toastEvidenceAdded", "Evidência adicionada."));
      refetchDispute();
    } catch (err: unknown) {
      const apiErr = err as ApiError;
      toast.error(apiErr?.error ?? "Erro ao adicionar evidência");
    } finally {
      setUploading(false);
    }
  };

  const finishAndAnalyze = async () => {
    setAnalyzing(true);
    try {
      await api.analyzeDispute(orderId);
      toast.success(t("order.toastAnalysisStarted", "Enviado para análise. Você será avisado quando sair um resultado."));
      refetchDispute();
    } catch (err: unknown) {
      const apiErr = err as ApiError;
      toast.error(apiErr?.error ?? t("order.toastAnalysisError", "Erro ao enviar para análise"));
    } finally {
      setAnalyzing(false);
    }
  };

  const statusLabel: Record<string, string> = {
    open: t("order.disputeStatusOpen", "Aberta"),
    under_review: t("order.disputeStatusReview", "Em análise pela IA"),
    resolved: t("order.disputeStatusResolved", "Resolvida"),
    closed: t("order.disputeStatusClosed", "Encerrada"),
  };

  const statusColor: Record<string, string> = {
    open: "text-destructive bg-destructive/10 border-destructive/20",
    under_review: "text-amber-600 bg-amber-50 border-amber-200",
    resolved: "text-secondary bg-secondary/10 border-secondary/20",
    closed: "text-muted-foreground bg-muted border-border",
  };

  const verdictLabel = (v: string | null) => {
    if (v === "favor_buyer") return t("order.aiVerdictBuyer", "IA decidiu a favor do comprador");
    if (v === "favor_seller") return t("order.aiVerdictSeller", "IA decidiu a favor do vendedor");
    if (v === "inconclusive") return t("order.aiVerdictInconclusive", "IA inconclusiva — revisão manual em andamento");
    return null;
  };

  const verdictColor = (v: string | null) => {
    if (v === "favor_buyer") return isSeller ? "text-destructive" : "text-secondary";
    if (v === "favor_seller") return isSeller ? "text-secondary" : "text-destructive";
    return "text-amber-600";
  };

  return (
    <div className="bg-card rounded-2xl p-5 border border-border space-y-3">
      <div className="flex items-center justify-between">
        <div className="flex items-center gap-2">
          <AlertTriangle className="h-5 w-5 text-destructive" />
          <p className="font-semibold text-sm">{t("order.disputeCard", "Disputa em andamento")}</p>
        </div>
        <span className={`text-xs font-semibold px-3 py-1 rounded-full border ${statusColor[dispute.status] ?? statusColor.closed}`}>
          {statusLabel[dispute.status] ?? dispute.status}
        </span>
      </div>

      {dispute.ai_verdict && (
        <div className="bg-muted/40 rounded-xl p-3 flex items-start gap-3">
          <Bot className={`h-5 w-5 shrink-0 mt-0.5 ${verdictColor(dispute.ai_verdict)}`} />
          <div className="space-y-1">
            <p className={`text-sm font-semibold ${verdictColor(dispute.ai_verdict)}`}>
              {verdictLabel(dispute.ai_verdict)}
            </p>
            {dispute.ai_confidence != null && (
              <p className="text-xs text-muted-foreground">
                {t("order.aiConfidence", "Confiança da análise")}: {Math.round(dispute.ai_confidence * 100)}%
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

      <div className="border-t border-border pt-3 space-y-2">
        <DisputeEvidenceTimeline evidence={dispute.evidence} currentParty={currentParty} />
      </div>

      {canAddEvidence && (
        <div className="border-t border-border pt-3 space-y-2">
          <p className="text-xs font-semibold text-foreground">{t("order.addEvidence", "Adicionar evidência")}</p>
          <p className="text-[11px] text-muted-foreground">
            {t("order.addEvidenceHint", "Envie fotos, vídeos ou o código de rastreio. Você pode enviar aos poucos.")}
          </p>
          <label className="flex items-center justify-center gap-2 border-2 border-dashed border-border rounded-xl p-3 cursor-pointer hover:bg-muted/50 transition text-xs text-muted-foreground">
            <Paperclip className="h-3.5 w-3.5" />
            <span>Fotos ou vídeos</span>
            <input
              type="file"
              accept="image/*,video/*"
              multiple
              className="hidden"
              onChange={handleFiles}
              disabled={uploading || dispute.evidence.length >= 5}
            />
          </label>
          <div className="flex gap-2">
            <Input
              value={trackingInput}
              onChange={(e) => setTrackingInput(e.target.value)}
              placeholder={t("order.evidenceKindTracking", "Código de rastreio")}
              className="h-9 text-xs"
              disabled={uploading}
            />
            <Button
              type="button"
              size="sm"
              variant="outline"
              onClick={addTracking}
              disabled={uploading || !trackingInput.trim()}
            >
              +
            </Button>
          </div>
          <Button
            type="button"
            className="w-full h-10 rounded-xl vault-card border-0 text-white text-xs font-semibold hover:opacity-90"
            onClick={finishAndAnalyze}
            disabled={analyzing || uploading}
          >
            {analyzing ? <Loader2 className="mr-2 h-3.5 w-3.5 animate-spin" /> : null}
            {t("order.finishAndAnalyze", "Concluir e enviar para análise")}
          </Button>
        </div>
      )}

      {dispute.resolution_notes && (
        <div className="text-xs text-muted-foreground border-t border-border pt-3">
          <p className="font-semibold text-foreground mb-0.5">{t("order.resolutionNotes", "Notas da resolução")}</p>
          <p>{dispute.resolution_notes}</p>
        </div>
      )}
    </div>
  );
}

function RiskBadge({ score, decision }: { score: number | null; decision: string | null }) {
  if (score == null) return null;
  const isApprove = decision === "approve";
  const isBlock = decision === "block";
  const colorClass = isApprove
    ? "text-secondary bg-secondary/10 border-secondary/20"
    : isBlock
      ? "text-destructive bg-destructive/10 border-destructive/20"
      : "text-amber-600 bg-amber-50 border-amber-200";
  const label = isApprove ? "Aprovado" : isBlock ? "Bloqueado" : "Em revisão";

  return (
    <div className="bg-card rounded-2xl p-4 border border-border flex items-center justify-between">
      <div>
        <p className="text-xs text-muted-foreground font-medium mb-0.5">Score de Risco</p>
        <div className="flex items-center gap-2">
          <span className="text-lg font-display font-bold">{score}</span>
          <span className="text-xs text-muted-foreground">/ 1000</span>
        </div>
      </div>
      <span className={`text-xs font-semibold px-3 py-1.5 rounded-full border ${colorClass}`}>
        {label}
      </span>
    </div>
  );
}

export default function AppOrderDetail() {
  const { t } = useTranslation();
  const { id } = useParams<{ id: string }>();
  const location = useLocation();
  const navigate = useNavigate();
  const { user } = useUserRole();
  const queryClient = useQueryClient();
  const isSeller = location.pathname.startsWith("/seller");
  const ordersPath = isSeller ? "/seller/orders" : "/buyer/orders";
  const profilePath = isSeller ? "/seller/profile" : "/buyer/profile";

  const [disputeOpen, setDisputeOpen] = useState(false);
  const [disputeReason, setDisputeReason] = useState("");
  const [disputeNote, setDisputeNote] = useState("");
  const [disputeSubmitting, setDisputeSubmitting] = useState(false);
  const [confirmOpen, setConfirmOpen] = useState(false);
  const [releaseLoading, setReleaseLoading] = useState(false);
  const [trackingInput, setTrackingInput] = useState("");
  const [trackingResult, setTrackingResult] = useState<TrackingInfo | null>(null);

  const prevOrderStatus = useRef<string | undefined>(undefined);
  const prevTrackingStatus = useRef<string | undefined>(undefined);

  const setTrackingMutation = useMutation({
    mutationFn: (code: string) => api.setTracking(id!, code),
    onSuccess: () => {
      toast.success("Código de rastreio registrado!");
      setTrackingInput("");
    },
    onError: (err: ApiError) => toast.error(err?.error ?? "Erro ao registrar rastreio"),
  });

  const { data: order, isLoading, isError } = useQuery({
    queryKey: ["order", id],
    queryFn: () => api.getOrder(id!),
    enabled: !!id,
    retry: 1,
    refetchInterval: (query) => {
      const status = query.state.data?.status;
      if (status === "pending_funding") return 5000;
      if (status === "in_custody") return 30_000;
      return false;
    },
  });

  // Auto-poll tracking a cada 30s quando há código de rastreio
  const { data: autoTrackingData, refetch: refetchTracking, isFetching: trackingFetching } = useQuery({
    queryKey: ["tracking-auto", order?.tracking_code],
    queryFn: () => api.trackShipment(order!.tracking_code!),
    enabled: !!order?.tracking_code && order.status === "in_custody",
    refetchInterval: 30_000,
    staleTime: 25_000,
    retry: false,
  });

  // Busca disputa — 404 significa que não há disputa; falha silenciosa
  const { data: dispute } = useQuery<DisputeResponse | null>({
    queryKey: ["dispute", id],
    queryFn: async () => {
      try {
        return await api.getDispute(id!);
      } catch {
        return null;
      }
    },
    enabled: !!id && !!order,
    retry: false,
    staleTime: 30_000,
    refetchInterval: (query) => {
      const d = query.state.data;
      // Atualiza a cada 10s enquanto a disputa está em análise
      return d?.status === "open" || d?.status === "under_review" ? 10_000 : false;
    },
  });

  // Notificação browser quando pagamento confirmado (para vendedor)
  useEffect(() => {
    if (!order?.status) return;
    const prev = prevOrderStatus.current;
    prevOrderStatus.current = order.status;
    if (prev !== undefined && prev === "pending_funding" && order.status === "in_custody" && isSeller) {
      const fire = () => new Notification(
        t("notifications.paymentConfirmedTitle", "Pagamento confirmado!"),
        { body: t("notifications.paymentConfirmedBody", "Seu pedido foi pago e está em custódia."), icon: "/favicon.ico" }
      );
      if (!("Notification" in window)) return;
      if (Notification.permission === "granted") fire();
      else if (Notification.permission !== "denied") Notification.requestPermission().then(p => p === "granted" && fire());
    }
  }, [order?.status, isSeller, t]);

  // Notificação quando rastreio atualiza
  useEffect(() => {
    if (!autoTrackingData?.current_status) return;
    const prev = prevTrackingStatus.current;
    prevTrackingStatus.current = autoTrackingData.current_status;
    if (prev !== undefined && prev !== autoTrackingData.current_status) {
      toast.info(`${t("notifications.trackingUpdatedTitle", "Rastreio")}: ${autoTrackingData.current_status}`);
      if (!("Notification" in window)) return;
      const fire = () => new Notification(
        t("notifications.trackingUpdatedTitle", "Rastreio atualizado"),
        { body: autoTrackingData.current_status }
      );
      if (Notification.permission === "granted") fire();
      else if (Notification.permission !== "denied") Notification.requestPermission().then(p => p === "granted" && fire());
    }
  }, [autoTrackingData?.current_status, t]);

  const steps = useMemo(
    () => [
      { label: t("order.stepPaid"), desc: t("order.stepPaidDesc") },
      { label: t("order.stepCustody"), desc: t("order.stepCustodyDesc") },
      { label: t("order.stepShipped"), desc: t("order.stepShippedDesc") },
      { label: t("order.stepDelivered"), desc: t("order.stepDeliveredDesc") },
      { label: t("order.stepReleased"), desc: t("order.stepReleasedDesc") },
    ],
    [t],
  );

  const activeStep = order
    ? orderStatusStep(order.status, autoTrackingData?.current_status, !!order.tracking_code)
    : 0;

  const submitDispute = async () => {
    if (!disputeReason) {
      toast.error(t("order.toastDisputeReasonRequired", "Selecione o motivo da disputa"));
      return;
    }
    if (disputeReason === "other" && !disputeNote.trim()) {
      toast.error(t("order.toastDisputeEmpty"));
      return;
    }
    setDisputeSubmitting(true);
    const reasonMap: Record<string, string> = {
      different: t("order.disputeReasonDifferent", "Produto diferente do anunciado"),
      defective: t("order.disputeReasonDefective", "Produto com defeito"),
      damaged: t("order.disputeReasonDamaged", "Produto danificado na entrega"),
      not_received: t("order.disputeReasonNotReceived", "Não recebi o produto"),
      other: t("order.disputeReasonOther", "Outro"),
    };
    const reasonLabel = reasonMap[disputeReason] ?? disputeReason;
    const fullReason = disputeNote.trim() ? `${reasonLabel}: ${disputeNote.trim()}` : reasonLabel;
    try {
      await api.openDispute(id!, fullReason);
      queryClient.invalidateQueries({ queryKey: ["dispute", id] });

      toast.success(t("order.toastDisputeSent"));
      setDisputeOpen(false);
      setDisputeReason("");
      setDisputeNote("");
    } catch (err: unknown) {
      const apiErr = err as ApiError;
      toast.error(apiErr?.error ?? t("order.toastDisputeError", "Erro ao abrir disputa"));
    } finally {
      setDisputeSubmitting(false);
    }
  };

  const confirmDelivery = async () => {
    setConfirmOpen(false);
    setReleaseLoading(true);
    try {
      await api.releaseCustody({
        order_id: id!,
        released_by: user?.id ?? "",
        idempotency_key: `release-${id}-${Date.now()}`,
      });
      if (isSeller) {
        toast.success(t("order.toastReleaseSeller", "Confirmado! Seu PIX está sendo enviado para a chave cadastrada."));
      } else {
        toast.success(t("order.toastDeliveryConfirmed"));
      }
      navigate(isSeller ? "/seller/orders" : "/buyer/transaction-complete", {
        state: { orderId: id, amount: order?.amount },
      });
    } catch (err: unknown) {
      const apiErr = err as ApiError;
      toast.error(apiErr?.error ?? t("order.toastReleaseError", "Erro ao liberar pagamento"));
    } finally {
      setReleaseLoading(false);
    }
  };

  const displayAmount = order ? parseFloat(order.amount) : 0;
  const displayStatus = order?.status ?? "in_custody";
  const isInCustody = displayStatus === "in_custody";

  return (
    <div className="px-5 pt-6 space-y-5">
      <div className="flex items-center justify-between gap-2">
        <div className="flex items-center gap-3 min-w-0">
          <Link
            to={ordersPath}
            className="h-10 w-10 rounded-full bg-muted flex items-center justify-center flex-shrink-0"
            aria-label={t("common.backToOrders")}
          >
            <ArrowLeft className="h-5 w-5" />
          </Link>
          <div className="h-10 w-10 rounded-full vault-card flex items-center justify-center flex-shrink-0">
            <Shield className="h-5 w-5 text-white" />
          </div>
          <span className="font-display font-bold text-lg truncate">{t("common.holdfy")}</span>
        </div>
        <Link
          to={profilePath}
          className="h-10 w-10 rounded-full bg-muted flex items-center justify-center flex-shrink-0"
          aria-label={t("common.profile")}
        >
          <span className="text-sm font-semibold">👤</span>
        </Link>
      </div>

      {isLoading && (
        <div className="flex items-center justify-center py-12">
          <Loader2 className="h-8 w-8 animate-spin text-primary" />
        </div>
      )}

      {isError && (
        <div className="bg-card rounded-2xl p-5 border border-border text-center text-muted-foreground text-sm">
          {t("order.notFound", "Pedido não encontrado")}
        </div>
      )}

      {order && (
        <>
          <div className="bg-card rounded-2xl p-5 border border-border">
            <div className="flex items-center justify-between mb-1">
              <p className="text-xs font-semibold tracking-wider text-muted-foreground uppercase">
                ORDER #{id?.slice(0, 8).toUpperCase()}
              </p>
              <span
                className={`text-xs font-semibold px-3 py-1 rounded-full ${
                  isInCustody ? "text-secondary bg-secondary/10" : "text-primary bg-primary/10"
                }`}
              >
                {isInCustody ? t("status.IN_CUSTODY_BADGE") : t(`status.${displayStatus.toUpperCase()}`, displayStatus)}
              </span>
            </div>
            <h2 className="text-sm font-semibold text-foreground">{order.description ?? t("order.escrowOrder", "Pedido de escrow")}</h2>
          </div>

          <div className="vault-card rounded-2xl p-5">
            <p className="text-xs font-semibold tracking-[0.15em] text-white/50 uppercase mb-1">{t("order.protectedPayment")}</p>
            <div className="flex items-baseline gap-2">
              <span className="text-3xl font-display font-bold text-secondary">{formatCurrency(displayAmount)}</span>
              <span className="text-sm text-white/60">{t("order.inCustody")}</span>
            </div>
            <p className="text-xs text-white/50 mt-2 leading-relaxed">{t("order.paidPix")}</p>
          </div>

          <div className="bg-card rounded-2xl p-5 border border-border flex items-center justify-between">
            <div className="flex items-center gap-3">
              <Shield className="h-5 w-5 text-primary" />
              <div>
                <p className="text-xs text-muted-foreground font-medium">{t("order.paymentStatus")}</p>
                <div className="flex items-center gap-2 mt-0.5">
                  <span className="font-semibold">{t("order.heldEscrow")}</span>
                  <span className="text-xs bg-muted px-2 py-0.5 rounded-full text-muted-foreground">{t("common.holdfy")}</span>
                </div>
              </div>
            </div>
            <CheckCircle2 className="h-6 w-6 text-secondary" />
          </div>

          {/* Score de Risco */}
          <RiskBadge score={order.risk_score} decision={order.risk_decision} />

          {/* Dados do comprador — visíveis apenas para o vendedor */}
          {isSeller && (order.buyer_name || order.buyer_id) && (
            <div className="bg-card rounded-2xl p-4 border border-border flex items-center gap-3">
              <div className="h-10 w-10 rounded-full bg-muted flex items-center justify-center shrink-0">
                <User className="h-5 w-5 text-muted-foreground" />
              </div>
              <div>
                <p className="text-xs text-muted-foreground font-medium">{t("order.buyerData", "Dados do comprador")}</p>
                {order.buyer_name && <p className="font-semibold text-sm">{order.buyer_name}</p>}
                <p className="text-xs text-muted-foreground font-mono">{order.buyer_id.slice(0, 8).toUpperCase()}</p>
              </div>
            </div>
          )}

          {/* Rastreio */}
          {isSeller && isInCustody && (
            <div className="bg-card rounded-2xl p-5 border border-border space-y-3">
              <div className="flex items-center gap-2">
                <PackageSearch className="h-5 w-5 text-primary" />
                <p className="font-semibold text-sm">Informar código de rastreio</p>
              </div>
              <p className="text-xs text-muted-foreground">
                Após postar o produto, informe o código para o comprador acompanhar a entrega.
              </p>
              <div className="flex gap-2">
                <Input
                  placeholder="Ex.: AA123456789BR"
                  value={trackingInput}
                  onChange={(e) => setTrackingInput(e.target.value.toUpperCase())}
                  onKeyDown={(e) => e.key === "Enter" && trackingInput.trim() && setTrackingMutation.mutate(trackingInput.trim())}
                  className="font-mono uppercase flex-1"
                />
                <Button
                  size="sm"
                  onClick={() => setTrackingMutation.mutate(trackingInput.trim())}
                  disabled={!trackingInput.trim() || setTrackingMutation.isPending}
                  className="vault-card border-0 text-white hover:opacity-90 shrink-0"
                >
                  {setTrackingMutation.isPending ? <Loader2 className="h-4 w-4 animate-spin" /> : "Registrar"}
                </Button>
              </div>
            </div>
          )}

          {order.tracking_code && (
            <div className="bg-card rounded-2xl p-5 border border-border space-y-3">
              <div className="flex items-center justify-between">
                <div className="flex items-center gap-2">
                  <PackageSearch className="h-5 w-5 text-primary" />
                  <p className="font-semibold text-sm">Rastreio da encomenda</p>
                </div>
                <Button
                  variant="outline"
                  size="sm"
                  onClick={() => {
                    refetchTracking();
                    setTrackingResult(null);
                  }}
                  disabled={trackingFetching}
                >
                  {trackingFetching ? <Loader2 className="h-3.5 w-3.5 animate-spin" /> : "Atualizar"}
                </Button>
              </div>
              {!autoTrackingData && !trackingResult && !trackingFetching && (
                <p className="font-mono text-sm font-semibold">{order.tracking_code}</p>
              )}
              {trackingFetching && !autoTrackingData && (
                <div className="flex items-center gap-2 text-xs text-muted-foreground">
                  <Loader2 className="h-3.5 w-3.5 animate-spin" />
                  Consultando transportadora…
                </div>
              )}
              {(trackingResult ?? autoTrackingData) && (
                <TrackingCard info={(trackingResult ?? autoTrackingData)!} />
              )}
            </div>
          )}

          <div className="bg-card rounded-2xl p-5 border border-border">
            <h3 className="font-display font-bold text-lg mb-5">{t("order.deliveryProgress")}</h3>
            <div className="space-y-0">
              {steps.map((step, i) => {
                const status = i < activeStep ? "done" : i === activeStep ? "active" : "pending";
                return (
                  <div key={step.label} className="flex gap-4">
                    <div className="flex flex-col items-center">
                      <div
                        className={`h-8 w-8 rounded-full flex items-center justify-center flex-shrink-0 ${
                          status === "done"
                            ? "bg-secondary text-white"
                            : status === "active"
                              ? "bg-secondary/20 border-2 border-secondary"
                              : "bg-muted"
                        }`}
                      >
                        {status === "done" ? (
                          <CheckCircle2 className="h-4 w-4" />
                        ) : status === "active" ? (
                          <div className="h-2.5 w-2.5 rounded-full bg-secondary" />
                        ) : (
                          <div className="h-2.5 w-2.5 rounded-full bg-muted-foreground/30" />
                        )}
                      </div>
                      {i < steps.length - 1 && (
                        <div className={`w-0.5 h-10 ${status === "done" ? "bg-secondary" : "bg-border"}`} />
                      )}
                    </div>
                    <div className="pb-6">
                      <p
                        className={`font-semibold text-sm ${
                          status === "active"
                            ? "text-secondary"
                            : status === "pending"
                              ? "text-muted-foreground"
                              : ""
                        }`}
                      >
                        {step.label}
                      </p>
                      <p className="text-xs text-muted-foreground mt-0.5">{step.desc}</p>
                    </div>
                  </div>
                );
              })}
            </div>
          </div>

          {/* Card de disputa com veredito da IA */}
          {dispute && (
            <DisputeCard dispute={dispute} isSeller={isSeller} orderId={id!} />
          )}

          <div className="flex gap-3">
            <Button
              type="button"
              variant="outline"
              className="flex-1 h-14 rounded-xl text-sm font-semibold"
              onClick={() => setDisputeOpen(true)}
              disabled={!isInCustody || !!dispute}
            >
              <HelpCircle className="mr-2 h-4 w-4" />
              {t("order.openDispute")}
            </Button>
            {!isSeller && (
              <Button
                type="button"
                className="flex-1 h-14 rounded-xl vault-card border-0 text-white text-sm font-semibold hover:opacity-90"
                onClick={() => setConfirmOpen(true)}
                disabled={!isInCustody || releaseLoading}
              >
                {releaseLoading ? (
                  <Loader2 className="mr-2 h-4 w-4 animate-spin" />
                ) : (
                  <CheckCheck className="mr-2 h-4 w-4" />
                )}
                {t("order.confirmDelivery")}
              </Button>
            )}
          </div>
        </>
      )}

      <div className="h-4" />

      {/* Dispute dialog — motivo apenas; evidência é enviada depois, aos poucos, no card da disputa */}
      <Dialog open={disputeOpen} onOpenChange={(o) => { if (!disputeSubmitting) { setDisputeOpen(o); if (!o) { setDisputeReason(""); setDisputeNote(""); } } }}>
        <DialogContent>
          <DialogHeader>
            <DialogTitle>{t("order.disputeTitle")}</DialogTitle>
            <DialogDescription>{t("order.disputeDesc")}</DialogDescription>
          </DialogHeader>

          <div className="space-y-4">
            {/* Motivo da disputa — menu de opções */}
            <div className="space-y-2">
              <Label>{t("order.disputeReasonLabel", "Motivo da disputa")}</Label>
              <div className="space-y-2">
                {(
                  [
                    ["different", t("order.disputeReasonDifferent", "Produto diferente do anunciado")],
                    ["defective", t("order.disputeReasonDefective", "Produto com defeito")],
                    ["damaged", t("order.disputeReasonDamaged", "Produto danificado na entrega")],
                    ["not_received", t("order.disputeReasonNotReceived", "Não recebi o produto")],
                    ["other", t("order.disputeReasonOther", "Outro")],
                  ] as [string, string][]
                ).map(([value, label]) => (
                  <button
                    key={value}
                    type="button"
                    onClick={() => setDisputeReason(value)}
                    className={`w-full text-left px-4 py-2.5 rounded-xl border text-sm transition ${
                      disputeReason === value
                        ? "border-destructive bg-destructive/10 text-destructive font-semibold"
                        : "border-border bg-muted/30 text-foreground hover:bg-muted/60"
                    }`}
                  >
                    {label}
                  </button>
                ))}
              </div>
            </div>

            <div className="space-y-2">
              <Label htmlFor="dispute-note">{t("order.disputeDetails", "Detalhes adicionais (opcional)")}</Label>
              <Textarea
                id="dispute-note"
                placeholder={t("order.disputePlaceholder")}
                value={disputeNote}
                onChange={(e) => setDisputeNote(e.target.value)}
                rows={3}
              />
            </div>

            <div className="flex items-center gap-2 text-xs text-muted-foreground bg-muted/50 rounded-lg p-2">
              <AlertTriangle className="h-3.5 w-3.5 shrink-0" />
              <span>{t("order.addEvidenceHint", "Envie fotos, vídeos ou o código de rastreio. Você pode enviar aos poucos.")}</span>
            </div>
          </div>

          <DialogFooter className="gap-2">
            <Button type="button" variant="outline" onClick={() => setDisputeOpen(false)} disabled={disputeSubmitting}>
              {t("common.cancel")}
            </Button>
            <Button
              type="button"
              className="vault-card border-0 text-white hover:opacity-90"
              onClick={submitDispute}
              disabled={disputeSubmitting}
            >
              {disputeSubmitting ? <Loader2 className="mr-2 h-4 w-4 animate-spin" /> : null}
              {t("order.openDispute")}
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>

      {!isSeller && (
        <AlertDialog open={confirmOpen} onOpenChange={setConfirmOpen}>
          <AlertDialogContent>
            <AlertDialogHeader>
              <AlertDialogTitle>{t("order.confirmDelivery")}</AlertDialogTitle>
              <AlertDialogDescription>{t("order.paidPix")}</AlertDialogDescription>
            </AlertDialogHeader>
            <AlertDialogFooter>
              <AlertDialogCancel>{t("common.cancel")}</AlertDialogCancel>
              <AlertDialogAction
                className="vault-card border-0 text-white hover:opacity-90"
                onClick={(e) => {
                  e.preventDefault();
                  confirmDelivery();
                }}
              >
                {t("order.confirmDelivery")}
              </AlertDialogAction>
            </AlertDialogFooter>
          </AlertDialogContent>
        </AlertDialog>
      )}
    </div>
  );
}
