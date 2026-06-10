import { useState } from "react";
import { ArrowLeft, Copy, Check, Shield, Loader2 } from "lucide-react";
import { Link } from "react-router-dom";
import { useTranslation } from "react-i18next";
import { useMutation } from "@tanstack/react-query";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Textarea } from "@/components/ui/textarea";
import { toast } from "sonner";
import { api } from "@/lib/api-client";
import type { ApiError } from "@/lib/api-client";

export default function SellerNewProposal() {
  const { t } = useTranslation();
  const [amount, setAmount] = useState("");
  const [description, setDescription] = useState("");
  const [pixKey, setPixKey] = useState("");
  const [proposalLink, setProposalLink] = useState<string | null>(null);
  const [copied, setCopied] = useState(false);

  const createMutation = useMutation({
    mutationFn: () =>
      api.createProposal({
        amount: amount.trim().replace(",", "."),
        description: description.trim() || undefined,
        seller_pix_key: pixKey.trim() || undefined,
      }),
    onSuccess: (data) => {
      const link = `${window.location.origin}/buyer/payment/${data.id}`;
      setProposalLink(link);
      toast.success(t("seller.proposalCreated", "Proposta criada! Compartilhe o link com o comprador."));
    },
    onError: (err: ApiError) => {
      toast.error(err?.error ?? t("seller.proposalError", "Erro ao criar proposta"));
    },
  });

  const copyLink = () => {
    if (!proposalLink) return;
    navigator.clipboard.writeText(proposalLink).then(() => {
      setCopied(true);
      toast.success(t("common.copied", "Link copiado!"));
      setTimeout(() => setCopied(false), 2000);
    });
  };

  const handleCreate = () => {
    if (!amount.trim()) {
      toast.error(t("seller.amountRequired", "Informe o valor da proposta"));
      return;
    }
    const val = parseFloat(amount.trim().replace(",", "."));
    if (isNaN(val) || val <= 0) {
      toast.error(t("seller.invalidAmount", "Valor inválido"));
      return;
    }
    if (!pixKey.trim()) {
      toast.error(t("seller.pixKeyRequired", "Informe sua chave PIX para receber o pagamento"));
      return;
    }
    createMutation.mutate();
  };

  return (
    <div className="px-5 pt-6 space-y-5 max-w-lg mx-auto md:px-0 md:pt-0">
      <div className="flex items-center gap-3">
        <Link
          to="/seller"
          className="h-10 w-10 rounded-full bg-muted flex items-center justify-center"
        >
          <ArrowLeft className="h-4 w-4" />
        </Link>
        <div>
          <h1 className="font-display font-bold text-xl">
            {t("seller.newProposalTitle", "Criar Pagamento Seguro")}
          </h1>
          <p className="text-xs text-muted-foreground">
            {t("seller.newProposalDesc", "O ID da proposta é gerado automaticamente")}
          </p>
        </div>
      </div>

      {!proposalLink ? (
        <div className="bg-card rounded-2xl p-6 border border-border space-y-5">
          <div className="space-y-2">
            <Label htmlFor="amount">{t("seller.amount", "Valor (R$)")}</Label>
            <Input
              id="amount"
              placeholder="0,00"
              value={amount}
              onChange={(e) => setAmount(e.target.value)}
              inputMode="decimal"
              className="text-lg font-semibold"
            />
          </div>

          <div className="space-y-2">
            <Label htmlFor="description">
              {t("seller.description", "Descrição")}
              <span className="text-muted-foreground text-xs ml-1">({t("common.optional", "opcional")})</span>
            </Label>
            <Textarea
              id="description"
              placeholder={t("seller.descriptionPlaceholder", "Ex: Tênis Nike Air Max, tamanho 42")}
              value={description}
              onChange={(e) => setDescription(e.target.value)}
              rows={3}
              className="resize-none"
            />
          </div>

          <div className="space-y-2">
            <Label htmlFor="pixKey">
              {t("seller.pixKey", "Sua Chave PIX")}
              <span className="text-destructive text-xs ml-1">*</span>
            </Label>
            <Input
              id="pixKey"
              placeholder={t("seller.pixKeyPlaceholder", "CPF, e-mail, celular ou chave aleatória")}
              value={pixKey}
              onChange={(e) => setPixKey(e.target.value)}
              autoComplete="off"
            />
            <p className="text-xs text-muted-foreground">
              {t("seller.pixKeyHint", "O PIX será enviado automaticamente após o comprador confirmar o recebimento.")}
            </p>
          </div>

          <div className="bg-muted/50 rounded-xl p-3 flex items-start gap-2 text-xs text-muted-foreground">
            <Shield className="h-4 w-4 mt-0.5 shrink-0 text-primary" />
            <span>{t("seller.escrowNote", "O dinheiro fica em custódia até o comprador confirmar o recebimento.")}</span>
          </div>

          <Button
            className="w-full h-12 rounded-xl vault-card border-0 text-white font-semibold hover:opacity-90"
            onClick={handleCreate}
            disabled={createMutation.isPending}
          >
            {createMutation.isPending ? (
              <Loader2 className="h-4 w-4 animate-spin mr-2" />
            ) : null}
            {t("buyer.addFunds", "Criar um Pagamento Seguro")}
          </Button>
        </div>
      ) : (
        <div className="bg-card rounded-2xl p-6 border border-border space-y-5">
          <div className="text-center space-y-1">
            <div className="text-4xl mb-2">🎉</div>
            <h2 className="font-display font-bold text-lg">
              {t("seller.proposalReady", "Proposta criada!")}
            </h2>
            <p className="text-sm text-muted-foreground">
              {t("seller.shareLink", "Compartilhe o link abaixo com o comprador")}
            </p>
          </div>

          <div className="bg-muted rounded-xl p-3 break-all font-mono text-xs text-foreground select-all">
            {proposalLink}
          </div>

          <Button
            className="w-full h-12 rounded-xl"
            variant={copied ? "outline" : "default"}
            onClick={copyLink}
          >
            {copied ? (
              <><Check className="h-4 w-4 mr-2" />{t("common.copied", "Link copiado!")}</>
            ) : (
              <><Copy className="h-4 w-4 mr-2" />{t("seller.copyLink", "Copiar link")}</>
            )}
          </Button>

          <Button
            variant="ghost"
            className="w-full"
            onClick={() => {
              setProposalLink(null);
              setAmount("");
              setDescription("");
              setPixKey("");
            }}
          >
            {t("seller.newProposal", "Criar outra proposta")}
          </Button>
        </div>
      )}
    </div>
  );
}
