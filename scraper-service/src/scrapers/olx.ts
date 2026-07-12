/**
 * Scraper para OLX Brasil.
 *
 * OLX bloqueia scraping HTTP simples via Cloudflare (challenge JS) — só um
 * browser real (Playwright + stealth) passa. A página renderizada expõe um
 * bloco `<script type="application/ld+json">` schema.org Product completo
 * (nome, descrição, preço, imagens), então a extração é quase toda via JSON-LD;
 * localização/vendedor não estão no JSON-LD e são pegos via texto do DOM (best-effort).
 */

import type { Page } from "playwright";

export interface OlxData {
  title?: string;
  description?: string;
  price?: string;
  images: string[];
  seller_name?: string;
  location?: string;
  product_id?: string;
  canonical_url?: string;
}

interface OlxJsonLd {
  name?: string;
  description?: string;
  identifier?: number | string;
  url?: string;
  image?: Array<{ contentUrl?: string }>;
  offers?: { price?: string; priceCurrency?: string };
}

export async function scrapeOlx(page: Page, url: string): Promise<OlxData> {
  await page.goto(url, { waitUntil: "domcontentloaded", timeout: 30000 });

  // O challenge do Cloudflare resolve em background — tempo de resolução varia
  // (visto 5-9s em produção). Faz polling pelo JSON-LD em vez de timeout fixo.
  try {
    await page.waitForFunction(
      () => !!document.querySelector('script[type="application/ld+json"]'),
      { timeout: 15000 }
    );
  } catch {
    // Deu timeout esperando o JSON-LD — segue com o que tiver (extração abaixo retorna vazio).
  }
  await page.waitForTimeout(1000);

  const canonicalUrl = page.url();

  const jsonLd = await page.evaluate(() => {
    const scripts = Array.from(
      document.querySelectorAll('script[type="application/ld+json"]')
    );
    for (const s of scripts) {
      try {
        const parsed = JSON.parse(s.textContent || "");
        if (parsed["@type"] === "Product") return parsed;
      } catch {
        // ignora blocos de JSON-LD que não sejam Product válido
      }
    }
    return null;
  }) as OlxJsonLd | null;

  const images = (jsonLd?.image ?? [])
    .map((i) => i.contentUrl)
    .filter((u): u is string => !!u);

  // Localização + tempo de conta aparecem juntos no card do vendedor — separa por regex.
  const locationRaw = await page.evaluate(() => {
    const candidates = Array.from(document.querySelectorAll("span, p, div"))
      .map((e) => e.textContent?.trim() || "")
      .filter((t) => /,\s*[A-Za-zÀ-ÿ ]+\s*-\s*[A-Z]{2}$/.test(t) && t.length < 120);
    return candidates[0] || null;
  });
  const location = locationRaw?.match(/([A-Za-zÀ-ÿ0-9 ]+,\s*[A-Za-zÀ-ÿ ]+\s*-\s*[A-Z]{2})$/)?.[1];

  console.log(
    `[olx] title=${jsonLd?.name?.substring(0, 60)} price=${jsonLd?.offers?.price} images=${images.length}`
  );

  return {
    title: jsonLd?.name,
    description: jsonLd?.description,
    price: jsonLd?.offers?.price,
    images,
    location,
    product_id: jsonLd?.identifier != null ? String(jsonLd.identifier) : undefined,
    canonical_url: canonicalUrl !== url ? canonicalUrl : undefined,
  };
}
