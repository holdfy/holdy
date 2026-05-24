import i18n from "@/i18n";
import { LOCALE_NUMBER_FORMAT, type SupportedLocale } from "@/i18n";

export function formatCurrency(amount: number, locale?: string): string {
  const lng = (locale ?? i18n.language) as SupportedLocale;
  const fmt = LOCALE_NUMBER_FORMAT[lng] ?? "pt-BR";
  return new Intl.NumberFormat(fmt, {
    style: "currency",
    currency: "BRL",
    minimumFractionDigits: 2,
  }).format(amount);
}
