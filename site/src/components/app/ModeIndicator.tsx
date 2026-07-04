import { Store, ShoppingBag } from "lucide-react";
import { useTranslation } from "react-i18next";
import { Link } from "react-router-dom";
import { cn } from "@/lib/utils";

interface ModeIndicatorProps {
  mode: "buyer" | "seller";
}

function segmentClasses(active: boolean, variant: "buyer" | "seller") {
  if (!active) return "text-muted-foreground hover:text-foreground";
  return variant === "seller" ? "bg-primary/10 text-primary" : "bg-secondary/10 text-secondary";
}

/** Toggle compacto no header desktop — clique troca de modo, sempre visível (mesmo com a sidebar colapsada). */
export function ModeSwitcher({ mode }: ModeIndicatorProps) {
  const { t } = useTranslation();

  return (
    <div className="inline-flex items-center gap-0.5 rounded-full border border-border p-0.5">
      <Link
        to="/buyer"
        className={cn(
          "inline-flex items-center gap-1.5 rounded-full px-2.5 py-1 text-xs font-semibold transition-colors",
          segmentClasses(mode === "buyer", "buyer"),
        )}
      >
        <ShoppingBag className="h-3.5 w-3.5" />
        {t("common.buyer")}
      </Link>
      <Link
        to="/seller"
        className={cn(
          "inline-flex items-center gap-1.5 rounded-full px-2.5 py-1 text-xs font-semibold transition-colors",
          segmentClasses(mode === "seller", "seller"),
        )}
      >
        <Store className="h-3.5 w-3.5" />
        {t("common.seller")}
      </Link>
    </div>
  );
}

/** Mesmo toggle em formato de faixa, fixa no topo das telas mobile. */
export function ModeSwitcherStrip({ mode }: ModeIndicatorProps) {
  const { t } = useTranslation();

  return (
    <div className="sticky top-0 z-40 flex items-stretch border-b border-border bg-background">
      <Link
        to="/buyer"
        className={cn(
          "flex flex-1 items-center justify-center gap-1.5 py-2 text-xs font-semibold transition-colors",
          segmentClasses(mode === "buyer", "buyer"),
        )}
      >
        <ShoppingBag className="h-3.5 w-3.5" />
        {t("common.buyer")}
      </Link>
      <Link
        to="/seller"
        className={cn(
          "flex flex-1 items-center justify-center gap-1.5 py-2 text-xs font-semibold transition-colors",
          segmentClasses(mode === "seller", "seller"),
        )}
      >
        <Store className="h-3.5 w-3.5" />
        {t("common.seller")}
      </Link>
    </div>
  );
}
