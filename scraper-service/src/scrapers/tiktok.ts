import type { Page } from "playwright";

export interface TikTokProduct {
  title: string;
  description: string | null;
  price: string | null;
  price_original: string | null;
  currency: string;
  images: string[];
  video_url: string | null;
  seller_name: string | null;
  seller_rating: string | null;
  product_id: string | null;
  source_url: string;
  platform: "tiktok";
}

export async function scrapeTikTok(page: Page, url: string): Promise<TikTokProduct | null> {
  // Capturar respostas de API via response event (não via route que causa bug com redirects relativos)
  const apiData: Record<string, unknown> = {};

  page.on("response", async (response) => {
    const respUrl = response.url();
    if (
      (respUrl.includes("/api/commerce") ||
       respUrl.includes("/api/shop") ||
       respUrl.includes("item/detail") ||
       respUrl.includes("product/detail")) &&
      response.status() === 200
    ) {
      try {
        const body = await response.text();
        if (body.startsWith("{") && (body.includes("price") || body.includes("image"))) {
          apiData[respUrl] = JSON.parse(body);
        }
      } catch {
        // Ignorar erros de parse
      }
    }
  });

  try {
    await page.goto(url, {
      waitUntil: "domcontentloaded",
      timeout: 25000,
    });
  } catch (e) {
    // TikTok pode gerar erros de navegação — continuar se a página carregou parcialmente
    console.warn("[tiktok] goto warning:", (e as Error).message?.substring(0, 100));
  }

  // Aguardar conteúdo JS renderizar
  await page.waitForTimeout(4000);

  // ── Extrair dados do DOM ──────────────────────────────────────────────────

  const result = await page.evaluate((): Partial<TikTokProduct> => {
    // Título
    const titleEl =
      document.querySelector("[data-testid='product-title']") ||
      document.querySelector(".product-title") ||
      document.querySelector("h1") ||
      document.querySelector("[class*='title']");
    const title = titleEl?.textContent?.trim() || "";

    // Preço — TikTok Shop usa vários seletores dependendo da versão
    const priceSelectors = [
      "[data-testid='product-price']",
      "[class*='price'][class*='sale']",
      "[class*='current-price']",
      "[class*='salePrice']",
      "[class*='Price'] span",
      ".price-sale",
      "[data-e2e='price']",
    ];
    let price: string | null = null;
    let price_original: string | null = null;
    for (const sel of priceSelectors) {
      const el = document.querySelector(sel);
      if (el?.textContent?.trim()) {
        price = el.textContent.trim();
        break;
      }
    }

    // Preço original (sem desconto)
    const origSelectors = [
      "[class*='original-price']",
      "[class*='originalPrice']",
      "[class*='price-through']",
      "del",
      "s",
    ];
    for (const sel of origSelectors) {
      const el = document.querySelector(sel);
      if (el?.textContent?.trim()) {
        price_original = el.textContent.trim();
        break;
      }
    }

    // Descrição
    const descEl =
      document.querySelector("[data-testid='product-description']") ||
      document.querySelector("[class*='description']") ||
      document.querySelector("[class*='detail'] p");
    const description = descEl?.textContent?.trim() || null;

    // Imagens — pegar todas as thumbnails do produto
    const images: string[] = [];
    const imgSelectors = [
      "[data-testid='product-image'] img",
      "[class*='thumbnail'] img",
      "[class*='gallery'] img",
      "[class*='swiper'] img",
      "[class*='carousel'] img",
      "[class*='product-img'] img",
    ];
    for (const sel of imgSelectors) {
      document.querySelectorAll(sel).forEach((img) => {
        const src = (img as HTMLImageElement).src || (img as HTMLImageElement).dataset.src;
        if (src && src.startsWith("http") && !images.includes(src)) {
          images.push(src);
        }
      });
    }

    // Fallback: todas as imagens grandes da página
    if (images.length === 0) {
      document.querySelectorAll("img").forEach((img) => {
        const src = img.src || img.dataset.src || "";
        if (
          src.startsWith("http") &&
          (src.includes("tiktok") || src.includes("ibyteimg")) &&
          !images.includes(src)
        ) {
          images.push(src);
        }
      });
    }

    // Vendedor
    const sellerEl =
      document.querySelector("[data-testid='shop-name']") ||
      document.querySelector("[class*='shop-name']") ||
      document.querySelector("[class*='seller-name']");
    const seller_name = sellerEl?.textContent?.trim() || null;

    // Rating
    const ratingEl =
      document.querySelector("[data-testid='shop-rating']") ||
      document.querySelector("[class*='rating']");
    const seller_rating = ratingEl?.textContent?.trim() || null;

    return { title, price, price_original, description, images, seller_name, seller_rating };
  });

  // ── Tentar dados da API interceptada ─────────────────────────────────────
  let apiPrice: string | null = null;
  let apiImages: string[] = [];
  for (const [, data] of Object.entries(apiData)) {
    const d = data as Record<string, unknown>;
    // Procurar preço no JSON interceptado
    const priceStr = findDeep(d, ["sale_price", "price", "currentPrice", "salePrice"]);
    if (priceStr && !apiPrice) apiPrice = String(priceStr);

    // Procurar imagens
    const imgs = findDeepArray(d, ["images", "img_urls", "imageList"]);
    if (imgs.length > 0) apiImages = [...apiImages, ...imgs];
  }

  const title = result.title || "";
  if (!title) return null;

  const allImages = dedupeAndClean([...apiImages, ...(result.images || [])]);

  return {
    title,
    description: result.description || null,
    price: apiPrice || result.price || null,
    price_original: result.price_original || null,
    currency: "BRL",
    images: allImages,
    video_url: null,
    seller_name: result.seller_name || null,
    seller_rating: result.seller_rating || null,
    product_id: extractProductId(url),
    source_url: url,
    platform: "tiktok",
  };
}

// ── Helpers ───────────────────────────────────────────────────────────────

function extractProductId(url: string): string | null {
  const m = url.match(/product\/(\d+)/);
  return m ? m[1] : null;
}

function findDeep(obj: unknown, keys: string[]): unknown {
  if (!obj || typeof obj !== "object") return null;
  for (const key of keys) {
    if ((obj as Record<string, unknown>)[key] !== undefined) {
      return (obj as Record<string, unknown>)[key];
    }
  }
  for (const val of Object.values(obj as Record<string, unknown>)) {
    const found = findDeep(val, keys);
    if (found !== null && found !== undefined) return found;
  }
  return null;
}

function findDeepArray(obj: unknown, keys: string[]): string[] {
  if (!obj || typeof obj !== "object") return [];
  const result: string[] = [];
  if (Array.isArray(obj)) {
    for (const item of obj) result.push(...findDeepArray(item, keys));
    return result;
  }
  for (const [k, v] of Object.entries(obj as Record<string, unknown>)) {
    if (keys.includes(k) && Array.isArray(v)) {
      for (const item of v) {
        if (typeof item === "string" && item.startsWith("http")) result.push(item);
        else if (typeof item === "object" && item !== null) {
          const url = (item as Record<string, string>).url || (item as Record<string, string>).src;
          if (url) result.push(url);
        }
      }
    } else {
      result.push(...findDeepArray(v, keys));
    }
  }
  return result;
}

function dedupeAndClean(urls: string[]): string[] {
  return [...new Set(urls)].filter(
    (u) => u.startsWith("http") && (u.includes("tiktok") || u.includes("ibyteimg") || u.includes("bytedance"))
  );
}
