/**
 * Scraper Service — headless browser para TikTok Shop, Instagram e outros.
 * Porta: SCRAPER_PORT (default 4000)
 * Auth: x-api-key = SCRAPER_API_KEY
 *
 * POST /scrape   { url: string }
 * GET  /health
 */

import { chromium as chromiumBase, type Browser } from "playwright";
import { chromium } from "playwright-extra";
import StealthPlugin from "puppeteer-extra-plugin-stealth";

chromium.use(StealthPlugin());
import { scrapeTikTok } from "./scrapers/tiktok";
import { scrapeFacebook } from "./scrapers/facebook";
import { scrapeOlx } from "./scrapers/olx";

const PORT = parseInt(process.env.SCRAPER_PORT || "4000");
const API_KEY = process.env.SCRAPER_API_KEY || "";

let browser: Browser | null = null;

async function getBrowser(): Promise<Browser> {
  if (!browser || !browser.isConnected()) {
    // Usar Chrome do sistema se disponível, senão o Chromium do Playwright
    const executablePath = [
      "/usr/bin/google-chrome-stable",
      "/usr/bin/google-chrome",
      "/usr/bin/chromium-browser",
      "/usr/bin/chromium",
    ].find((p) => {
      try { return Bun.spawnSync(["test", "-f", p]).exitCode === 0; } catch { return false; }
    });

    browser = await (chromium as typeof chromiumBase).launch({
      headless: true,
      executablePath,
      args: [
        "--no-sandbox",
        "--disable-setuid-sandbox",
        "--disable-dev-shm-usage",
        "--disable-blink-features=AutomationControlled",
        "--disable-web-security",
        "--disable-features=IsolateOrigins,site-per-process",
      ],
    });
  }
  return browser;
}

function isTikTok(url: string): boolean {
  return url.includes("tiktok.com");
}

function isFacebook(url: string): boolean {
  return url.includes("facebook.com") || url.includes("fb.com");
}

function isOlx(url: string): boolean {
  return url.includes("olx.com");
}

async function scrape(url: string): Promise<unknown> {
  const b = await getBrowser();
  // OLX: challenge do Cloudflare foi validado com UA/viewport desktop — mobile não foi testado.
  const context = await b.newContext(
    isOlx(url)
      ? {
          userAgent:
            "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/124.0.0.0 Safari/537.36",
          viewport: { width: 1366, height: 900 },
          locale: "pt-BR",
          timezoneId: "America/Sao_Paulo",
          extraHTTPHeaders: {
            "Accept-Language": "pt-BR,pt;q=0.9",
          },
        }
      : {
          userAgent:
            "Mozilla/5.0 (iPhone; CPU iPhone OS 16_0 like Mac OS X) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/16.0 Mobile/15E148 Safari/604.1",
          viewport: { width: 390, height: 844 },
          locale: "pt-BR",
          timezoneId: "America/Sao_Paulo",
          extraHTTPHeaders: {
            "Accept-Language": "pt-BR,pt;q=0.9",
          },
        }
  );

  try {
    const page = await context.newPage();

    // Stealth mode — mascarar sinais de automação que o TikTok detecta
    await page.addInitScript(() => {
      // Remove webdriver flag
      Object.defineProperty(navigator, "webdriver", { get: () => undefined });

      // Simular plugins reais
      Object.defineProperty(navigator, "plugins", {
        get: () => {
          const arr = [
            { name: "Chrome PDF Plugin", filename: "internal-pdf-viewer", description: "Portable Document Format" },
            { name: "Chrome PDF Viewer", filename: "mhjfbmdgcfjbbpaeojofohoefgiehjai", description: "" },
            { name: "Native Client", filename: "internal-nacl-plugin", description: "" },
          ];
          return Object.assign(arr, { item: (i: number) => arr[i], namedItem: (n: string) => arr.find(p => p.name === n) || null, refresh: () => {} });
        },
      });

      // Simular languages
      Object.defineProperty(navigator, "languages", { get: () => ["pt-BR", "pt", "en-US", "en"] });

      // Chrome runtime
      (window as Record<string, unknown>).chrome = {
        runtime: {},
        loadTimes: () => ({}),
        csi: () => ({}),
      };

      // Permissions
      const originalQuery = window.navigator.permissions?.query?.bind(window.navigator.permissions);
      if (originalQuery) {
        (window.navigator.permissions as Record<string, unknown>).query = (parameters: unknown) => {
          const params = parameters as { name: string };
          return params.name === "notifications"
            ? Promise.resolve({ state: "denied", onchange: null })
            : originalQuery(parameters as PermissionDescriptor);
        };
      }
    });

    if (isTikTok(url)) {
      // Para vt.tiktok.com: extrair o product_id via curl -I (só o primeiro redirect, sem seguir login)
      let targetUrl = url;
      if (url.includes("vt.tiktok.com")) {
        const proc = Bun.spawnSync([
          "curl", "-sI",
          "-A", "Mozilla/5.0 (iPhone; CPU iPhone OS 16_0 like Mac OS X) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/16.0 Mobile/15E148 Safari/604.1",
          "--max-redirs", "0",
          url,
        ]);
        const headers = proc.stdout.toString();
        const locationMatch = headers.match(/[Ll]ocation:\s*(https?:\/\/[^\r\n]+)/);
        if (locationMatch) {
          const location = locationMatch[1].trim();
          if (location.includes("tiktok.com") && !location.includes("/login")) {
            targetUrl = location;
          } else if (location.includes("/view/product/")) {
            targetUrl = location.startsWith("http") ? location : `https://www.tiktok.com${location}`;
          }
        }
      }
      console.log(`[scraper] TikTok target URL: ${targetUrl.substring(0, 100)}`);
      return await scrapeTikTok(page, targetUrl);
    }

    if (isFacebook(url)) {
      console.log(`[scraper] Facebook URL: ${url.substring(0, 100)}`);
      return await scrapeFacebook(page, url);
    }

    if (isOlx(url)) {
      console.log(`[scraper] OLX URL: ${url.substring(0, 100)}`);
      return await scrapeOlx(page, url);
    }

    return { error: "platform not supported", url };
  } finally {
    await context.close();
  }
}

const server = Bun.serve({
  port: PORT,
  async fetch(req) {
    const url = new URL(req.url);

    // Health check
    if (url.pathname === "/health" && req.method === "GET") {
      return Response.json({ status: "ok", service: "scraper-service" });
    }

    // Auth
    if (API_KEY) {
      const key = req.headers.get("x-api-key") || req.headers.get("authorization")?.replace("Bearer ", "");
      if (key !== API_KEY) {
        return Response.json({ error: "unauthorized" }, { status: 401 });
      }
    }

    if (url.pathname === "/scrape" && req.method === "POST") {
      let body: { url?: string };
      try {
        body = await req.json();
      } catch {
        return Response.json({ error: "invalid JSON" }, { status: 400 });
      }

      if (!body.url) {
        return Response.json({ error: "url is required" }, { status: 400 });
      }

      const start = Date.now();
      try {
        const data = await scrape(body.url);
        const elapsed = Date.now() - start;
        console.log(`[scraper] ${body.url} → ${elapsed}ms`);
        return Response.json({ ok: true, data, elapsed_ms: elapsed });
      } catch (e) {
        const msg = e instanceof Error ? e.message : String(e);
        console.error(`[scraper] error: ${msg}`);
        return Response.json({ ok: false, error: msg }, { status: 500 });
      }
    }

    return Response.json({ error: "not found" }, { status: 404 });
  },
});

console.log(`[scraper-service] listening on :${PORT}`);

// Graceful shutdown
process.on("SIGTERM", async () => {
  if (browser) await browser.close();
  server.stop();
});
process.on("SIGINT", async () => {
  if (browser) await browser.close();
  server.stop();
});
