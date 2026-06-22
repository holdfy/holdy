import { useState, useEffect } from "react";
import { ArrowLeft, Copy, Check, Shield, Loader2, Link2, CheckCircle2, User } from "lucide-react";
import { Link } from "react-router-dom";
import { useTranslation } from "react-i18next";
import { useMutation } from "@tanstack/react-query";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Textarea } from "@/components/ui/textarea";
import { toast } from "sonner";
import { api } from "@/lib/api-client";
import type { ApiError, ImportedProductDraft } from "@/lib/api-client";
import { useUserRole } from "@/contexts/UserRoleContext";

export default function SellerNewProposal() {
  const { t } = useTranslation();
  const { user } = useUserRole();
  const sellerDoc = user?.document?.replace(/\D/g, "") ?? "";
  const [sellerKyc, setSellerKyc] = useState<{ name: string | null; situation: string | null; loading: boolean; error: boolean }>(
    { name: null, situation: null, loading: false, error: false }
  );
  const [amount, setAmount] = useState("");
  const [description, setDescription] = useState("");
  const [pixKey, setPixKey] = useState("");
  const [listingUrl, setListingUrl] = useState("");
  const [listingId, setListingId] = useState<string | undefined>(undefined);
  const [listingImported, setListingImported] = useState(false);
  const [isImporting, setIsImporting] = useState(false);
  const [importedDraft, setImportedDraft] = useState<ImportedProductDraft | null>(null);
  const [isPreviewLoading, setIsPreviewLoading] = useState(false);
  const [proposalLink, setProposalLink] = useState<string | null>(null);
  const [copied, setCopied] = useState(false);

  useEffect(() => {
    if (sellerDoc.length < 11) return;
    setSellerKyc(prev => ({ ...prev, loading: true }));
    api.lookupDocument(sellerDoc)
      .then(r => setSellerKyc({ name: r.name, situation: r.situation, loading: false, error: false }))
      .catch(() => setSellerKyc({ name: null, situation: null, loading: false, error: true }));
  }, [sellerDoc]);

  const handlePreviewListing = async () => {
    const url = listingUrl.trim();
    if (!url) return;
    setIsPreviewLoading(true);
    setImportedDraft(null);
    try {
      const draft = await api.importListing(url);
      setImportedDraft(draft);
      setListingId(draft.listing_id ?? undefined);
      setListingImported(true);
      if (!amount.trim() && draft.price_suggested) setAmount(draft.price_suggested);
      if (!description.trim() && draft.title) setDescription(draft.title);
      toast.success(t("seller.listingImported", "Anúncio importado!"));
    } catch {
      toast.error(t("seller.importError", "Erro ao importar produto"));
    } finally {
      setIsPreviewLoading(false);
    }
  };

  const createMutation = useMutation({
    mutationFn: async () => {
      let resolvedListingId = listingId;

      // Import listing if URL provided and not yet imported
      if (listingUrl.trim() && !resolvedListingId) {
        setIsImporting(true);
        try {
          const imported = await api.importListing(listingUrl.trim());
          resolvedListingId = imported.listing_id ?? undefined;
          if (!amount.trim() && imported.price_suggested) {
            setAmount(imported.price_suggested);
          }
          if (!description.trim() && imported.title) {
            setDescription(imported.title);
          }
          setListingId(resolvedListingId);
          setListingImported(true);
        } catch {
          toast.error(t("seller.importError", "Erro ao importar produto"));
        } finally {
          setIsImporting(false);
        }
      }

      return api.createProposal({
        amount: amount.trim().replace(",", "."),
        description: description.trim() || undefined,
        seller_pix_key: pixKey.trim() || undefined,
        listing_id: resolvedListingId,
      });
    },
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

  const isPending = createMutation.isPending || isImporting || isPreviewLoading;
  const pendingLabel = isImporting
    ? t("seller.importingListing", "Importando anúncio...")
    : t("buyer.addFunds", "Criar um Pagamento Seguro");

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

      {/* Card do vendedor — dados do usuário logado via JWT + RF */}
      {sellerDoc.length >= 11 && (
        <div className="bg-card rounded-2xl p-4 border border-border flex items-start gap-3">
          <div className="h-10 w-10 rounded-full bg-primary/10 flex items-center justify-center shrink-0">
            <User className="h-5 w-5 text-primary" />
          </div>
          <div className="flex-1 min-w-0">
            <p className="text-xs font-semibold tracking-wider text-muted-foreground uppercase mb-0.5">
              {t("seller.yourData", "Você (Vendedor)")}
            </p>
            {sellerKyc.loading ? (
              <div className="flex items-center gap-2 text-xs text-muted-foreground">
                <Loader2 className="h-3 w-3 animate-spin" />
                <span>{t("seller.kycVerifying", "Verificando seus dados...")}</span>
              </div>
            ) : sellerKyc.name ? (
              <>
                <p className="font-semibold text-sm text-foreground truncate">{sellerKyc.name}</p>
                <p className="text-xs text-muted-foreground mt-0.5">
                  {sellerDoc.length === 11
                    ? sellerDoc.replace(/(\d{3})(\d{3})(\d{3})(\d{2})/, "$1.$2.$3-$4")
                    : sellerDoc.replace(/(\d{2})(\d{3})(\d{3})(\d{4})(\d{2})/, "$1.$2.$3/$4-$5")}
                </p>
                {sellerKyc.situation && (
                  <p className="text-xs text-muted-foreground">
                    {t("seller.kycSituation", "Situação")}: {sellerKyc.situation}
                  </p>
                )}
              </>
            ) : (
              <p className="text-sm text-muted-foreground">
                {sellerKyc.error ? t("seller.kycError", "Dados não disponíveis") : sellerDoc}
              </p>
            )}
          </div>
        </div>
      )}

      {!proposalLink ? (
        <div className="bg-card rounded-2xl p-6 border border-border space-y-5">
          <div className="space-y-2">
            <Label htmlFor="listingUrl">
              {t("seller.listingUrl", "Link do Anúncio")}
              <span className="text-muted-foreground text-xs ml-1">({t("common.optional", "opcional")})</span>
            </Label>
            <div className="flex gap-2">
              <div className="relative flex-1">
                <Link2 className="absolute left-3 top-1/2 -translate-y-1/2 h-4 w-4 text-muted-foreground" />
                <Input
                  id="listingUrl"
                  placeholder={t("seller.listingUrlPlaceholder", "Cole o link (OLX, Mercado Livre, Instagram...)")}
                  value={listingUrl}
                  onChange={(e) => {
                    setListingUrl(e.target.value);
                    setListingId(undefined);
                    setListingImported(false);
                    setImportedDraft(null);
                  }}
                  className="pl-9"
                  autoComplete="off"
                />
                {listingImported && (
                  <CheckCircle2 className="absolute right-3 top-1/2 -translate-y-1/2 h-4 w-4 text-green-500" />
                )}
              </div>
              {listingUrl.trim() && !listingImported && (
                <Button
                  type="button"
                  variant="outline"
                  size="sm"
                  className="h-10 shrink-0 px-3"
                  onClick={handlePreviewListing}
                  disabled={isPreviewLoading}
                >
                  {isPreviewLoading
                    ? <Loader2 className="h-3.5 w-3.5 animate-spin" />
                    : t("seller.previewBtn", "Pré-visualizar")}
                </Button>
              )}
            </div>
            <p className="text-xs text-muted-foreground">
              {t("seller.listingUrlHint", "Título e valor serão preenchidos automaticamente.")}
            </p>

            {importedDraft && (
              <div className="rounded-xl border border-border overflow-hidden flex gap-0 bg-muted/30">
                {importedDraft.photos[0] && (
                  <img
                    src={importedDraft.photos[0]}
                    alt={importedDraft.title}
                    className="h-24 w-24 object-cover shrink-0"
                  />
                )}
                <div className="p-3 flex flex-col justify-center gap-1 min-w-0">
                  <p className="text-sm font-semibold line-clamp-2 text-foreground">{importedDraft.title}</p>
                  {importedDraft.price_suggested && (
                    <p className="text-xs text-primary font-medium">R$ {importedDraft.price_suggested}</p>
                  )}
                  <p className="text-[10px] text-muted-foreground capitalize">{importedDraft.source_platform}</p>
                </div>
              </div>
            )}
          </div>

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
            disabled={isPending}
          >
            {isPending ? (
              <><Loader2 className="h-4 w-4 animate-spin mr-2" />{pendingLabel}</>
            ) : (
              t("buyer.addFunds", "Criar um Pagamento Seguro")
            )}
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
              setListingUrl("");
              setListingId(undefined);
              setListingImported(false);
              setImportedDraft(null);
            }}
          >
            {t("seller.newProposal", "Criar outra proposta")}
          </Button>
        </div>
      )}
    </div>
  );
}
