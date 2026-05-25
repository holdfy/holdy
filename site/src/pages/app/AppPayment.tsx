import { useState } from "react";
import { Shield, ArrowLeft, Copy, Clock, Lock, HelpCircle, QrCode, Loader2 } from "lucide-react";
import { Link, useLocation, useNavigate } from "react-router-dom";
import { useTranslation } from "react-i18next";
import { useMutation } from "@tanstack/react-query";
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
import { formatCurrency } from "@/lib/format";
import { toast } from "sonner";
import { api } from "@/lib/api-client";
import type { ApiError } from "@/lib/api-client";

interface PaymentRouteState {
  pixBrCode?: string;
  amount?: number | string;
  orderId?: string;
  description?: string;
  proposalId?: string;
}

export default function AppPayment() {
  const { t } = useTranslation();
  const location = useLocation();
  const navigate = useNavigate();
  const state = (location.state ?? {}) as PaymentRouteState;
  const [helpOpen, setHelpOpen] = useState(false);

  // Se já temos um código PIX (veio de aceitação de proposta ou link externo)
  const [pixBrCode, setPixBrCode] = useState<string | null>(state.pixBrCode ?? null);
  const [amount, setAmount] = useState<number>(state.amount ? parseFloat(String(state.amount)) : 0);
  const [orderId, setOrderId] = useState<string | null>(state.orderId ?? null);

  // Formulário de aceite de proposta
  const [proposalId, setProposalId] = useState(state.proposalId ?? "");
  const [cpf, setCpf] = useState("");

  const acceptMutation = useMutation({
    mutationFn: () => api.acceptProposal(proposalId.trim(), cpf.trim() || undefined),
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

  const copyPixCode = () => {
    if (pixBrCode) {
      navigator.clipboard.writeText(pixBrCode).then(() => {
        toast.success(t("payment.copied"));
      });
    }
  };

  const handleConfirmPaid = () => {
    if (orderId) {
      navigate(`/buyer/orders/${orderId}`);
    } else {
      navigate("/buyer/orders");
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

      {/* Etapa 1 — Formulário de proposta (antes do PIX ser gerado) */}
      {!pixBrCode && (
        <div className="bg-card rounded-2xl p-6 border border-border space-y-4">
          <p className="font-semibold text-sm">{t("payment.enterProposal", "Cole o ID da proposta recebida do vendedor")}</p>
          <div className="space-y-2">
            <Label htmlFor="proposal-id">{t("payment.proposalId", "ID da Proposta")}</Label>
            <Input
              id="proposal-id"
              placeholder="xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx"
              value={proposalId}
              onChange={(e) => setProposalId(e.target.value)}
              className="font-mono text-sm"
            />
          </div>
          <div className="space-y-2">
            <Label htmlFor="cpf">{t("payment.cpfOptional", "CPF (opcional — para verificação antifraude)")}</Label>
            <Input
              id="cpf"
              placeholder="000.000.000-00"
              value={cpf}
              onChange={(e) => setCpf(e.target.value.replace(/\D/g, "").slice(0, 11))}
              inputMode="numeric"
            />
          </div>
          <Button
            className="w-full h-12 rounded-xl vault-card border-0 text-white font-semibold hover:opacity-90"
            onClick={() => acceptMutation.mutate()}
            disabled={!proposalId.trim() || acceptMutation.isPending}
          >
            {acceptMutation.isPending ? (
              <Loader2 className="h-4 w-4 animate-spin mr-2" />
            ) : null}
            {t("payment.acceptProposal", "Aceitar Proposta e Gerar PIX")}
          </Button>
        </div>
      )}

      {/* Etapa 2 — Código PIX gerado */}
      {pixBrCode && (
        <>
          <div className="bg-card rounded-2xl p-6 border border-border text-center space-y-5">
            <div>
              <p className="text-xs text-muted-foreground font-medium uppercase tracking-wider">{t("payment.amount")}</p>
              <p className="text-4xl font-display font-bold mt-1">{formatCurrency(amount)}</p>
            </div>

            <div className="mx-auto w-48 h-48 rounded-2xl vault-card flex items-center justify-center">
              <div className="text-center text-white/80 px-4">
                <QrCode className="h-16 w-16 mx-auto mb-2 text-white" />
                <p className="text-[10px] font-mono break-all leading-tight text-white/60">
                  {pixBrCode.slice(0, 50)}…
                </p>
              </div>
            </div>

            <Button
              className="w-full h-12 rounded-xl vault-card border-0 text-white font-semibold hover:opacity-90"
              onClick={copyPixCode}
            >
              <Copy className="mr-2 h-4 w-4" />
              {t("payment.copyPaste")}
            </Button>

            <Button variant="outline" className="w-full h-12 rounded-xl" onClick={handleConfirmPaid}>
              {t("payment.alreadyPaid", "Já paguei — Ver pedido")}
            </Button>
          </div>

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
