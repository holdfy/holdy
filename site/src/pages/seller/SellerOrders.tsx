import { ChevronRight, Plus, Link2, Loader2, Copy, ExternalLink, PackageSearch, Phone } from "lucide-react";
import { Link } from "react-router-dom";
import { useMemo, useState } from "react";
import { useTranslation } from "react-i18next";
import { useMutation, useQuery } from "@tanstack/react-query";
import { formatCurrency, maskPhone, maskCurrencyBR, unmaskCurrencyBR, decimalToMaskedBR } from "@/lib/format";
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
import { toast } from "sonner";
import { api } from "@/lib/api-client";
import type { ApiError, ImportedProductDraft, ProposalResponse } from "@/lib/api-client";

const statusFilters = ["ALL", "PENDING", "IN_CUSTODY", "COMPLETED", "CANCELLED"] as const;

// API returns lowercase: map to display keys
const apiStatusToFilter: Record<string, string> = {
  pending_funding: "PENDING",
  in_custody: "IN_CUSTODY",
  completed: "COMPLETED",
  cancelled: "CANCELLED",
  failed: "CANCELLED",
};

export default function SellerOrders() {
  const { t } = useTranslation();
  const [filter, setFilter] = useState<string>("ALL");
  const [propOpen, setPropOpen] = useState(false);
  const [importOpen, setImportOpen] = useState(false);
  const [buyerId, setBuyerId] = useState("");
  const [amount, setAmount] = useState("");
  const [description, setDescription] = useState("");
  const [importUrl, setImportUrl] = useState("");
  const [importedDraft, setImportedDraft] = useState<ImportedProductDraft | null>(null);
  const [createdProposal, setCreatedProposal] = useState<ProposalResponse | null>(null);
  const [pixKey, setPixKey] = useState("");
  const [sellerPhone, setSellerPhone] = useState("");

  const { data: ordersData, isLoading: ordersLoading } = useQuery({
    queryKey: ["seller-orders"],
    queryFn: () => api.listOrders("seller"),
    refetchInterval: 30_000,
  });

  const orders = ordersData?.orders ?? [];
  const filtered = filter === "ALL" ? orders : orders.filter((o) => apiStatusToFilter[o.status] === filter);

  const filterLabels = useMemo(
    () => ({
      ALL: t("seller.filterAll"),
      PENDING: t("seller.filterPending"),
      IN_CUSTODY: t("seller.filterInCustody"),
      COMPLETED: t("seller.filterCompleted"),
      CANCELLED: t("seller.filterCancelled"),
    }),
    [t],
  );

  const proposalMutation = useMutation({
    mutationFn: () =>
      api.createProposal({
        buyer_id: buyerId.trim() || undefined,
        amount: unmaskCurrencyBR(amount),
        description: description.trim() || undefined,
        seller_pix_key: pixKey.trim() || undefined,
        listing_id: importedDraft?.listing_id ?? undefined,
        seller_phone: sellerPhone.replace(/\D/g, "") || undefined,
      }),
    onSuccess: (data) => {
      setCreatedProposal(data);
      setBuyerId("");
      setAmount("");
      setDescription("");
      setPixKey("");
      setSellerPhone("");
      setImportedDraft(null);
    },
    onError: (err: ApiError) => {
      toast.error(err?.error ?? t("seller.proposalError", "Erro ao criar proposta"));
    },
  });

  const importMutation = useMutation({
    mutationFn: () => api.importListing(importUrl.trim()),
    onSuccess: (data) => {
      setImportedDraft(data);
      if (data.price_suggested) {
        setAmount(decimalToMaskedBR(data.price_suggested));
      }
      if (data.title) {
        setDescription(data.title);
      }
      toast.success(t("seller.importSuccess", "Produto importado com sucesso!"));
    },
    onError: (err: ApiError) => {
      toast.error(err?.error ?? t("seller.importError", "Erro ao importar produto"));
    },
  });

  const createProposal = () => {
    if (!amount.trim()) {
      toast.error(t("auth.toastFillFields"));
      return;
    }
    if (isNaN(parseFloat(unmaskCurrencyBR(amount)))) {
      toast.error(t("seller.invalidAmount", "Valor inválido"));
      return;
    }
    if (!pixKey.trim()) {
      toast.error(t("seller.pixKeyRequired", "Informe sua chave PIX para receber o pagamento"));
      return;
    }
    proposalMutation.mutate();
  };

  const copyProposalId = () => {
    if (createdProposal) {
      navigator.clipboard.writeText(createdProposal.id).then(() => {
        toast.success(t("seller.proposalIdCopied", "ID da proposta copiado"));
      });
    }
  };

  return (
    <div className="px-5 pt-6 space-y-5 md:px-0 md:pt-0">
      <div className="flex items-center justify-between">
        <h1 className="font-display text-2xl font-bold">{t("seller.ordersTitle")}</h1>
        <div className="flex items-center gap-2">
          <Button
            size="sm"
            variant="outline"
            className="text-xs"
            onClick={() => setImportOpen(true)}
          >
            <Link2 className="h-3.5 w-3.5 mr-1.5" />
            {t("seller.importProduct", "Importar Produto")}
          </Button>
          <Button size="sm" className="text-xs vault-card border-0 text-white" onClick={() => setPropOpen(true)}>
            <Plus className="h-3.5 w-3.5 mr-1.5" />
            {t("seller.newProposal", "Nova Proposta")}
          </Button>
        </div>
      </div>

      <div className="flex gap-2 overflow-x-auto pb-1 -mx-1 px-1">
        {statusFilters.map((s) => (
          <button
            key={s}
            onClick={() => setFilter(s)}
            className={`px-3 py-1.5 rounded-full text-xs font-semibold whitespace-nowrap transition ${
              filter === s ? "bg-primary text-primary-foreground" : "bg-muted text-muted-foreground hover:bg-muted/80"
            }`}
          >
            {filterLabels[s]}
          </button>
        ))}
      </div>

      {ordersLoading && (
        <div className="flex justify-center py-8">
          <Loader2 className="h-6 w-6 animate-spin text-muted-foreground" />
        </div>
      )}

      <div className="grid grid-cols-1 md:grid-cols-2 gap-3">
        {filtered.map((order) => {
          const statusKey = apiStatusToFilter[order.status] ?? order.status.toUpperCase();
          const shortId = order.id.slice(0, 8);
          const buyerInitials = order.buyer_id.slice(0, 2).toUpperCase();
          return (
            <Link
              key={order.id}
              to={`/seller/orders/${order.id}`}
              className="flex items-center gap-3 bg-card rounded-xl p-4 border border-border hover:border-primary/20 transition"
            >
              <div className="h-11 w-11 rounded-full bg-muted flex items-center justify-center text-sm font-semibold text-muted-foreground flex-shrink-0">
                {buyerInitials}
              </div>
              <div className="flex-1 min-w-0">
                <p className="font-semibold text-sm truncate">
                  {order.description ?? t("seller.order", "Pedido")}
                </p>
                <p className="text-xs text-muted-foreground mt-0.5 font-mono">#{shortId}</p>
              </div>
              <div className="text-right flex-shrink-0">
                <p className="text-sm font-semibold">{formatCurrency(Number(order.amount))}</p>
                <span
                  className={`text-[10px] font-semibold px-2 py-0.5 rounded-full ${
                    statusKey === "COMPLETED"
                      ? "text-secondary bg-secondary/10"
                      : statusKey === "CANCELLED"
                        ? "text-destructive bg-destructive/10"
                        : statusKey === "PENDING"
                          ? "text-amber-600 bg-amber-100"
                          : "text-primary bg-primary/10"
                  }`}
                >
                  {statusKey === "IN_CUSTODY" ? t("status.IN_CUSTODY_BADGE") : t(`status.${statusKey}`, statusKey)}
                </span>
              </div>
              <ChevronRight className="h-4 w-4 text-muted-foreground flex-shrink-0" />
            </Link>
          );
        })}
      </div>

      {!ordersLoading && filtered.length === 0 && (
        <div className="text-center py-12 text-muted-foreground">
          <p className="text-sm">{t("seller.noOrders")}</p>
        </div>
      )}

      {/* Nova Proposta */}
      <Dialog open={propOpen} onOpenChange={(open) => { setPropOpen(open); if (!open) setCreatedProposal(null); }}>
        <DialogContent>
          <DialogHeader>
            <DialogTitle>{t("seller.newProposal", "Nova Proposta")}</DialogTitle>
            <DialogDescription>
              {t("seller.newProposalDesc", "Crie uma proposta de escrow para o comprador aceitar.")}
            </DialogDescription>
          </DialogHeader>

          {createdProposal ? (
            <div className="space-y-4">
              <div className="bg-secondary/10 rounded-xl p-4 border border-secondary/20 space-y-2">
                <p className="text-sm font-semibold text-secondary">
                  {t("seller.proposalCreated", "Proposta criada com sucesso!")}
                </p>
                <p className="text-xs text-muted-foreground">
                  {t("seller.proposalShareId", "Compartilhe o ID abaixo com o comprador:")}
                </p>
                <div className="flex items-center gap-2">
                  <code className="text-xs font-mono bg-muted px-3 py-2 rounded-lg flex-1 break-all">
                    {createdProposal.id}
                  </code>
                  <Button size="sm" variant="outline" onClick={copyProposalId}>
                    <Copy className="h-3.5 w-3.5" />
                  </Button>
                </div>
                <p className="text-xs text-muted-foreground">
                  {t("seller.proposalAmount", "Valor:")} <strong>{createdProposal.amount}</strong>
                </p>
              </div>
            </div>
          ) : (
            <div className="space-y-3">
              <div className="space-y-1.5">
                <Label>{t("seller.buyerId", "ID do Comprador")} <span className="text-muted-foreground text-xs">({t("common.optional", "opcional")})</span></Label>
                <Input
                  placeholder="xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx"
                  value={buyerId}
                  onChange={(e) => setBuyerId(e.target.value)}
                  className="font-mono text-xs"
                />
              </div>
              <div className="space-y-1.5">
                <Label>{t("payment.amount", "Valor (R$)")}</Label>
                <Input
                  placeholder="0,00"
                  value={amount}
                  onChange={(e) => setAmount(maskCurrencyBR(e.target.value))}
                  type="text"
                  inputMode="decimal"
                />
              </div>
              <div className="space-y-1.5">
                <Label>{t("seller.descriptionOptional", "Descrição (opcional)")}</Label>
                <Input
                  placeholder={t("seller.descriptionPlaceholder", "Ex: Notebook Dell Inspiron")}
                  value={description}
                  onChange={(e) => setDescription(e.target.value)}
                />
              </div>
              <div className="space-y-1.5">
                <Label>
                  {t("seller.pixKey", "Sua Chave PIX")}
                  <span className="text-destructive text-xs ml-1">*</span>
                </Label>
                <Input
                  placeholder={t("seller.pixKeyPlaceholder", "CPF, e-mail, celular ou chave aleatória")}
                  value={pixKey}
                  onChange={(e) => setPixKey(e.target.value)}
                  autoComplete="off"
                />
                <p className="text-xs text-muted-foreground">
                  {t("seller.pixKeyHint", "PIX enviado automaticamente após confirmação do comprador.")}
                </p>
              </div>
              <div className="space-y-1.5">
                <Label className="flex items-center gap-1.5">
                  <Phone className="h-3.5 w-3.5 text-muted-foreground" />
                  {t("seller.whatsapp", "Seu WhatsApp")}
                  <span className="text-muted-foreground text-xs">({t("common.optional", "opcional")})</span>
                </Label>
                <Input
                  placeholder="(41) 99999-0000"
                  value={sellerPhone}
                  onChange={(e) => setSellerPhone(maskPhone(e.target.value))}
                  type="tel"
                  inputMode="numeric"
                />
                <p className="text-xs text-muted-foreground">
                  {t("seller.whatsappHint", "Receba notificações de rastreio da entrega pelo WhatsApp.")}
                </p>
              </div>
              {importedDraft && (
                <div className="bg-secondary/10 rounded-lg p-2.5 text-xs text-secondary font-medium">
                  {t("seller.listingLinked", "Anúncio importado será vinculado a este pedido.")}
                </div>
              )}
            </div>
          )}

          <DialogFooter>
            <Button variant="outline" onClick={() => { setPropOpen(false); setCreatedProposal(null); }}>
              {createdProposal ? t("common.close") : t("common.cancel")}
            </Button>
            {!createdProposal && (
              <Button
                className="vault-card border-0 text-white hover:opacity-90"
                onClick={createProposal}
                disabled={proposalMutation.isPending}
              >
                {proposalMutation.isPending && <Loader2 className="mr-2 h-4 w-4 animate-spin" />}
                {t("seller.createProposal", "Criar Proposta")}
              </Button>
            )}
          </DialogFooter>
        </DialogContent>
      </Dialog>

      {/* Importar Produto */}
      <Dialog open={importOpen} onOpenChange={(open) => { setImportOpen(open); if (!open) { setImportUrl(""); setImportedDraft(null); } }}>
        <DialogContent>
          <DialogHeader>
            <DialogTitle>{t("seller.importProduct", "Importar Produto")}</DialogTitle>
            <DialogDescription>
              {t("seller.importDesc", "Cole o link do produto (OLX, Mercado Livre, Instagram, TikTok, etc.)")}
            </DialogDescription>
          </DialogHeader>

          <div className="space-y-3">
            <div className="space-y-1.5">
              <Label>{t("seller.productUrl", "Link do Produto")}</Label>
              <div className="flex gap-2">
                <Input
                  placeholder="https://..."
                  value={importUrl}
                  onChange={(e) => { setImportUrl(e.target.value); setImportedDraft(null); }}
                  type="url"
                  className="flex-1"
                />
                <Button
                  variant="outline"
                  size="icon"
                  disabled={!importUrl}
                  onClick={() => window.open(importUrl, "_blank")}
                >
                  <ExternalLink className="h-4 w-4" />
                </Button>
              </div>
            </div>

            {importedDraft && (
              <div className="bg-secondary/10 rounded-xl p-4 border border-secondary/20 space-y-2">
                <div className="flex items-start gap-3">
                  {importedDraft.photos[0] && (
                    <img
                      src={importedDraft.photos[0]}
                      alt={importedDraft.title}
                      className="h-16 w-16 rounded-lg object-cover flex-shrink-0 border border-border"
                    />
                  )}
                  <div className="min-w-0">
                    <p className="text-sm font-semibold truncate">{importedDraft.title}</p>
                    {importedDraft.price_suggested && (
                      <p className="text-xs text-secondary font-medium mt-0.5">
                        R$ {importedDraft.price_suggested}
                      </p>
                    )}
                    {importedDraft.description && (
                      <p className="text-xs text-muted-foreground mt-1 line-clamp-2">
                        {importedDraft.description}
                      </p>
                    )}
                    <p className="text-[10px] text-muted-foreground/60 mt-1 uppercase tracking-wider">
                      via {importedDraft.extractor_used}
                    </p>
                  </div>
                </div>
              </div>
            )}

            {!importedDraft && !importMutation.isPending && (
              <div className="flex items-center gap-2 bg-muted rounded-xl p-3 text-xs text-muted-foreground">
                <PackageSearch className="h-4 w-4 flex-shrink-0" />
                {t("seller.importHint", "Título, fotos e preço serão extraídos automaticamente.")}
              </div>
            )}
          </div>

          <DialogFooter>
            <Button variant="outline" onClick={() => { setImportOpen(false); setImportUrl(""); setImportedDraft(null); }}>
              {t("common.cancel")}
            </Button>
            {!importedDraft ? (
              <Button
                className="vault-card border-0 text-white hover:opacity-90"
                onClick={() => importMutation.mutate()}
                disabled={!importUrl.trim() || importMutation.isPending}
              >
                {importMutation.isPending && <Loader2 className="mr-2 h-4 w-4 animate-spin" />}
                {t("seller.import", "Importar")}
              </Button>
            ) : (
              <Button
                className="vault-card border-0 text-white hover:opacity-90"
                onClick={() => {
                  setImportOpen(false);
                  setImportedDraft(null);
                  setPropOpen(true);
                }}
              >
                {t("seller.createProposalFromImport", "Criar Proposta")}
              </Button>
            )}
          </DialogFooter>
        </DialogContent>
      </Dialog>
    </div>
  );
}
