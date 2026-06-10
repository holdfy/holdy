import { useState } from "react";
import { AlertTriangle, MessageSquare, CheckCircle, Shield, Loader2, Paperclip, X } from "lucide-react";
import { useTranslation } from "react-i18next";
import { useQuery } from "@tanstack/react-query";
import { api } from "@/lib/api-client";
import type { ApiError } from "@/lib/api-client";
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

const DISPUTED_STATUSES = ["failed", "cancelled"];

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
  const [respondId, setRespondId] = useState<string | null>(null);
  const [message, setMessage] = useState("");
  const [responseFiles, setResponseFiles] = useState<File[]>([]);
  const [submitting, setSubmitting] = useState(false);

  const { data: ordersData, isLoading } = useQuery({
    queryKey: ["orders", "seller"],
    queryFn: () => api.listOrders("seller"),
  });

  const disputedOrders = (ordersData?.orders ?? []).filter((o) =>
    DISPUTED_STATUSES.includes(o.status),
  );

  const handleFileChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    const selected = Array.from(e.target.files ?? []).slice(0, 5);
    setResponseFiles((prev) => [...prev, ...selected].slice(0, 5));
    e.target.value = "";
  };

  const removeFile = (index: number) => {
    setResponseFiles((prev) => prev.filter((_, i) => i !== index));
  };

  const sendResponse = async () => {
    if (!message.trim() || !respondId) {
      toast.error(t("seller.toastRespondEmpty"));
      return;
    }
    setSubmitting(true);
    try {
      // Send text message as evidence with kind="message"
      await api.addDisputeEvidence(respondId, "message", message);

      // Upload any attached files
      for (const file of responseFiles) {
        const base64 = await fileToBase64(file);
        const kind = file.type.startsWith("video/") ? "video" : "photo";
        const ext = file.name.split(".").pop();
        await api.addDisputeEvidence(respondId, kind, base64, ext);
      }

      toast.success(t("seller.toastRespondSent"));
      setRespondId(null);
      setMessage("");
      setResponseFiles([]);
    } catch (err: unknown) {
      const apiErr = err as ApiError;
      toast.error(apiErr?.error ?? "Erro ao enviar resposta");
    } finally {
      setSubmitting(false);
    }
  };

  const openRespond = (orderId: string) => {
    setMessage("");
    setResponseFiles([]);
    setRespondId(orderId);
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
          <div className="flex justify-center py-8">
            <Loader2 className="h-5 w-5 animate-spin text-primary" />
          </div>
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
                    <p className="text-xs text-muted-foreground">#{order.id.slice(0, 8)}</p>
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

              {/* Risk score badge */}
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

              <div className="flex gap-2">
                <button
                  type="button"
                  className="flex-1 flex items-center justify-center gap-2 px-4 py-2.5 rounded-xl bg-primary text-primary-foreground text-sm font-semibold hover:opacity-90 transition"
                  onClick={() => openRespond(order.id)}
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

      {/* Resposta do vendedor com upload */}
      <Dialog open={!!respondId} onOpenChange={(o) => { if (!submitting) !o && setRespondId(null); }}>
        <DialogContent>
          <DialogHeader>
            <DialogTitle>{t("seller.respondTitle")}</DialogTitle>
            <DialogDescription>
              {t("seller.respondDesc")}
              {" "}Envie sua contestação com evidências para análise.
            </DialogDescription>
          </DialogHeader>

          <div className="space-y-4">
            <div className="space-y-2">
              <Label htmlFor="dispute-reply">{t("common.message")}</Label>
              <Textarea
                id="dispute-reply"
                placeholder={t("seller.respondPlaceholder")}
                value={message}
                onChange={(e) => setMessage(e.target.value)}
                rows={4}
              />
            </div>

            {/* Upload de contra-evidências */}
            <div className="space-y-2">
              <Label>Contra-evidências (opcional — até 5 arquivos)</Label>
              <label className="flex items-center justify-center gap-2 border-2 border-dashed border-border rounded-xl p-3 cursor-pointer hover:bg-muted/50 transition text-sm text-muted-foreground">
                <Paperclip className="h-4 w-4" />
                <span>Adicionar fotos ou vídeos</span>
                <input
                  type="file"
                  accept="image/*,video/*"
                  multiple
                  className="hidden"
                  onChange={handleFileChange}
                  disabled={responseFiles.length >= 5}
                />
              </label>

              {responseFiles.length > 0 && (
                <div className="flex flex-wrap gap-2">
                  {responseFiles.map((file, i) => (
                    <div key={i} className="flex items-center gap-1 bg-muted text-xs px-2 py-1 rounded-full max-w-[160px]">
                      <span className="truncate">{file.name}</span>
                      <button
                        type="button"
                        onClick={() => removeFile(i)}
                        className="shrink-0 ml-0.5 text-muted-foreground hover:text-destructive"
                      >
                        <X className="h-3 w-3" />
                      </button>
                    </div>
                  ))}
                </div>
              )}
            </div>
          </div>

          <DialogFooter className="gap-2">
            <Button type="button" variant="outline" onClick={() => setRespondId(null)} disabled={submitting}>
              {t("common.cancel")}
            </Button>
            <Button type="button" onClick={sendResponse} disabled={submitting} className="vault-card border-0 text-white hover:opacity-90">
              {submitting ? <Loader2 className="mr-2 h-4 w-4 animate-spin" /> : null}
              {t("seller.respond")}
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>
    </div>
  );
}
