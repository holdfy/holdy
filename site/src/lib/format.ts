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

/**
 * Máscara de valor monetário BR (estilo 1.000.000,00) — digita-se em centavos,
 * formata progressivamente com ponto de milhar e vírgula decimal.
 */
export function maskCurrencyBR(rawValue: string): string {
  const digits = rawValue.replace(/\D/g, "").replace(/^0+(?=\d)/, "");
  if (!digits) return "";
  const cents = digits.padStart(3, "0");
  const intPart = cents.slice(0, -2);
  const centsPart = cents.slice(-2);
  const withThousands = intPart.replace(/\B(?=(\d{3})+(?!\d))/g, ".");
  return `${withThousands},${centsPart}`;
}

/** Converte valor mascarado BR ("1.000.000,00") para string decimal ("1000000.00") pro backend. */
export function unmaskCurrencyBR(masked: string): string {
  const digits = masked.replace(/\D/g, "");
  if (!digits) return "";
  const cents = digits.padStart(3, "0");
  const intPart = cents.slice(0, -2).replace(/^0+(?=\d)/, "") || "0";
  const centsPart = cents.slice(-2);
  return `${intPart}.${centsPart}`;
}

/** Converte um decimal simples ("150.5", 150) pro formato mascarado BR ("150,50") — usado ao pré-preencher. */
export function decimalToMaskedBR(decimal: string | number): string {
  const n = typeof decimal === "number" ? decimal : parseFloat(decimal.replace(",", "."));
  if (!Number.isFinite(n)) return "";
  return maskCurrencyBR(Math.round(n * 100).toString());
}

/** Aplica máscara CPF (000.000.000-00) ou CNPJ (00.000.000/0001-00) progressivamente. */
export function maskCpfCnpj(value: string): string {
  const d = value.replace(/\D/g, "").slice(0, 14);
  if (d.length <= 11) {
    return d
      .replace(/(\d{3})(\d)/, "$1.$2")
      .replace(/(\d{3})(\d)/, "$1.$2")
      .replace(/(\d{3})(\d{1,2})$/, "$1-$2");
  }
  return d
    .replace(/(\d{2})(\d)/, "$1.$2")
    .replace(/(\d{3})(\d)/, "$1.$2")
    .replace(/(\d{3})(\d)/, "$1/$2")
    .replace(/(\d{4})(\d{1,2})$/, "$1-$2");
}

/** Retorna os dígitos puros de um CPF/CNPJ formatado. */
export function stripDoc(value: string): string {
  return value.replace(/\D/g, "");
}

/** Aplica máscara de telefone brasileiro: (XX) XXXX-XXXX ou (XX) XXXXX-XXXX */
export function maskPhone(value: string): string {
  const d = value.replace(/\D/g, "").slice(0, 11);
  if (d.length <= 10) {
    return d
      .replace(/^(\d{2})(\d)/, "($1) $2")
      .replace(/(\d{4})(\d{1,4})$/, "$1-$2");
  }
  return d
    .replace(/^(\d{2})(\d)/, "($1) $2")
    .replace(/(\d{5})(\d{1,4})$/, "$1-$2");
}

// Algoritmo oficial Receita Federal — pesos explícitos conforme documentação
export function validateCpf(digits: string): boolean {
  if (digits.length !== 11) return false;
  if (/^(\d)\1{10}$/.test(digits)) return false;
  const check = (len: number) => {
    const weights = Array.from({ length: len }, (_, i) => len + 1 - i);
    const sum = weights.reduce((acc, w, i) => acc + +digits[i] * w, 0);
    const r = 11 - (sum % 11);
    return r >= 10 ? 0 : r;
  };
  return check(9) === +digits[9] && check(10) === +digits[10];
}

export function validateCnpj(digits: string): boolean {
  if (digits.length !== 14) return false;
  if (/^(\d)\1{13}$/.test(digits)) return false;
  const check = (len: number) => {
    const weights = len === 12
      ? [5, 4, 3, 2, 9, 8, 7, 6, 5, 4, 3, 2]
      : [6, 5, 4, 3, 2, 9, 8, 7, 6, 5, 4, 3, 2];
    const sum = weights.reduce((acc, w, i) => acc + +digits[i] * w, 0);
    const r = 11 - (sum % 11);
    return r >= 10 ? 0 : r;
  };
  return check(12) === +digits[12] && check(13) === +digits[13];
}

export function validateCpfOrCnpj(value: string): boolean {
  const d = stripDoc(value);
  if (d.length === 11) return validateCpf(d);
  if (d.length === 14) return validateCnpj(d);
  return false;
}

export type PixKeyType = "cpf" | "cnpj" | "email" | "phone" | "random" | null;

/** Detecta o tipo de chave PIX pelo formato — usuário nunca precisa informar. */
export function detectPixKeyType(rawKey: string): PixKeyType {
  const key = rawKey.trim();
  if (!key) return null;
  if (/^[^\s@]+@[^\s@]+\.[^\s@]+$/.test(key)) return "email";
  if (/^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$/i.test(key)) return "random";
  if (key.startsWith("+")) return "phone";
  const digits = stripDoc(key);
  if (digits.length !== key.length) return null; // mistura letras/símbolos fora dos formatos acima
  if (digits.length === 11) return validateCpf(digits) ? "cpf" : "phone";
  if (digits.length === 14) return validateCnpj(digits) ? "cnpj" : null;
  if (digits.length === 10) return "phone";
  return null;
}
