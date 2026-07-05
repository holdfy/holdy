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
