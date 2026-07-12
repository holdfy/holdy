import { clsx, type ClassValue } from "clsx";
import { twMerge } from "tailwind-merge";

export function cn(...inputs: ClassValue[]) {
  return twMerge(clsx(inputs));
}

/**
 * Base URL to use when building links meant to be shared with third parties
 * (e.g. a payment link sent to a buyer). Prefers VITE_SITE_PUBLIC_URL (set at
 * build/dev time) since window.location.origin reflects whatever host the
 * seller happened to open the site with — which breaks if that's
 * 127.0.0.1/localhost and the link is opened from another device.
 */
export function getPublicSiteUrl(): string {
  const configured = import.meta.env.VITE_SITE_PUBLIC_URL as string | undefined;
  if (configured) return configured.replace(/\/$/, "");

  const { origin, hostname } = window.location;
  if (hostname === "127.0.0.1" || hostname === "localhost") {
    console.warn(
      "[getPublicSiteUrl] Site aberto via localhost/127.0.0.1 e VITE_SITE_PUBLIC_URL não definida — " +
        "o link gerado não será acessível de outros dispositivos. Defina VITE_SITE_PUBLIC_URL ou acesse pelo IP da rede.",
    );
  }
  return origin;
}

/**
 * Copia texto pra área de transferência. `navigator.clipboard` só existe em
 * contexto seguro (HTTPS ou localhost) — acessando o site pelo IP da rede via
 * HTTP (comum em dev), `navigator.clipboard` é `undefined` e `.writeText`
 * quebra silenciosamente (o botão "parece" não fazer nada). Cai pro truque
 * clássico de `textarea` + `execCommand("copy")` nesse caso.
 */
export async function copyToClipboard(text: string): Promise<boolean> {
  if (navigator.clipboard && window.isSecureContext) {
    try {
      await navigator.clipboard.writeText(text);
      return true;
    } catch {
      // cai pro fallback abaixo
    }
  }
  try {
    const textarea = document.createElement("textarea");
    textarea.value = text;
    textarea.style.position = "fixed";
    textarea.style.opacity = "0";
    document.body.appendChild(textarea);
    textarea.focus();
    textarea.select();
    const ok = document.execCommand("copy");
    document.body.removeChild(textarea);
    return ok;
  } catch {
    return false;
  }
}
