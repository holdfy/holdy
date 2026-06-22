import { useState, useEffect } from "react";
import { Shield, ArrowLeft, Copy, Clock, Lock, HelpCircle, Loader2, CheckCircle2, User, Store, AlertCircle } from "lucide-react";
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
import type { ApiError } from "@/lib/api-client";
import QRCode from "qrcode";
import { useUserRole } from "@/contexts/UserRoleContext";

interface PaymentRouteState {
  pixBrCode?: string;
  amount?: number | string;
  orderId?: string;
  description?: string;
  proposalId?: string;
}

interface PartyInfo {
  name: string | null;
  situation: string | null;
  loading: boolean;
  error: boolean;
}

// Lookup CPF/CNPJ and return name+situation (null on failure)
async function lookupParty(doc: string): Promise<PartyInfo> {
  const digits = doc.replace(/\D/g, "");
  if (digits.length < 11) return { name: null, situation: null, loading: false, error: false };
  try {
    const result = await api.lookupDocument(digits);
    return { name: result.name, situation: result.situation, loading: false, error: false };
  } catch {
    return { name: null, situation: null, loading: false, error: true };
  }
}

function PartyCard({
  icon,
  label,
  document,
  info,
}: {
  icon: React.ReactNode;
  label: string;
  document: string;
  info: PartyInfo;
}) {
  const maskedDoc = document.replace(/\D/g, "").length === 11
    ? document.replace(/(\d{3})(\d{3})(\d{3})(\d{2})/, "$1.$2.$3-$4")
    : document.replace(/(\d{2})(\d{3})(\d{3})(\d{4})(\d{2})/, "$1.$2.$3/$4-$5");

  return (
    <div className="bg-card rounded-2xl p-4 border border-border flex items-start gap-3">
      <div className="h-10 w-10 rounded-full bg-primary/10 flex items-center justify-center shrink-0">
        {icon}
      </div>
      <div className="flex-1 min-w-0">
        <p className="text-xs font-semibold tracking-wider text-muted-foreground uppercase mb-0.5">{label}</p>
        {info.loading ? (
          <div className="flex items-center gap-2 text-xs text-muted-foreground">
            <Loader2 className="h-3 w-3 animate-spin" />
            <span>Consultando Receita Federal…</span>
          </div>
        ) : info.name ? (
          <>
            <p className="font-semibold text-sm text-foreground truncate">{info.name}</p>
            <p className="text-xs text-muted-foreground mt-0.5">{maskedDoc}</p>
            {info.situation && (
              <p className="text-xs text-muted-foreground">Situação: {info.situation}</p>
            )}
          </>
        ) : (
          <>
            <p className="text-sm text-foreground font-medium">{maskedDoc}</p>
            {info.error && (
              <p className="text-xs text-muted-foreground">Não foi possível consultar a Receita Federal</p>
            )}
          </>
        )}
      </div>
    </div>
  );
}

export default function AppPayment() {
  const { t } = useTranslation();
  const location = useLocation();
  const navigate = useNavigate();
  const { user } = useUserRole();
  const { proposalId: proposalIdParam } = useParams<{ proposalId?: string }>();
  const state = (location.state ?? {}) as PaymentRouteState;
  const [helpOpen, setHelpOpen] = useState(false);

  const [pixBrCode, setPixBrCode] = useState<string | null>(state.pixBrCode ?? null);
  const [amount, setAmount] = useState<number>(state.amount ? parseFloat(String(state.amount)) : 0);
  const [orderId, setOrderId] = useState<string | null>(state.orderId ?? null);
  const [qrDataUrl, setQrDataUrl] = useState<string | null>(null);
  const [isPaid, setIsPaid] = useState(false);

  const [proposalId] = useState(proposalIdParam ?? state.proposalId ?? "");

  // Buyer CPF — pre-filled from JWT if available
  const buyerDocFromJwt = user?.document?.replace(/\D/g, "") ?? "";
  const [cpf, setCpf] = useState(
    buyerDocFromJwt.length === 11
      ? buyerDocFromJwt.replace(/(\d{3})(\d{3})(\d{3})(\d{2})/, "$1.$2.$3-$4")
      : buyerDocFromJwt.length === 14
      ? buyerDocFromJwt.replace(/(\d{2})(\d{3})(\d{3})(\d{4})(\d{2})/, "$1.$2.$3/$4-$5")
      : ""
  );
  const [cpfError, setCpfError] = useState<string | null>(null);
  const [buyerPhone, setBuyerPhone] = useState("");

  // Card do comprador (fixo — dados do usuário logado via JWT, nunca sobrescrito pelo campo)
  const [buyerInfo, setBuyerInfo] = useState<PartyInfo>({ name: null, situation: null, loading: false, error: false });
  // Card do vendedor (dados da proposta via KYC)
  const [sellerInfo, setSellerInfo] = useState<PartyInfo>({ name: null, situation: null, loading: false, error: false });
  // KYC inline do campo CPF (separado do card — não mexe no buyerInfo)
  const [docInfo, setDocInfo] = useState<PartyInfo | null>(null);
  const [docLoading, setDocLoading] = useState(false);


  // Generate real QR code image whenever pixBrCode changes
  useEffect(() => {
    if (!pixBrCode) { setQrDataUrl(null); return; }
    QRCode.toDataURL(pixBrCode, {
      width: 220,
      margin: 2,
      color: { light: "#ffffff", dark: "#111111" },
    }).then(setQrDataUrl).catch(() => setQrDataUrl(null));
  }, [pixBrCode]);

  // Auto-lookup buyer KYC from JWT document on mount
  useEffect(() => {
    if (!buyerDocFromJwt || buyerDocFromJwt.length < 11) return;
    setBuyerInfo(prev => ({ ...prev, loading: true }));
    lookupParty(buyerDocFromJwt).then(setBuyerInfo);
  // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  // Fetch proposal details (product image + seller_document)
  const { data: proposalData } = useQuery({
    queryKey: ["proposal-preview", proposalIdParam],
    queryFn: () => api.getProposal(proposalIdParam!),
    enabled: !!proposalIdParam && !pixBrCode,
    retry: false,
    staleTime: 60_000,
  });

  // Auto-lookup seller KYC when proposal loads
  useEffect(() => {
    const sellerDoc = proposalData?.seller_document;
    if (!sellerDoc) return;
    setSellerInfo(prev => ({ ...prev, loading: true }));
    lookupParty(sellerDoc).then(setSellerInfo);
  }, [proposalData?.seller_document]);

  // Seller reputation
  const { data: sellerReputation } = useQuery({
    queryKey: ["reputation", proposalData?.seller_id],
    queryFn: () => api.getReputation(proposalData!.seller_id),
    enabled: !!proposalData?.seller_id && !pixBrCode,
    staleTime: 300_000,
    retry: false,
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
    // Só busca KYC inline se o CPF digitado for diferente do usuário logado
    // (evita duplicar o card que já mostra o usuário logado)
    if (digits === buyerDocFromJwt) { setDocInfo(null); return; }
    setDocLoading(true);
    setDocInfo(null);
    lookupParty(digits).then(info => { setDocInfo(info); setDocLoading(false); });
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
      setCpfError(t("payment.cpfRequired"));
      return;
    }
    const valid = digits.length === 11 ? validateCpf(digits) : digits.length === 14 ? validateCnpj(digits) : false;
    if (!valid) {
      setCpfError(t("payment.invalidCpf"));
      return;
    }
    setCpfError(null);
    if (buyerPhone.replace(/\D/g, "").length < 10) {
      toast.error(t("payment.whatsappRequired", "WhatsApp é obrigatório para acompanhar a entrega."));
      return;
    }
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

      {/* Cards das partes — comprador e vendedor */}
      {!pixBrCode && (
        <div className="space-y-2">
          {/* Comprador */}
          {(buyerDocFromJwt.length >= 11 || buyerInfo.name) && (
            <PartyCard
              icon={<User className="h-5 w-5 text-primary" />}
              label={t("payment.partyBuyer", "Você (Comprador)")}
              document={buyerDocFromJwt || stripDoc(cpf)}
              info={buyerInfo}
            />
          )}

          {/* Vendedor */}
          {proposalData?.seller_document && (
            <PartyCard
              icon={<Store className="h-5 w-5 text-secondary" />}
              label={t("payment.partySeller", "Vendedor")}
              document={proposalData.seller_document}
              info={sellerInfo}
            />
          )}

          {/* Reputação do vendedor */}
          {proposalData?.seller_document && sellerReputation && (
            <div className="bg-muted/40 border border-border rounded-xl px-4 py-3 flex items-center justify-between">
              <div className="text-xs text-muted-foreground">
                <span className="font-semibold text-foreground">{sellerReputation.completed_transactions}</span>
                {" "}{t("payment.sellerTransactions", "transações concluídas")}
              </div>
              {sellerReputation.seal ? (
                <span className={`text-xs font-semibold px-2.5 py-1 rounded-full border ${
                  sellerReputation.seal.badge_color === "gold"
                    ? "text-amber-600 bg-amber-50 border-amber-200"
                    : sellerReputation.seal.badge_color === "green"
                    ? "text-secondary bg-secondary/10 border-secondary/20"
                    : "text-primary bg-primary/10 border-primary/20"
                }`}>
                  {sellerReputation.seal.label}
                </span>
              ) : sellerReputation.kyc_approved ? (
                <span className="text-xs font-semibold px-2.5 py-1 rounded-full border text-secondary bg-secondary/10 border-secondary/20">
                  KYC ✓
                </span>
              ) : null}
            </div>
          )}
        </div>
      )}

      {/* Etapa 1 — Formulário (antes do PIX ser gerado) */}
      {!pixBrCode && (
        <div className="bg-card rounded-2xl p-6 border border-border space-y-4">
          <p className="font-semibold text-sm">{t("payment.enterProposal", "Proposta de pagamento seguro")}</p>

          {/* CPF/CNPJ — pré-preenchido do JWT quando disponível */}
          <div className="space-y-1.5">
            <Label htmlFor="cpf">
              {t("payment.cpfLabel", "Seu CPF ou CNPJ")}
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
              <div className="flex items-center gap-1.5 text-xs text-destructive">
                <AlertCircle className="h-3 w-3 shrink-0" />
                <span>{cpfError}</span>
              </div>
            )}
            {docLoading && (
              <div className="flex items-center gap-2 text-xs text-muted-foreground">
                <Loader2 className="h-3 w-3 animate-spin" />
                <span>Consultando Receita Federal…</span>
              </div>
            )}
            {docInfo && !docLoading && (docInfo.name || docInfo.situation) && (
              <div className="flex items-start gap-2 bg-muted/40 border border-border rounded-xl p-3">
                <User className="h-4 w-4 text-muted-foreground mt-0.5 shrink-0" />
                <div className="text-xs space-y-0.5">
                  {docInfo.name && <p className="font-semibold text-foreground">{docInfo.name}</p>}
                  {docInfo.situation && <p className="text-muted-foreground">Situação: {docInfo.situation}</p>}
                </div>
              </div>
            )}
          </div>

          {/* WhatsApp do comprador — obrigatório para rastreio */}
          <div className="space-y-1.5">
            <Label htmlFor="buyer-phone" className="flex items-center gap-1.5">
              <span>WhatsApp</span>
              <span className="text-destructive text-xs font-semibold ml-1">*</span>
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
              {t("payment.whatsappHint")}
            </p>
          </div>

          <Button
            className="w-full h-12 rounded-xl vault-card border-0 text-white font-semibold hover:opacity-90"
            onClick={handleAcceptProposal}
            disabled={acceptMutation.isPending}
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
