import { useState, useEffect } from "react";
import { Shield, ArrowLeft, Copy, Clock, Lock, HelpCircle, Loader2, Truck, ChevronDown, ChevronUp, CheckCircle2, User } from "lucide-react";
import { Link, useLocation, useNavigate, useParams } from "react-router-dom";
import { useTranslation } from "react-i18next";
import { useMutation, useQuery } from "@tanstack/react-query";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog";
import { formatCurrency, maskCpfCnpj, maskPhone, stripDoc, validateCpf, validateCnpj } from "@/lib/format";
import { toast } from "sonner";
import { api } from "@/lib/api-client";
import type { ApiError, ShippingQuote } from "@/lib/api-client";
import QRCode from "qrcode";

interface PaymentRouteState {
  pixBrCode?: string;
  amount?: number | string;
  orderId?: string;
  description?: string;
  proposalId?: string;
}

interface DocInfo {
  name: string | null;
  situation: string | null;
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

export default function AppPayment() {
  const { t } = useTranslation();
  const location = useLocation();
  const navigate = useNavigate();
  const { proposalId: proposalIdParam } = useParams<{ proposalId?: string }>();
  const state = (location.state ?? {}) as PaymentRouteState;
  const [helpOpen, setHelpOpen] = useState(false);

  const [pixBrCode, setPixBrCode] = useState<string | null>(state.pixBrCode ?? null);
  const [amount, setAmount] = useState<number>(state.amount ? parseFloat(String(state.amount)) : 0);
  const [orderId, setOrderId] = useState<string | null>(state.orderId ?? null);
  const [qrDataUrl, setQrDataUrl] = useState<string | null>(null);
  const [isPaid, setIsPaid] = useState(false);

  const [proposalId, setProposalId] = useState(proposalIdParam ?? state.proposalId ?? "");
  const [cpf, setCpf] = useState("");
  const [cpfError, setCpfError] = useState<string | null>(null);
  const [docInfo, setDocInfo] = useState<DocInfo | null>(null);
  const [docCheckLoading, setDocCheckLoading] = useState(false);
  const [buyerPhone, setBuyerPhone] = useState("");

  const [freightOpen, setFreightOpen] = useState(false);
  const [cepDestino, setCepDestino] = useState("");
  const [freightQuotes, setFreightQuotes] = useState<ShippingQuote[]>([]);
  const [freightLoading, setFreightLoading] = useState(false);

  // Generate real QR code image whenever pixBrCode changes
  useEffect(() => {
    if (!pixBrCode) { setQrDataUrl(null); return; }
    QRCode.toDataURL(pixBrCode, {
      width: 220,
      margin: 2,
      color: { light: "#ffffff", dark: "#111111" },
    }).then(setQrDataUrl).catch(() => setQrDataUrl(null));
  }, [pixBrCode]);

  // Fetch proposal details to show product image (soft failure — buyer may not be authed)
  const { data: proposalData } = useQuery({
    queryKey: ["proposal-preview", proposalIdParam],
    queryFn: () => api.getProposal(proposalIdParam!),
    enabled: !!proposalIdParam && !pixBrCode,
    retry: false,
    staleTime: 60_000,
  });

  // Poll order status after proposal acceptance — auto-navigate when paid
  const { data: polledOrder } = useQuery({
    queryKey: ["order-payment-poll", orderId],
    queryFn: () => api.getOrder(orderId!),
    enabled: !!orderId && !isPaid,
    refetchInterval: 5000,
    staleTime: 0,
  });

  useEffect(() => {
    if (polledOrder?.status === "in_custody" || polledOrder?.status === "completed") {
      setIsPaid(true);
      toast.success(t("payment.paymentConfirmed", "Pagamento confirmado! Redirecionando..."));
      setTimeout(() => navigate(`/buyer/orders/${orderId}`), 1800);
    }
  }, [polledOrder?.status, orderId, navigate, t]);

  const handleFreightQuote = async () => {
    const cep = cepDestino.replace(/\D/g, "");
    if (cep.length !== 8) {
      toast.error("CEP inválido — informe 8 dígitos");
      return;
    }
    setFreightLoading(true);
    try {
      const res = await api.quoteShipping({
        from_postal_code: "01001000",
        to_postal_code: cep,
        weight_kg: "0.5",
        width_cm: 20,
        height_cm: 10,
        length_cm: 15,
      });
      setFreightQuotes(res.quotes);
    } catch {
      toast.error("Erro ao calcular frete — tente novamente");
    } finally {
      setFreightLoading(false);
    }
  };

  const handleCpfBlur = async () => {
    const digits = stripDoc(cpf);
    if (!digits) return;
    const valid = digits.length === 11 ? validateCpf(digits) : digits.length === 14 ? validateCnpj(digits) : false;
    if (!valid) {
      setCpfError(t("payment.invalidCpf", "Documento inválido — verifique os dígitos."));
      setDocInfo(null);
      return;
    }
    setCpfError(null);
    setDocCheckLoading(true);
    try {
      const result = await api.lookupDocument(digits);
      setDocInfo({ name: result.name, situation: result.situation });
    } catch {
      // soft error — Receita Federal lookup failed; don't block the flow
      setDocInfo(null);
    } finally {
      setDocCheckLoading(false);
    }
  };

  const acceptMutation = useMutation({
    mutationFn: (cpfDigits: string) =>
      api.acceptProposal(proposalId.trim(), cpfDigits, buyerPhone.replace(/\D/g, "") || undefined),
    onSuccess: (data) => {
      setPixBrCode(data.pix_br_code);
      setAmount(parseFloat(data.amount));
      setOrderId(data.order_id);
      toast.success(t("payment.proposalAccepted", "Proposta aceita! Pague o PIX abaixo."));
    },
    onError: (err: ApiError) => {
      toast.error(err?.error ?? t("payment.acceptError", "Erro ao aceitar proposta"));
    },
  });

  const handleAcceptProposal = () => {
    const digits = stripDoc(cpf);
    if (!digits) {
      setCpfError(t("payment.cpfRequired", "CPF ou CNPJ é obrigatório para verificação."));
      return;
    }
    const valid = digits.length === 11 ? validateCpf(digits) : digits.length === 14 ? validateCnpj(digits) : false;
    if (!valid) {
      setCpfError(t("payment.invalidCpf", "Documento inválido — verifique os dígitos."));
      return;
    }
    setCpfError(null);
    acceptMutation.mutate(digits);
  };

  const copyPixCode = () => {
    if (pixBrCode) {
      navigator.clipboard.writeText(pixBrCode).then(() => {
        toast.success(t("payment.copied"));
      });
    }
  };

  return (
    <div className="px-5 pt-6 space-y-5">
      <div className="flex items-center justify-between">
        <div className="flex items-center gap-3">
          <Link to="/buyer" className="h-10 w-10 rounded-full bg-muted flex items-center justify-center">
            <ArrowLeft className="h-5 w-5" />
          </Link>
          <span className="font-display font-bold text-lg">{t("common.holdfy")}</span>
        </div>
        <div className="h-10 w-10 rounded-full vault-card flex items-center justify-center">
          <Shield className="h-5 w-5 text-white" />
        </div>
      </div>

      <div>
        <p className="text-xs font-semibold tracking-wider text-muted-foreground uppercase mb-1">{t("payment.title")}</p>
        <h1 className="font-display text-2xl font-bold">{t("payment.title")}</h1>
        <p className="text-sm text-muted-foreground mt-1">{t("payment.helpDesc")}</p>
      </div>

      {/* Produto vinculado à proposta */}
      {!pixBrCode && proposalData?.listing_photo && (
        <div className="bg-card rounded-2xl overflow-hidden border border-border">
          <img
            src={proposalData.listing_photo}
            alt={proposalData.description ?? t("payment.productAlt", "Produto")}
            className="w-full h-48 object-cover"
          />
          {proposalData.description && (
            <div className="px-4 py-3">
              <p className="text-sm font-semibold line-clamp-2 text-foreground">{proposalData.description}</p>
              <p className="text-xs text-muted-foreground mt-0.5">
                R$ {parseFloat(proposalData.amount).toLocaleString("pt-BR", { minimumFractionDigits: 2 })}
              </p>
            </div>
          )}
        </div>
      )}

      {/* Etapa 1 — Formulário (antes do PIX ser gerado) */}
      {!pixBrCode && (
        <div className="bg-card rounded-2xl p-6 border border-border space-y-4">
          <p className="font-semibold text-sm">{t("payment.enterProposal", "Proposta de pagamento seguro")}</p>

          <div className="space-y-2">
            <Label htmlFor="proposal-id">{t("payment.proposalId", "ID da Proposta")}</Label>
            {proposalIdParam ? (
              <div className="flex items-center gap-2 px-3 py-2 bg-muted rounded-xl border border-border">
                <span className="font-mono text-xs text-muted-foreground truncate flex-1">{proposalId}</span>
                <span className="text-[10px] font-semibold text-primary bg-primary/10 px-2 py-0.5 rounded-full shrink-0">
                  {t("payment.autoFilled", "automático")}
                </span>
              </div>
            ) : (
              <Input
                id="proposal-id"
                placeholder="xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx"
                value={proposalId}
                onChange={(e) => setProposalId(e.target.value)}
                className="font-mono text-sm"
              />
            )}
          </div>

          {/* CPF/CNPJ obrigatório */}
          <div className="space-y-1.5">
            <Label htmlFor="cpf">
              {t("payment.cpfLabel", "CPF ou CNPJ")}
              <span className="ml-1 text-destructive text-xs font-semibold">*</span>
            </Label>
            <Input
              id="cpf"
              placeholder="000.000.000-00"
              value={cpf}
              onChange={(e) => {
                setCpf(maskCpfCnpj(e.target.value));
                setCpfError(null);
                setDocInfo(null);
              }}
              onBlur={handleCpfBlur}
              inputMode="numeric"
              maxLength={18}
              className={cpfError ? "border-destructive" : ""}
            />
            {cpfError && (
              <p className="text-xs text-destructive">{cpfError}</p>
            )}
            {docCheckLoading && (
              <div className="flex items-center gap-2 text-xs text-muted-foreground">
                <Loader2 className="h-3 w-3 animate-spin" />
                <span>Consultando Receita Federal…</span>
              </div>
            )}
            {docInfo && !docCheckLoading && (
              <div className="flex items-start gap-2 bg-secondary/5 border border-secondary/20 rounded-xl p-3">
                <User className="h-4 w-4 text-secondary mt-0.5 shrink-0" />
                <div className="text-xs space-y-0.5">
                  {docInfo.name && (
                    <p className="font-semibold text-foreground">{docInfo.name}</p>
                  )}
                  {docInfo.situation && (
                    <p className="text-muted-foreground">Situação: {docInfo.situation}</p>
                  )}
                  {!docInfo.name && !docInfo.situation && (
                    <p className="text-muted-foreground">Documento aceito. Receita Federal não retornou dados.</p>
                  )}
                </div>
              </div>
            )}
          </div>

          {/* WhatsApp do comprador — notificações de rastreio */}
          <div className="space-y-1.5">
            <Label htmlFor="buyer-phone" className="flex items-center gap-1.5">
              <span>WhatsApp</span>
              <span className="text-muted-foreground text-xs">({t("common.optional", "opcional")})</span>
            </Label>
            <Input
              id="buyer-phone"
              placeholder="(41) 99999-0000"
              value={buyerPhone}
              onChange={(e) => setBuyerPhone(maskPhone(e.target.value))}
              type="tel"
              inputMode="numeric"
            />
            <p className="text-xs text-muted-foreground">
              {t("payment.whatsappHint", "Receba atualizações de rastreio da entrega pelo WhatsApp.")}
            </p>
          </div>

          {/* Cotação de frete — opcional */}
          <div className="border border-border rounded-xl overflow-hidden">
            <button
              type="button"
              className="w-full flex items-center justify-between px-4 py-3 text-sm font-medium hover:bg-muted/50 transition"
              onClick={() => setFreightOpen((v) => !v)}
            >
              <span className="flex items-center gap-2">
                <Truck className="h-4 w-4 text-muted-foreground" />
                Estimar frete (opcional)
              </span>
              {freightOpen ? <ChevronUp className="h-4 w-4" /> : <ChevronDown className="h-4 w-4" />}
            </button>

            {freightOpen && (
              <div className="px-4 pb-4 space-y-3 border-t border-border">
                <div className="space-y-1 pt-3">
                  <Label htmlFor="cep-destino" className="text-xs">CEP de destino</Label>
                  <div className="flex gap-2">
                    <Input
                      id="cep-destino"
                      placeholder="00000-000"
                      value={cepDestino}
                      onChange={(e) => setCepDestino(e.target.value.replace(/\D/g, "").slice(0, 8))}
                      inputMode="numeric"
                      className="flex-1"
                    />
                    <Button
                      type="button"
                      variant="outline"
                      size="sm"
                      className="h-10 px-3 shrink-0"
                      onClick={handleFreightQuote}
                      disabled={freightLoading || cepDestino.replace(/\D/g, "").length !== 8}
                    >
                      {freightLoading ? <Loader2 className="h-3.5 w-3.5 animate-spin" /> : "Calcular"}
                    </Button>
                  </div>
                  <p className="text-[10px] text-muted-foreground">Estimativa para pacote padrão (~0,5 kg)</p>
                </div>

                {freightQuotes.length > 0 && (
                  <div className="space-y-1.5">
                    {freightQuotes.map((q, i) => (
                      <div key={i} className="flex items-center justify-between bg-muted/40 rounded-lg px-3 py-2 text-sm">
                        <span className="text-muted-foreground font-medium">{q.carrier_label}</span>
                        <div className="text-right">
                          <span className="font-semibold">R$ {parseFloat(q.price_brl).toFixed(2)}</span>
                          <span className="text-xs text-muted-foreground ml-2">{q.estimated_days}d úteis</span>
                        </div>
                      </div>
                    ))}
                  </div>
                )}
              </div>
            )}
          </div>

          <Button
            className="w-full h-12 rounded-xl vault-card border-0 text-white font-semibold hover:opacity-90"
            onClick={handleAcceptProposal}
            disabled={!proposalId.trim() || acceptMutation.isPending}
          >
            {acceptMutation.isPending ? <Loader2 className="h-4 w-4 animate-spin mr-2" /> : null}
            {t("payment.acceptProposal", "Aceitar Proposta e Gerar PIX")}
          </Button>
        </div>
      )}

      {/* Etapa 2 — PIX gerado + aguardando pagamento automático */}
      {pixBrCode && (
        <>
          {isPaid ? (
            <div className="bg-card rounded-2xl p-6 border border-secondary/30 text-center space-y-3">
              <CheckCircle2 className="h-12 w-12 text-secondary mx-auto" />
              <p className="font-display font-bold text-lg text-secondary">
                {t("payment.paymentConfirmed", "Pagamento confirmado!")}
              </p>
              <p className="text-sm text-muted-foreground">Redirecionando para o pedido…</p>
              <Loader2 className="h-5 w-5 animate-spin text-secondary mx-auto" />
            </div>
          ) : (
            <div className="bg-card rounded-2xl p-6 border border-border text-center space-y-5">
              <div>
                <p className="text-xs text-muted-foreground font-medium uppercase tracking-wider">{t("payment.amount")}</p>
                <p className="text-4xl font-display font-bold mt-1">{formatCurrency(amount)}</p>
              </div>

              {/* QR Code real */}
              <div className="mx-auto w-56 h-56 rounded-2xl bg-white flex items-center justify-center p-2 shadow-sm">
                {qrDataUrl ? (
                  <img src={qrDataUrl} alt="QR Code PIX" className="w-full h-full rounded-xl" />
                ) : (
                  <Loader2 className="h-8 w-8 animate-spin text-muted-foreground" />
                )}
              </div>

              <p className="text-xs text-muted-foreground">Escaneie o QR code acima no seu app de pagamento</p>

              <Button
                className="w-full h-12 rounded-xl vault-card border-0 text-white font-semibold hover:opacity-90"
                onClick={copyPixCode}
              >
                <Copy className="mr-2 h-4 w-4" />
                {t("payment.copyPaste")}
              </Button>

              <div className="flex items-center gap-3 bg-muted/50 rounded-xl p-3 text-left">
                <Loader2 className="h-4 w-4 animate-spin text-primary shrink-0" />
                <div>
                  <p className="text-xs font-semibold text-foreground">
                    {t("payment.waitingPayment", "Aguardando confirmação automática do pagamento")}
                  </p>
                  <p className="text-[10px] text-muted-foreground mt-0.5">
                    {t("payment.autoConfirmDesc", "Você será redirecionado automaticamente após o pagamento ser detectado.")}
                  </p>
                </div>
              </div>
            </div>
          )}

          {!isPaid && (
            <>
              <div className="bg-card rounded-2xl p-4 border border-border flex items-center gap-3">
                <div className="h-10 w-10 rounded-full bg-muted flex items-center justify-center flex-shrink-0">
                  <Clock className="h-5 w-5 text-muted-foreground animate-pulse" />
                </div>
                <div>
                  <p className="font-semibold text-sm">{t("payment.confirm")}</p>
                  <p className="text-xs text-muted-foreground">{t("payment.helpDesc")}</p>
                </div>
              </div>

              <div className="bg-card rounded-2xl p-4 border border-border flex items-start gap-3">
                <div className="h-10 w-10 rounded-xl bg-secondary/10 flex items-center justify-center flex-shrink-0">
                  <Lock className="h-5 w-5 text-secondary" />
                </div>
                <div>
                  <p className="font-semibold text-sm">{t("order.inCustody")}</p>
                  <p className="text-xs text-muted-foreground mt-0.5 leading-relaxed">{t("buyer.protectedDesc")}</p>
                </div>
              </div>
            </>
          )}
        </>
      )}

      <div className="flex items-center justify-center gap-2 py-2 text-xs text-muted-foreground">
        <button type="button" className="inline-flex items-center gap-2 hover:text-foreground transition" onClick={() => setHelpOpen(true)}>
          <HelpCircle className="h-3.5 w-3.5" />
          <span>{t("payment.help")}</span>
        </button>
      </div>

      <div className="h-4" />

      <Dialog open={helpOpen} onOpenChange={setHelpOpen}>
        <DialogContent>
          <DialogHeader>
            <DialogTitle>{t("payment.helpTitle")}</DialogTitle>
            <DialogDescription>{t("payment.helpDesc")}</DialogDescription>
          </DialogHeader>
          <DialogFooter>
            <Button type="button" onClick={() => setHelpOpen(false)}>
              {t("common.close")}
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>
    </div>
  );
}
