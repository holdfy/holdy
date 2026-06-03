/**
 * Scraper para Facebook Marketplace e posts de produto.
 *
 * Estratégia:
 *   1. Navega até a URL (resolve share links automaticamente via redirect)
 *   2. Extrai og: meta tags (sempre presentes, independente de login)
 *   3. Se for Marketplace (/marketplace/item/), tenta extrair preço/local/condição do DOM
 *   4. Faz parse do og:description no formato Marketplace: "R$ X · Cidade, UF · Condição"
 */

import type { Page } from "playwright";

export interface FacebookData {
  title?: string;
  description?: string;
  price?: string;
  images: string[];
  video_url?: string;
  seller_name?: string;
  location?: string;
  condition?: string;
  product_id?: string;
  canonical_url?: string;
}

export async function scrapeFacebook(page: Page, url: string): Promise<FacebookData> {
  await page.goto(url, { waitUntil: "domcontentloaded", timeout: 30000 });

  // Share links precisam de tempo para redirecionar
  await page.waitForTimeout(2500);

  const canonicalUrl = page.url();
  const isMarketplace = canonicalUrl.includes("/marketplace/item/");

  // Extrai og: meta tags — sempre disponível sem login
  const ogData = await page.evaluate(() => {
    const getMeta = (prop: string): string => {
      const el =
        document.querySelector(`meta[property="${prop}"]`) ||
        document.querySelector(`meta[name="${prop}"]`);
      return el?.getAttribute("content")?.trim() || "";
    };

    return {
      title: getMeta("og:title"),
      description: getMeta("og:description"),
      image: getMeta("og:image"),
      video: getMeta("og:video") || getMeta("og:video:url") || getMeta("og:video:secure_url"),
    };
  });

  let price: string | undefined;
  let location: string | undefined;
  let condition: string | undefined;
  let sellerName: string | undefined;
  const images: string[] = [];

  // Parse Marketplace description: "R$ 150 · São Paulo, SP · Usado"
  if (ogData.description) {
    const parts = ogData.description.split("·").map((s) => s.trim());
    for (const part of parts) {
      if (/R\$\s*[\d.,]+/.test(part) && !price) {
        price = part;
      } else if (/\bnovo\b/i.test(part)) {
        condition = "new";
      } else if (/\brecondicionado\b/i.test(part)) {
        condition = "refurbished";
      } else if (/\busado\b/i.test(part)) {
        condition = "used";
      } else if (/^[^·]+,\s*[A-Z]{2}$/.test(part)) {
        location = part; // "São Paulo, SP"
      }
    }
  }

  // Extração adicional via DOM para Marketplace (melhor qualidade de dados)
  if (isMarketplace) {
    try {
      const marketplaceData = await page.evaluate(() => {
        // Preço: aria-label com "R$" ou texto direto
        const allSpans = Array.from(document.querySelectorAll("span, div"));
        const priceEl = allSpans.find((el) => /^R\$\s*[\d.,]+/.test(el.textContent?.trim() || ""));

        // Imagens do produto (fbcdn = Facebook CDN)
        const imgEls = Array.from(document.querySelectorAll("img"))
          .filter((img) => img.src.includes("fbcdn.net") && img.width > 100)
          .map((img) => img.src);

        // Título via h1
        const h1 = document.querySelector("h1")?.textContent?.trim();

        // Vendedor: link de perfil na página
        const sellerLink = document.querySelector("a[href*='/user/'], a[href*='/profile.php']");
        const seller = sellerLink?.textContent?.trim();

        return {
          price: priceEl?.textContent?.trim() || null,
          images: [...new Set(imgEls)].slice(0, 10),
          h1Title: h1 || null,
          seller: seller || null,
        };
      });

      if (marketplaceData.price && !price) price = marketplaceData.price;
      if (marketplaceData.seller) sellerName = marketplaceData.seller;
      images.push(...marketplaceData.images);

      // Título do h1 tem precedência sobre og:title no Marketplace (mais limpo)
      if (marketplaceData.h1Title && marketplaceData.h1Title.length < 200) {
        ogData.title = marketplaceData.h1Title;
      }
    } catch {
      // DOM extraction falhou — og: tags já coletados acima são suficientes
    }
  }

  if (images.length === 0 && ogData.image) {
    images.push(ogData.image);
  }

  // product_id do Marketplace: /marketplace/item/{id}/
  const marketplaceIdMatch = canonicalUrl.match(/\/marketplace\/item\/(\d+)/);
  const productId = marketplaceIdMatch?.[1];

  console.log(
    `[facebook] title=${ogData.title?.substring(0, 60)} price=${price} images=${images.length} marketplace=${isMarketplace}`
  );

  return {
    title: ogData.title || undefined,
    description: ogData.description || undefined,
    price,
    images,
    video_url: ogData.video || undefined,
    seller_name: sellerName,
    location,
    condition,
    product_id: productId,
    canonical_url: canonicalUrl !== url ? canonicalUrl : undefined,
  };
}
