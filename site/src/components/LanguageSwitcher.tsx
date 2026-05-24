import { Globe } from "lucide-react";
import { useTranslation } from "react-i18next";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import { LOCALE_LABELS, SUPPORTED_LOCALES, normalizeLocale } from "@/i18n";
import { cn } from "@/lib/utils";

type Props = {
  variant?: "landing" | "app";
  className?: string;
};

export function LanguageSwitcher({ variant = "app", className }: Props) {
  const { i18n, t } = useTranslation();
  const current = normalizeLocale(i18n.resolvedLanguage ?? i18n.language);
  const currentLabel = LOCALE_LABELS[current] ?? LOCALE_LABELS["pt-BR"];
  const isLanding = variant === "landing";

  const handleChange = (value: string) => {
    void i18n.changeLanguage(normalizeLocale(value));
  };

  return (
    <div className={cn("flex items-center gap-2", className)}>
      {!isLanding && (
        <span className="hidden text-xs text-muted-foreground sm:inline">{t("lang.label")}</span>
      )}
      <Select value={current} onValueChange={handleChange}>
        <SelectTrigger
          className={cn(
            "h-9 w-[140px] gap-2 text-xs",
            isLanding && "border-white/20 bg-white/5 text-white hover:bg-white/10",
          )}
          aria-label={t("lang.label")}
        >
          <Globe className={cn("h-3.5 w-3.5 shrink-0", isLanding ? "text-[#00E5A0]" : "text-muted-foreground")} />
          <SelectValue>{currentLabel}</SelectValue>
        </SelectTrigger>
        <SelectContent align="end">
          {SUPPORTED_LOCALES.map((code) => (
            <SelectItem key={code} value={code}>
              {LOCALE_LABELS[code]}
            </SelectItem>
          ))}
        </SelectContent>
      </Select>
    </div>
  );
}
