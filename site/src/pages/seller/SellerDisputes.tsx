import { useState } from "react";
import { AlertTriangle, MessageSquare, CheckCircle, Clock, Shield } from "lucide-react";
import { useTranslation } from "react-i18next";
import { mockDisputes } from "@/lib/mock-data";
import { getDisputeReason, getDisputeResolution } from "@/lib/mock-i18n";
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

const statusIcons = {
  OPEN: AlertTriangle,
  IN_REVIEW: Clock,
  RESOLVED: CheckCircle,
  CLOSED: CheckCircle,
} as const;

const statusStyles = {
  OPEN: { color: "text-destructive", bg: "bg-destructive/10" },
  IN_REVIEW: { color: "text-amber-600", bg: "bg-amber-100" },
  RESOLVED: { color: "text-secondary", bg: "bg-secondary/10" },
  CLOSED: { color: "text-muted-foreground", bg: "bg-muted" },
} as const;

export default function SellerDisputes() {
  const { t } = useTranslation();
  const [respondId, setRespondId] = useState<string | null>(null);
  const [message, setMessage] = useState("");
  const active = respondId ? mockDisputes.find((d) => d.id === respondId) : undefined;

  const sendResponse = () => {
    if (!message.trim()) {
      toast.error(t("seller.toastRespondEmpty"));
      return;
    }
    toast.success(t("seller.toastRespondSent"));
    setRespondId(null);
    setMessage("");
  };

  return (
    <div className="px-5 pt-6 space-y-5 md:px-0 md:pt-0">
      <h1 className="font-display text-2xl font-bold">{t("seller.disputesTitle")}</h1>

      <div className="grid grid-cols-3 gap-3">
        <div className="bg-destructive/5 border border-destructive/20 rounded-xl p-3 text-center">
          <p className="text-xl font-display font-bold text-destructive">{mockDisputes.filter((d) => d.status === "OPEN").length}</p>
          <p className="text-[10px] text-muted-foreground font-medium uppercase tracking-wider">{t("status.OPEN")}</p>
        </div>
        <div className="bg-amber-50 border border-amber-200 rounded-xl p-3 text-center">
          <p className="text-xl font-display font-bold text-amber-600">{mockDisputes.filter((d) => d.status === "IN_REVIEW").length}</p>
          <p className="text-[10px] text-muted-foreground font-medium uppercase tracking-wider">{t("status.IN_REVIEW")}</p>
        </div>
        <div className="bg-secondary/5 border border-secondary/20 rounded-xl p-3 text-center">
          <p className="text-xl font-display font-bold text-secondary">
            {mockDisputes.filter((d) => d.status === "RESOLVED" || d.status === "CLOSED").length}
          </p>
          <p className="text-[10px] text-muted-foreground font-medium uppercase tracking-wider">{t("status.RESOLVED")}</p>
        </div>
      </div>

      <div className="space-y-3">
        {mockDisputes.map((dispute) => {
          const styles = statusStyles[dispute.status];
          const StatusIcon = statusIcons[dispute.status];
          const resolution = getDisputeResolution(dispute.id, t) ?? dispute.resolution;

          return (
            <div key={dispute.id} className="bg-card rounded-xl p-4 border border-border space-y-3">
              <div className="flex items-start justify-between gap-2">
                <div className="flex items-center gap-3 min-w-0">
                  <div className={`h-10 w-10 rounded-xl ${styles.bg} flex items-center justify-center flex-shrink-0`}>
                    <StatusIcon className={`h-5 w-5 ${styles.color}`} />
                  </div>
                  <div className="min-w-0">
                    <p className="font-semibold text-sm truncate">{dispute.buyer}</p>
                    <p className="text-xs text-muted-foreground">
                      {dispute.orderId} · {dispute.date}
                    </p>
                  </div>
                </div>
                <span className={`text-[10px] font-semibold px-2 py-0.5 rounded-full flex-shrink-0 ${styles.color} ${styles.bg}`}>
                  {t(`status.${dispute.status}`)}
                </span>
              </div>
              <div className="bg-muted/50 rounded-lg p-3">
                <p className="text-xs font-medium text-muted-foreground mb-1">{t("order.disputeTitle")}</p>
                <p className="text-sm">{getDisputeReason(dispute.id, t)}</p>
              </div>
              {resolution && (
                <div className="bg-secondary/5 rounded-lg p-3">
                  <p className="text-xs font-medium text-secondary mb-1">{t("status.RESOLVED")}</p>
                  <p className="text-sm">{resolution}</p>
                </div>
              )}
              {(dispute.status === "OPEN" || dispute.status === "IN_REVIEW") && (
                <div className="flex gap-2">
                  <button
                    type="button"
                    className="flex-1 flex items-center justify-center gap-2 px-4 py-2.5 rounded-xl bg-primary text-primary-foreground text-sm font-semibold hover:opacity-90 transition"
                    onClick={() => {
                      setMessage("");
                      setRespondId(dispute.id);
                    }}
                  >
                    <MessageSquare className="h-4 w-4" />
                    {t("seller.respond")}
                  </button>
                </div>
              )}
            </div>
          );
        })}
      </div>

      <div className="flex items-center justify-center gap-2 py-4 text-xs text-muted-foreground">
        <Shield className="h-3.5 w-3.5" />
        <span className="tracking-wider uppercase font-medium">{t("common.protectedPayment")}</span>
      </div>

      <Dialog open={!!respondId} onOpenChange={(o) => !o && setRespondId(null)}>
        <DialogContent>
          <DialogHeader>
            <DialogTitle>{t("seller.respondTitle")}</DialogTitle>
            <DialogDescription>{t("seller.respondDesc")}</DialogDescription>
          </DialogHeader>
          <div className="space-y-2">
            <Label htmlFor="dispute-reply">{t("common.message")}</Label>
            <Textarea
              id="dispute-reply"
              placeholder={t("seller.respondPlaceholder")}
              value={message}
              onChange={(e) => setMessage(e.target.value)}
            />
          </div>
          <DialogFooter className="gap-2">
            <Button type="button" variant="outline" onClick={() => setRespondId(null)}>
              {t("common.cancel")}
            </Button>
            <Button type="button" onClick={sendResponse}>
              {t("seller.respond")}
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>
    </div>
  );
}
