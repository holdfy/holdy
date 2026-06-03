#!/usr/bin/env python3
"""
TikTok Shop scraper — Playwright + session por cookies

Uso:
    python scraper.py register           # cria conta TikTok (semi-automático)
    python scraper.py login              # abre browser visível para login manual
    python scraper.py keepalive          # renova cookies sem re-login (uso via cron)
    python scraper.py check              # verifica se a sessão ainda é válida
    python scraper.py scrape <url>       # extrai produto e imprime JSON
    python scraper.py server             # HTTP server (POST /scrape, GET /health)

Variáveis de ambiente:
    SCRAPER_PORT      porta do server (default: 4000)
    SCRAPER_API_KEY   chave de auth para o server (opcional)
    COOKIES_PATH      caminho do arquivo de cookies (default: cookies.json)
    TT_EMAIL          email para registro/login (opcional)
    TT_PASSWORD       senha para registro/login (opcional)
"""

import asyncio
import getpass
import json
import os
import random
import re
import string
import subprocess
import sys
import time
from pathlib import Path
from typing import Optional

import captcha as captcha_solver

COOKIES_PATH = Path(os.getenv("COOKIES_PATH", Path(__file__).parent / "cookies.json"))
SCRAPER_PORT = int(os.getenv("SCRAPER_PORT", "4000"))
API_KEY = os.getenv("SCRAPER_API_KEY", "")

MOBILE_UA = (
    "Mozilla/5.0 (iPhone; CPU iPhone OS 17_0 like Mac OS X) "
    "AppleWebKit/605.1.15 (KHTML, like Gecko) Version/17.0 Mobile/15E148 Safari/604.1"
)

STEALTH_SCRIPT = """
Object.defineProperty(navigator, 'webdriver', { get: () => undefined });
Object.defineProperty(navigator, 'plugins', {
    get: () => {
        const arr = [
            { name: 'Chrome PDF Plugin', filename: 'internal-pdf-viewer' },
            { name: 'Chrome PDF Viewer', filename: 'mhjfbmdgcfjbbpaeojofohoefgiehjai' },
            { name: 'Native Client', filename: 'internal-nacl-plugin' },
        ];
        arr.item = i => arr[i];
        arr.namedItem = n => arr.find(p => p.name === n) || null;
        arr.refresh = () => {};
        return arr;
    }
});
Object.defineProperty(navigator, 'languages', { get: () => ['pt-BR', 'pt', 'en-US', 'en'] });
window.chrome = { runtime: {}, loadTimes: () => ({}), csi: () => ({}) };
const origQuery = window.navigator.permissions?.query?.bind(window.navigator.permissions);
if (origQuery) {
    window.navigator.permissions.query = (p) =>
        p.name === 'notifications'
            ? Promise.resolve({ state: 'denied', onchange: null })
            : origQuery(p);
}
"""


# ── Session ────────────────────────────────────────────────────────────────────

class SessionManager:
    def __init__(self, path: Path = COOKIES_PATH):
        self.path = path

    def load(self) -> list:
        if self.path.exists():
            try:
                return json.loads(self.path.read_text())
            except Exception:
                return []
        return []

    def save(self, cookies: list):
        self.path.write_text(json.dumps(cookies, ensure_ascii=False, indent=2))
        print(f"[session] {len(cookies)} cookies salvos em {self.path}")

    def has_cookies(self) -> bool:
        cookies = self.load()
        return any(c.get("name") == "sessionid" for c in cookies)

    def session_id(self) -> Optional[str]:
        for c in self.load():
            if c.get("name") == "sessionid":
                return c.get("value")
        return None


# ── Scraper ────────────────────────────────────────────────────────────────────

class TikTokScraper:
    def __init__(self):
        self.session = SessionManager()
        self._pw = None
        self._browser = None

    async def start(self):
        from playwright.async_api import async_playwright
        import glob

        self._pw = await async_playwright().start()

        self._use_persistent = False
        self._browser = await self._pw.chromium.launch(
            headless=True,
            args=[
                "--no-sandbox",
                "--disable-setuid-sandbox",
                "--disable-dev-shm-usage",
                "--disable-blink-features=AutomationControlled",
                "--disable-web-security",
            ],
        )

    async def stop(self):
        if self._browser:
            await self._browser.close()
        if self._pw:
            await self._pw.stop()

    async def _new_context(self):
        # Contexto persistente (perfil Chrome) já é o próprio browser
        if getattr(self, "_use_persistent", False):
            return self._browser

        context = await self._browser.new_context(
            user_agent=MOBILE_UA,
            viewport={"width": 390, "height": 844},
            locale="pt-BR",
            timezone_id="America/Sao_Paulo",
            extra_http_headers={"Accept-Language": "pt-BR,pt;q=0.9,en;q=0.8"},
        )
        cookies = self.session.load()
        if cookies:
            await context.add_cookies(cookies)
        return context

    # ── Login ──────────────────────────────────────────────────────────────────

    async def login(self):
        """Abre browser com QR code do TikTok. Escaneie com o app no celular."""
        from playwright.async_api import async_playwright
        print("[login] Abrindo QR code do TikTok...")
        print("[login] Escaneie com o app TikTok no celular (Perfil → ··· → Login with QR)")

        pw = await async_playwright().start()
        browser = await pw.chromium.launch(
            headless=False,
            args=["--start-maximized"],
        )
        # Desktop UA para a página de QR code (não mobile)
        context = await browser.new_context(
            user_agent=(
                "Mozilla/5.0 (Windows NT 10.0; Win64; x64) "
                "AppleWebKit/537.36 (KHTML, like Gecko) "
                "Chrome/124.0.0.0 Safari/537.36"
            ),
            viewport={"width": 1280, "height": 800},
            locale="pt-BR",
        )
        page = await context.new_page()
        await page.add_init_script(STEALTH_SCRIPT)

        await page.goto("https://www.tiktok.com/login")
        await page.wait_for_load_state("domcontentloaded")
        await page.wait_for_timeout(3000)

        # Clicar no botão de QR code se existir
        for sel in [
            "[class*='qrcode']", "[class*='QRCode']", "[class*='qr-code']",
            "button:has-text('QR')", "[data-e2e*='qr']",
        ]:
            btn = page.locator(sel).first
            if await btn.count() > 0 and await btn.is_visible():
                await btn.click()
                print("[login] Clicou no botão QR code")
                await page.wait_for_timeout(1500)
                break

        print("[login] QR code visível no browser — escaneie agora com o celular")
        print("[login] No app TikTok: Perfil → ··· → Escanear código QR")
        print("[login] Aguardando login... (timeout: 3 minutos)")

        try:
            await page.wait_for_function(
                "() => !window.location.href.includes('/login')",
                timeout=180_000,
            )
        except Exception:
            print("[login] Timeout — tente novamente")
            await browser.close()
            await pw.stop()
            return

        # Aguarda cookies de sessão serem definidos via JS após redirect
        print("[login] Escaneado! Aguardando sessão...")
        for _ in range(15):
            await page.wait_for_timeout(1000)
            cookies = await context.cookies()
            if any(c["name"] == "sessionid" for c in cookies):
                break

        cookies = await context.cookies()
        has_session = any(c["name"] == "sessionid" for c in cookies)

        if has_session:
            self.session.save(cookies)
            print("[login] Login via QR OK! Cookies salvos.")
        else:
            print("[login] sessionid não encontrado — tente novamente")

        await browser.close()
        await pw.stop()

    # ── Register ───────────────────────────────────────────────────────────────

    async def register(self, email: Optional[str] = None, password: Optional[str] = None):
        """
        Cria uma conta TikTok em browser visível.
        - Preenche email, senha e data de nascimento automaticamente.
        - CAPTCHA e verificação de email ficam para o usuário.
        - Salva cookies ao final.
        """
        from playwright.async_api import async_playwright

        email = email or os.getenv("TT_EMAIL") or input("Email: ").strip()
        password = password or os.getenv("TT_PASSWORD") or getpass.getpass("Senha (min 8 chars, 1 número): ")

        birth_year = random.randint(1990, 2000)
        birth_month = random.randint(1, 12)
        birth_day = random.randint(1, 28)

        print(f"[register] email:    {email}")
        print(f"[register] birthday: {birth_year}-{birth_month:02d}-{birth_day:02d}")
        print("[register] Abrindo browser...")

        pw = await async_playwright().start()
        browser = await pw.chromium.launch(headless=False, args=["--start-maximized"])
        context = await browser.new_context(
            user_agent=MOBILE_UA,
            viewport={"width": 390, "height": 844},
            locale="pt-BR",
        )
        page = await context.new_page()
        await page.add_init_script(STEALTH_SCRIPT)

        await page.goto("https://www.tiktok.com/signup/phone-or-email/email")
        await page.wait_for_load_state("domcontentloaded")
        await page.wait_for_timeout(2000)

        # ── Instruções para o usuário ─────────────────────────────────────────
        print()
        print("=" * 55)
        print("  FAÇA O CADASTRO NO BROWSER:")
        print(f"  Email:  {email}")
        print(f"  Senha:  {password}")
        print()
        print("  1. Preencha email e senha no browser")
        print("  2. Clique em Avançar")
        print("  3. Resolva o CAPTCHA se aparecer")
        print(f"  4. Abra o email {email} e copie o código")
        print("  5. Cole o código no browser e finalize")
        print()
        print("  O script salva os cookies automaticamente ao final.")
        print("=" * 55)
        print()
        print("[register] Aguardando... (timeout: 15 min)")

        try:
            await page.wait_for_function(
                "() => !window.location.href.includes('/signup') && "
                "       !window.location.href.includes('/login')",
                timeout=900_000,
            )
        except Exception:
            # Mesmo com timeout, tenta salvar o que tiver
            print("[register] Timeout — tentando salvar cookies mesmo assim...")
            pass

        await page.wait_for_timeout(2000)
        cookies = await context.cookies()
        has_session = any(c["name"] == "sessionid" for c in cookies)

        if has_session:
            self.session.save(cookies)
            print("[register] Conta criada com sucesso! sessionid salvo.")
        else:
            print("[register] sessionid não encontrado — tente: python scraper.py login")

        await browser.close()
        await pw.stop()

    # ── Keepalive ──────────────────────────────────────────────────────────────

    async def keepalive(self) -> bool:
        """
        Visita o TikTok com os cookies existentes para manter a sessão ativa e
        salva os cookies atualizados (com novos timestamps/validade).
        Se a sessão estiver expirada e TWOCAPTCHA_API_KEY + TT_EMAIL + TT_PASSWORD
        estiverem configurados, faz re-login automático.
        """
        if not self.session.has_cookies():
            print("[keepalive] Sem cookies — execute: python scraper.py login")
            return False

        context = await self._new_context()
        page = await context.new_page()
        await page.add_init_script(STEALTH_SCRIPT)

        try:
            await page.goto("https://www.tiktok.com/", wait_until="domcontentloaded", timeout=20_000)
            await page.wait_for_timeout(2000)

            cookies = await context.cookies()
            has_session = any(c["name"] == "sessionid" for c in cookies)

            if has_session:
                self.session.save(cookies)
                print(f"[keepalive] OK — sessão ativa, {len(cookies)} cookies salvos")
                await context.close()
                return True

            # Sessão expirada — tentar re-login automático se credenciais disponíveis
            print("[keepalive] Sessão expirada")
            await context.close()

            email = os.getenv("TT_EMAIL", "")
            password = os.getenv("TT_PASSWORD", "")
            if email and password:
                print("[keepalive] Credenciais encontradas — tentando re-login automático")
                await self.login()
                return self.session.has_cookies()

            print("[keepalive] Configure TT_EMAIL e TT_PASSWORD no .env para re-login automático")
            return False

        except Exception as e:
            print(f"[keepalive] Erro: {e}")
            await context.close()
            return False

    # ── Check ──────────────────────────────────────────────────────────────────

    async def check_session(self) -> bool:
        """Verifica se a sessão ainda é válida via API leve do TikTok."""
        if not self.session.has_cookies():
            print("[check] Sem sessionid — execute: python scraper.py login")
            return False

        context = await self._new_context()
        page = await context.new_page()
        await page.add_init_script(STEALTH_SCRIPT)

        try:
            resp = await page.request.get(
                "https://www.tiktok.com/api/user/detail/?uniqueId=tiktok",
                headers={"Referer": "https://www.tiktok.com/"},
            )
            data = await resp.json()
            # Se retornar userInfo sem erro de auth, sessão OK
            if data.get("statusCode") in (0, None) and "userInfo" in data:
                print("[check] Sessão válida.")
                await context.close()
                return True
            # statusCode 10102/10111 = não autenticado
            if data.get("statusCode") in (10102, 10111, -1):
                print("[check] Sessão expirada — execute: python scraper.py login")
                await context.close()
                return False
            # Resposta ambígua — tentar verificar via página de conta
            print(f"[check] Resposta ambígua: {data.get('statusCode')} — assumindo válida")
            await context.close()
            return True
        except Exception as e:
            print(f"[check] Erro ao verificar sessão: {e}")
            await context.close()
            return False

    # ── Scrape ─────────────────────────────────────────────────────────────────

    async def scrape(self, url: str) -> Optional[dict]:
        """Extrai dados completos de um produto TikTok Shop."""
        target_url = await self._resolve_short_url(url)
        print(f"[scrape] target: {target_url[:90]}")

        context = await self._new_context()
        api_data: dict = {}

        async def capture_response(response):
            ru = response.url
            if any(k in ru for k in ["/api/commerce", "/api/shop", "item/detail", "product/detail"]):
                if response.status == 200:
                    try:
                        body = await response.text()
                        if body.startswith("{") and ("price" in body or "image" in body):
                            api_data[ru] = json.loads(body)
                    except Exception:
                        pass

        try:
            page = await context.new_page()
            await page.add_init_script(STEALTH_SCRIPT)
            page.on("response", lambda r: asyncio.create_task(capture_response(r)))

            try:
                await page.goto(target_url, wait_until="domcontentloaded", timeout=30_000)
            except Exception as e:
                print(f"[scrape] goto warning: {str(e)[:80]}")

            # Scroll para disparar lazy-load de imagens
            await self._scroll_page(page)

            # Verificar CAPTCHA / security check
            page_title = await page.title()
            if any(k in page_title.lower() for k in ["verify", "security check", "robot"]):
                print("[scrape] CAPTCHA detectado — sessão precisa ser renovada")
                await context.close()
                return None

            # Extrair do DOM
            dom = await page.evaluate(DOM_EXTRACTION_SCRIPT)

            if not dom.get("title"):
                # Tentar capturar título mínimo do og_info do redirect
                og = _parse_og_info(target_url)
                if og:
                    dom["title"] = og.get("title", "")
                    if not dom.get("images") and og.get("image"):
                        dom["images"] = [og["image"]]

            if not dom.get("title"):
                print("[scrape] Título não encontrado — produto não carregou")
                await context.close()
                return None

            # Mesclar com dados interceptados da API interna
            api_price, api_images = _extract_from_api_data(api_data)
            all_images = _dedupe_images(api_images + dom.get("images", []))

            product_id = _extract_product_id(target_url)

            product = {
                "title": dom["title"],
                "description": dom.get("description") or None,
                "price": api_price or dom.get("price") or None,
                "price_original": dom.get("price_original") or None,
                "currency": "BRL",
                "images": all_images,
                "video_url": None,
                "seller_name": dom.get("seller_name") or None,
                "seller_rating": dom.get("seller_rating") or None,
                "product_id": product_id,
                "source_url": url,
                "platform": "tiktok",
            }

            print(
                f"[scrape] OK: '{product['title'][:50]}' | "
                f"{len(all_images)} imagens | preço: {product['price']}"
            )
            if not getattr(self, "_use_persistent", False):
                await context.close()
            return product

        except Exception as e:
            print(f"[scrape] erro: {e}")
            if not getattr(self, "_use_persistent", False):
                await context.close()
            return None

    async def _scroll_page(self, page):
        """Scroll suave para disparar lazy-load de imagens."""
        try:
            await page.evaluate("""async () => {
                await new Promise(resolve => {
                    let total = document.body.scrollHeight;
                    let current = 0;
                    const step = 300;
                    const delay = 120;
                    const timer = setInterval(() => {
                        window.scrollBy(0, step);
                        current += step;
                        if (current >= total) { clearInterval(timer); resolve(); }
                    }, delay);
                    setTimeout(() => { clearInterval(timer); resolve(); }, 5000);
                });
            }""")
            await page.wait_for_timeout(1500)
            await page.evaluate("window.scrollTo(0, 0)")
        except Exception:
            pass

    async def _resolve_short_url(self, url: str) -> str:
        """Resolve vt.tiktok.com → URL completa do produto."""
        if "vt.tiktok.com" not in url:
            return url
        try:
            proc = subprocess.run(
                ["curl", "-sI", "-A", MOBILE_UA, "--max-redirs", "0", url],
                capture_output=True, text=True, timeout=10,
            )
            for line in proc.stdout.splitlines():
                if line.lower().startswith("location:"):
                    location = line.split(":", 1)[1].strip()
                    if "tiktok.com" in location and "/login" not in location:
                        return location
        except Exception as e:
            print(f"[resolve] erro: {e}")
        return url


# ── DOM extraction script ──────────────────────────────────────────────────────

DOM_EXTRACTION_SCRIPT = """() => {
    const get = (...sels) => {
        for (const s of sels) {
            const el = document.querySelector(s);
            if (el?.textContent?.trim()) return el.textContent.trim();
        }
        return null;
    };

    const title = get(
        "[data-testid='product-title']", ".product-title",
        "h1[class*='title']", "h1"
    );

    const price = get(
        "[data-testid='product-price']", "[class*='salePrice']",
        "[class*='current-price']", "[class*='CurrentPrice']",
        "[data-e2e='price']", ".price-sale", "[class*='Price'] span"
    );

    const price_original = get(
        "[class*='original-price']", "[class*='originalPrice']",
        "[class*='OriginalPrice']", "[class*='price-through']", "del", "s"
    );

    const description = get(
        "[data-testid='product-description']",
        "[class*='description']", "[class*='detail'] p"
    );

    const seller_name = get(
        "[data-testid='shop-name']", "[class*='shop-name']",
        "[class*='shopName']", "[class*='seller-name']"
    );

    const seller_rating = get(
        "[data-testid='shop-rating']", "[class*='rating'][class*='shop']"
    );

    // Coletar todas as imagens do produto
    const images = [];
    const seen = new Set();
    const imgSels = [
        "[data-testid='product-image'] img",
        "[class*='thumbnail'] img",
        "[class*='gallery'] img",
        "[class*='swiper'] img",
        "[class*='carousel'] img",
        "[class*='product-img'] img",
        "[class*='ProductImg'] img",
    ];
    for (const sel of imgSels) {
        document.querySelectorAll(sel).forEach(img => {
            const src = img.src || img.dataset.src || img.dataset.lazySrc;
            if (src?.startsWith("http") && !seen.has(src)) {
                seen.add(src);
                images.push(src);
            }
        });
    }
    // Fallback: todas as imagens que parecem ser do TikTok/ByteDance
    if (images.length === 0) {
        document.querySelectorAll("img").forEach(img => {
            const src = img.src || "";
            if (
                src.startsWith("http") &&
                (src.includes("tiktok") || src.includes("ibyteimg") || src.includes("bytedance")) &&
                !seen.has(src)
            ) {
                seen.add(src);
                images.push(src);
            }
        });
    }

    return { title, price, price_original, description, images, seller_name, seller_rating };
}"""


# ── Helpers ────────────────────────────────────────────────────────────────────

def _import_chrome_cookies():
    """Exporta cookies do TikTok do Chrome para cookies.json."""
    import sqlite3, shutil, tempfile
    src = os.path.expanduser("~/.config/google-chrome/Default/Cookies")
    if not os.path.exists(src):
        print("[import-chrome] Arquivo de cookies do Chrome não encontrado")
        return
    tmp = tempfile.mktemp(suffix=".db")
    shutil.copy2(src, tmp)
    conn = sqlite3.connect(tmp)
    rows = conn.execute(
        "SELECT name, value, host_key, path, expires_utc, is_secure, is_httponly, samesite "
        "FROM cookies WHERE host_key LIKE '%tiktok%'"
    ).fetchall()
    conn.close()
    os.unlink(tmp)
    same_map = {0: "Strict", 1: "Lax", 2: "None", -1: "None"}
    cookies = []
    for name, value, domain, path, expires, secure, httponly, samesite in rows:
        unix_ts = (expires / 1_000_000) - 11644473600 if expires > 0 else -1
        cookies.append({
            "name": name, "value": value, "domain": domain, "path": path,
            "expires": unix_ts, "secure": bool(secure),
            "httpOnly": bool(httponly), "sameSite": same_map.get(samesite, "None"),
        })
    COOKIES_PATH.write_text(json.dumps(cookies, indent=2))
    has_session = any(c["name"] == "sessionid" for c in cookies)
    print(f"[import-chrome] {len(cookies)} cookies exportados → {COOKIES_PATH}")
    print(f"[import-chrome] sessionid presente: {has_session}")


def _generate_username() -> str:
    """Gera username único no formato user_XXXXXX."""
    suffix = "".join(random.choices(string.ascii_lowercase + string.digits, k=8))
    return f"user_{suffix}"


async def _type_humanlike(page, locator, text: str):
    """Digita texto com delays aleatórios para parecer humano."""
    await locator.fill("")
    for char in text:
        await locator.press(char)
        await page.wait_for_timeout(random.randint(60, 180))


async def _click_button(page, labels: list) -> bool:
    """Tenta clicar no primeiro botão visível cujo texto bate com algum da lista."""
    for label in labels:
        for sel in [
            f"button:has-text('{label}')",
            f"[role='button']:has-text('{label}')",
            f"input[type='submit'][value*='{label}' i]",
        ]:
            btn = page.locator(sel).first
            try:
                if await btn.count() > 0 and await btn.is_visible():
                    await btn.click()
                    return True
            except Exception:
                pass
    return False


def _extract_product_id(url: str) -> Optional[str]:
    m = re.search(r"product/(\d+)", url)
    return m.group(1) if m else None


def _parse_og_info(url: str) -> Optional[dict]:
    """Extrai og_info do query string do redirect do TikTok."""
    from urllib.parse import urlparse, parse_qs, unquote
    try:
        qs = parse_qs(urlparse(url).query)
        og_raw = qs.get("og_info", [None])[0]
        if not og_raw:
            return None
        og = json.loads(unquote(og_raw))
        image = og.get("image", "").replace("\\/", "/")
        return {"title": og.get("title", ""), "image": image if image else None}
    except Exception:
        return None


def _find_deep(obj, keys: list) -> Optional[str]:
    if not isinstance(obj, dict):
        return None
    for k in keys:
        if k in obj and obj[k]:
            return str(obj[k])
    for v in obj.values():
        result = _find_deep(v, keys)
        if result:
            return result
    return None


def _find_deep_array(obj, keys: list) -> list:
    if not isinstance(obj, (dict, list)):
        return []
    result = []
    if isinstance(obj, list):
        for item in obj:
            result.extend(_find_deep_array(item, keys))
        return result
    for k, v in obj.items():
        if k in keys and isinstance(v, list):
            for item in v:
                if isinstance(item, str) and item.startswith("http"):
                    result.append(item)
                elif isinstance(item, dict):
                    src = item.get("url") or item.get("src") or item.get("uri")
                    if src:
                        result.append(src)
        else:
            result.extend(_find_deep_array(v, keys))
    return result


def _extract_from_api_data(api_data: dict) -> tuple[Optional[str], list]:
    price = None
    images = []
    for data in api_data.values():
        if not price:
            price = _find_deep(data, ["sale_price", "price", "currentPrice", "salePrice", "display_price"])
        images.extend(_find_deep_array(data, ["images", "img_urls", "imageList", "image_list"]))
    return price, images


def _dedupe_images(urls: list) -> list:
    seen: set = set()
    result = []
    for u in urls:
        if u not in seen and u.startswith("http") and any(
            d in u for d in ["tiktok", "ibyteimg", "bytedance"]
        ):
            seen.add(u)
            result.append(u)
    return result


# ── HTTP Server ────────────────────────────────────────────────────────────────

async def run_server(scraper: TikTokScraper):
    try:
        from aiohttp import web
    except ImportError:
        print("[server] aiohttp não instalado — execute: pip install aiohttp")
        sys.exit(1)

    async def handle_health(request):
        valid = scraper.session.has_cookies()
        return web.json_response({
            "status": "ok",
            "service": "tiktok-scraper-python",
            "session": "valid" if valid else "missing",
        })

    async def handle_scrape(request):
        if API_KEY:
            key = (
                request.headers.get("x-api-key")
                or request.headers.get("Authorization", "").replace("Bearer ", "")
            )
            if key != API_KEY:
                return web.json_response({"error": "unauthorized"}, status=401)

        try:
            body = await request.json()
        except Exception:
            return web.json_response({"error": "invalid JSON"}, status=400)

        url = body.get("url")
        if not url:
            return web.json_response({"error": "url is required"}, status=400)

        t0 = time.time()
        data = await scraper.scrape(url)
        elapsed = int((time.time() - t0) * 1000)

        if data:
            return web.json_response({"ok": True, "data": data, "elapsed_ms": elapsed})
        return web.json_response({"ok": False, "error": "scrape failed"}, status=500)

    app = web.Application()
    app.router.add_get("/health", handle_health)
    app.router.add_post("/scrape", handle_scrape)

    runner = web.AppRunner(app)
    await runner.setup()
    site = web.TCPSite(runner, "0.0.0.0", SCRAPER_PORT)
    await site.start()
    print(f"[server] escutando em :{SCRAPER_PORT}")
    await asyncio.Event().wait()


# ── CLI ────────────────────────────────────────────────────────────────────────

async def main():
    cmd = sys.argv[1] if len(sys.argv) > 1 else "help"

    if cmd == "import-chrome":
        _import_chrome_cookies()

    elif cmd == "keepalive":
        scraper = TikTokScraper()
        await scraper.start()
        try:
            ok = await scraper.keepalive()
            sys.exit(0 if ok else 1)
        finally:
            await scraper.stop()

    elif cmd == "register":
        email = sys.argv[2] if len(sys.argv) > 2 else None
        password = sys.argv[3] if len(sys.argv) > 3 else None
        scraper = TikTokScraper()
        await scraper.register(email, password)

    elif cmd == "login":
        scraper = TikTokScraper()
        await scraper.login()

    elif cmd == "check":
        scraper = TikTokScraper()
        await scraper.start()
        try:
            valid = await scraper.check_session()
            sys.exit(0 if valid else 1)
        finally:
            await scraper.stop()

    elif cmd == "scrape":
        if len(sys.argv) < 3:
            print("Uso: python scraper.py scrape <url>")
            sys.exit(1)
        url = sys.argv[2]
        scraper = TikTokScraper()
        await scraper.start()
        try:
            result = await scraper.scrape(url)
            if result:
                print(json.dumps(result, ensure_ascii=False, indent=2))
            else:
                print("[erro] Scrape falhou — verifique o login", file=sys.stderr)
                sys.exit(1)
        finally:
            await scraper.stop()

    elif cmd == "server":
        scraper = TikTokScraper()
        await scraper.start()
        try:
            await run_server(scraper)
        finally:
            await scraper.stop()

    else:
        print(__doc__)


if __name__ == "__main__":
    asyncio.run(main())
