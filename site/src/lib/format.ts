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
