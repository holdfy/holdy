import { useMemo, useState } from "react";
import { Shield, ArrowLeft, CheckCircle2, HelpCircle, CheckCheck } from "lucide-react";
import { Link, useLocation, useNavigate } from "react-router-dom";
import { useTranslation } from "react-i18next";
import { Button } from "@/components/ui/button";
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

export default function AppOrderDetail() {
  const { t } = useTranslation();
  const location = useLocation();
  const navigate = useNavigate();
  const isSeller = location.pathname.startsWith("/seller");
  const ordersPath = isSeller ? "/seller/orders" : "/buyer/orders";
  const profilePath = isSeller ? "/seller/profile" : "/buyer/profile";

  const [disputeOpen, setDisputeOpen] = useState(false);
  const [disputeNote, setDisputeNote] = useState("");
  const [confirmOpen, setConfirmOpen] = useState(false);

  const steps = useMemo(
    () => [
      { label: t("order.stepPaid"), desc: t("order.stepPaidDesc"), status: "done" as const },
      { label: t("order.stepCustody"), desc: t("order.stepCustodyDesc"), status: "active" as const },
      { label: t("order.stepShipped"), desc: t("order.stepShippedDesc"), status: "pending" as const },
      { label: t("order.stepDelivered"), desc: t("order.stepDeliveredDesc"), status: "pending" as const },
      { label: t("order.stepReleased"), desc: t("order.stepReleasedDesc"), status: "pending" as const },
    ],
    [t],
  );

  const submitDispute = () => {
    if (!disputeNote.trim()) {
      toast.error(t("order.toastDisputeEmpty"));
      return;
    }
    toast.success(t("order.toastDisputeSent"));
    setDisputeOpen(false);
    setDisputeNote("");
  };

  const confirmDelivery = () => {
    setConfirmOpen(false);
    toast.success(t("order.toastDeliveryConfirmed"));
    navigate(isSeller ? "/seller/orders" : "/buyer/transaction-complete");
  };

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

      <div className="bg-card rounded-2xl p-5 border border-border">
        <div className="flex items-center justify-between mb-1">
          <p className="text-xs font-semibold tracking-wider text-muted-foreground uppercase">ORDER #8921-X</p>
          <span className="text-xs font-semibold text-secondary bg-secondary/10 px-3 py-1 rounded-full">
            {t("status.IN_CUSTODY_BADGE")}
          </span>
        </div>
        <h2 className="font-display text-2xl font-bold">MacBook Pro M2</h2>
      </div>

      <div className="vault-card rounded-2xl p-5">
        <p className="text-xs font-semibold tracking-[0.15em] text-white/50 uppercase mb-1">{t("order.protectedPayment")}</p>
        <div className="flex items-baseline gap-2">
          <span className="text-3xl font-display font-bold text-secondary">{formatCurrency(8500)}</span>
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

      <div className="bg-card rounded-2xl p-5 border border-border">
        <h3 className="font-display font-bold text-lg mb-5">{t("order.deliveryProgress")}</h3>
        <div className="space-y-0">
          {steps.map((step, i) => (
            <div key={step.label} className="flex gap-4">
              <div className="flex flex-col items-center">
                <div
                  className={`h-8 w-8 rounded-full flex items-center justify-center flex-shrink-0 ${
                    step.status === "done"
                      ? "bg-secondary text-white"
                      : step.status === "active"
                        ? "bg-secondary/20 border-2 border-secondary"
                        : "bg-muted"
                  }`}
                >
                  {step.status === "done" ? (
                    <CheckCircle2 className="h-4 w-4" />
                  ) : step.status === "active" ? (
                    <div className="h-2.5 w-2.5 rounded-full bg-secondary" />
                  ) : (
                    <div className="h-2.5 w-2.5 rounded-full bg-muted-foreground/30" />
                  )}
                </div>
                {i < steps.length - 1 && (
                  <div className={`w-0.5 h-10 ${step.status === "done" ? "bg-secondary" : "bg-border"}`} />
                )}
              </div>
              <div className="pb-6">
                <p
                  className={`font-semibold text-sm ${
                    step.status === "active"
                      ? "text-secondary"
                      : step.status === "pending"
                        ? "text-muted-foreground"
                        : ""
                  }`}
                >
                  {step.label}
                </p>
                <p className="text-xs text-muted-foreground mt-0.5">{step.desc}</p>
              </div>
            </div>
          ))}
        </div>
      </div>

      <div className="flex gap-3">
        <Button type="button" variant="outline" className="flex-1 h-14 rounded-xl text-sm font-semibold" onClick={() => setDisputeOpen(true)}>
          <HelpCircle className="mr-2 h-4 w-4" />
          {t("order.openDispute")}
        </Button>
        <Button
          type="button"
          className="flex-1 h-14 rounded-xl vault-card border-0 text-white text-sm font-semibold hover:opacity-90"
          onClick={() => setConfirmOpen(true)}
        >
          <CheckCheck className="mr-2 h-4 w-4" />
          {t("order.confirmDelivery")}
        </Button>
      </div>

      <div className="h-4" />

      <Dialog open={disputeOpen} onOpenChange={setDisputeOpen}>
        <DialogContent>
          <DialogHeader>
            <DialogTitle>{t("order.disputeTitle")}</DialogTitle>
            <DialogDescription>{t("order.disputeDesc")}</DialogDescription>
          </DialogHeader>
          <div className="space-y-2">
            <Label htmlFor="dispute-note">{t("common.message")}</Label>
            <Textarea
              id="dispute-note"
              placeholder={t("order.disputePlaceholder")}
              value={disputeNote}
              onChange={(e) => setDisputeNote(e.target.value)}
            />
          </div>
          <DialogFooter className="gap-2">
            <Button type="button" variant="outline" onClick={() => setDisputeOpen(false)}>
              {t("common.cancel")}
            </Button>
            <Button type="button" className="vault-card border-0 text-white hover:opacity-90" onClick={submitDispute}>
              {t("seller.respond")}
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>

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
    </div>
  );
}
