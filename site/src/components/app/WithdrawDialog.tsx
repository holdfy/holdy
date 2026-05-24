import { type ReactNode, useState } from "react";
import { useTranslation } from "react-i18next";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
  DialogTrigger,
} from "@/components/ui/dialog";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { toast } from "sonner";

export function WithdrawDialog({ children }: { children: ReactNode }) {
  const { t } = useTranslation();
  const [open, setOpen] = useState(false);
  const [amount, setAmount] = useState("");
  const [pixKey, setPixKey] = useState("");

  const submit = () => {
    const n = parseFloat(amount.replace(",", "."));
    if (!amount.trim() || Number.isNaN(n) || n <= 0 || !pixKey.trim()) {
      toast.error(t("withdraw.toastInvalid"));
      return;
    }
    toast.success(t("withdraw.toastSuccess"));
    setOpen(false);
    setAmount("");
    setPixKey("");
  };

  return (
    <Dialog open={open} onOpenChange={setOpen}>
      <DialogTrigger asChild>{children}</DialogTrigger>
      <DialogContent>
        <DialogHeader>
          <DialogTitle>{t("withdraw.title")}</DialogTitle>
          <DialogDescription>{t("wallet.depositDesc")}</DialogDescription>
        </DialogHeader>
        <div className="space-y-3">
          <div className="space-y-2">
            <Label htmlFor="withdraw-amt">{t("withdraw.amount")}</Label>
            <Input
              id="withdraw-amt"
              inputMode="decimal"
              placeholder="0.00"
              value={amount}
              onChange={(e) => setAmount(e.target.value)}
            />
          </div>
          <div className="space-y-2">
            <Label htmlFor="withdraw-pix">{t("withdraw.pixKey")}</Label>
            <Input
              id="withdraw-pix"
              placeholder="email@example.com"
              value={pixKey}
              onChange={(e) => setPixKey(e.target.value)}
            />
          </div>
        </div>
        <DialogFooter className="gap-2 sm:gap-0">
          <Button type="button" variant="outline" onClick={() => setOpen(false)}>
            {t("common.cancel")}
          </Button>
          <Button type="button" className="vault-card border-0 text-white hover:opacity-90" onClick={submit}>
            {t("withdraw.confirm")}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}
