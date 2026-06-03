"""
2captcha integration para TikTok slide CAPTCHA.

Documentação: https://2captcha.com/api-docs/coordinates
Custo: ~$3 por 1000 CAPTCHAs

Variável de ambiente obrigatória:
    TWOCAPTCHA_API_KEY   chave da API do 2captcha

Fluxo:
    1. Detecta CAPTCHA na página pelo seletor
    2. Tira screenshot do container
    3. Envia para 2captcha (método coordinates)
    4. Recebe x,y de onde soltar o slider
    5. Executa o drag no Playwright com movimento gradual
"""

import base64
import os
import re
from typing import Optional

TWOCAPTCHA_KEY = os.getenv("TWOCAPTCHA_API_KEY", "")

# Seletores conhecidos do CAPTCHA do TikTok
CAPTCHA_SELECTORS = [
    "[class*='captcha-verify']",
    "[class*='captcha_verify']",
    "[id*='captcha']",
    "[class*='Secsdk']",
    "[class*='secsdk']",
    "iframe[src*='captcha']",
]

SLIDER_SELECTORS = [
    "[class*='captcha-slider']",
    "[class*='slider-btn']",
    "[class*='drag-btn']",
    "[class*='sliderbtn']",
    "[class*='handler']",
    "[class*='Slider']",
]


async def detect_captcha(page) -> bool:
    """Retorna True se há CAPTCHA visível na página."""
    for sel in CAPTCHA_SELECTORS:
        el = page.locator(sel).first
        try:
            if await el.count() > 0 and await el.is_visible():
                return True
        except Exception:
            pass

    title = await page.title()
    return any(k in title.lower() for k in ["verify", "security check"])


async def solve(page) -> bool:
    """
    Tenta resolver o CAPTCHA da página atual via 2captcha.
    Retorna True se resolveu, False se não havia CAPTCHA ou falhou.
    """
    if not TWOCAPTCHA_KEY:
        print("[captcha] TWOCAPTCHA_API_KEY não configurado — ignorando CAPTCHA")
        return False

    if not await detect_captcha(page):
        return False

    print("[captcha] CAPTCHA detectado — enviando para 2captcha...")

    try:
        from twocaptcha import TwoCaptcha  # type: ignore
    except ImportError:
        print("[captcha] 2captcha-python não instalado — execute: pip install 2captcha-python")
        return False

    try:
        solver = TwoCaptcha(TWOCAPTCHA_KEY)

        # Tirar screenshot do container do CAPTCHA
        container = None
        for sel in CAPTCHA_SELECTORS:
            el = page.locator(sel).first
            if await el.count() > 0 and await el.is_visible():
                container = el
                break

        if container is None:
            screenshot_bytes = await page.screenshot()
        else:
            screenshot_bytes = await container.screenshot()

        image_b64 = base64.b64encode(screenshot_bytes).decode()

        # Enviar para 2captcha como task de coordenadas
        result = solver.coordinates(
            image=image_b64,
            textinstructions="Drag the slider piece to complete the image puzzle",
            lang="en",
        )
        coords = _parse_coords(result.get("code", ""))
        if not coords:
            print(f"[captcha] Resposta inesperada: {result}")
            return False

        dest_x, dest_y = coords
        print(f"[captcha] Destino recebido: x={dest_x}, y={dest_y}")

        # Localizar o slider
        slider = None
        for sel in SLIDER_SELECTORS:
            el = page.locator(sel).first
            if await el.count() > 0 and await el.is_visible():
                slider = el
                break

        if slider is None:
            print("[captcha] Slider não encontrado na página")
            return False

        box = await slider.bounding_box()
        if not box:
            return False

        start_x = box["x"] + box["width"] / 2
        start_y = box["y"] + box["height"] / 2

        # Drag com movimento gradual (simula humano)
        await page.mouse.move(start_x, start_y)
        await page.mouse.down()
        await page.wait_for_timeout(200)

        steps = 30
        for i in range(1, steps + 1):
            t = i / steps
            # Easing suave
            ease = t * t * (3 - 2 * t)
            cx = start_x + (dest_x - start_x) * ease
            cy = start_y + (dest_y - start_y) * ease
            await page.mouse.move(cx, cy)
            await page.wait_for_timeout(15)

        await page.wait_for_timeout(300)
        await page.mouse.up()
        await page.wait_for_timeout(1500)

        # Verificar se o CAPTCHA sumiu
        still_present = await detect_captcha(page)
        if still_present:
            print("[captcha] CAPTCHA ainda presente — pode ter errado")
            return False

        print("[captcha] Resolvido!")
        return True

    except Exception as e:
        print(f"[captcha] Erro: {e}")
        return False


def _parse_coords(code: str) -> Optional[tuple[int, int]]:
    """Parse 'x=123,y=45' ou '123,45' → (x, y)."""
    m = re.search(r"x=(\d+)[,;]y=(\d+)", code)
    if m:
        return int(m.group(1)), int(m.group(2))
    m = re.search(r"(\d+)[,;](\d+)", code)
    if m:
        return int(m.group(1)), int(m.group(2))
    return None
