import { type ReactNode, useMemo } from "react";
import { useTranslation } from "react-i18next";
import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
  DialogTrigger,
} from "@/components/ui/dialog";

export function NotificationsDialog({ children }: { children: ReactNode }) {
  const { t } = useTranslation();

  const items = useMemo(
    () => [
      { title: t("notifications.n1Title"), body: t("notifications.n1Body"), time: "2h" },
      { title: t("notifications.n2Title"), body: t("notifications.n2Body"), time: "1d" },
      { title: t("notifications.n3Title"), body: t("notifications.n3Body"), time: "Apr 8" },
    ],
    [t],
  );

  return (
    <Dialog>
      <DialogTrigger asChild>{children}</DialogTrigger>
      <DialogContent className="max-w-md">
        <DialogHeader>
          <DialogTitle>{t("notifications.title")}</DialogTitle>
        </DialogHeader>
        <ul className="space-y-3 max-h-[min(60vh,420px)] overflow-y-auto pr-1">
          {items.map((n) => (
            <li key={n.title} className="rounded-lg border border-border p-3 text-sm">
              <p className="font-semibold">{n.title}</p>
              <p className="text-muted-foreground text-xs mt-1 leading-relaxed">{n.body}</p>
              <p className="text-[10px] text-muted-foreground mt-2">{n.time}</p>
            </li>
          ))}
        </ul>
      </DialogContent>
    </Dialog>
  );
}
