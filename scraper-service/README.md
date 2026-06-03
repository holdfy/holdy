# scraper-service

Scraper headless para TikTok Shop com gerenciamento de sessão por cookies e resolução automática de CAPTCHA via 2captcha.

## Visão geral

```
vt.tiktok.com/XYZ  →  resolve redirect  →  product page  →  extrai dados
                                                ↑
                                         cookies salvos
                                         (sessionid)
```

**O que extrai:** título, preço, preço original, todas as imagens, descrição, seller, rating, product_id.

**Por que cookies em vez de scraping puro:** o TikTok bloqueia requests sem autenticação com Security Check / CAPTCHA. Com `sessionid` de uma conta real, a página carrega normalmente.

---

## Setup

### Pré-requisitos

- Python 3.11+
- `curl` disponível no PATH

```bash
cd scraper-service

# Criar ambiente virtual
python3 -m venv .venv
source .venv/bin/activate   # Windows: .venv\Scripts\activate

# Instalar dependências
pip install -r requirements.txt

# Instalar o Chromium do Playwright
python -m playwright install chromium
```

---

## Variáveis de ambiente

| Variável | Default | Descrição |
|---|---|---|
| `SCRAPER_PORT` | `4000` | Porta do servidor HTTP |
| `SCRAPER_API_KEY` | _(vazio)_ | Chave de autenticação para o server (opcional) |
| `COOKIES_PATH` | `cookies.json` | Caminho do arquivo de sessão |
| `TT_EMAIL` | _(vazio)_ | Email para `register` (evita prompt interativo) |
| `TT_PASSWORD` | _(vazio)_ | Senha para `register` |
| `TWOCAPTCHA_API_KEY` | _(vazio)_ | Chave do 2captcha para resolução automática de CAPTCHA |

Coloque no `.env` do projeto (`money/.env`) ou exporte antes de rodar.

---

## Comandos

### Criar conta TikTok

```bash
python scraper.py register
# ou com variáveis de ambiente:
TT_EMAIL=seu@email.com TT_PASSWORD=SuaSenha123 python scraper.py register
```

**O que acontece:**
1. Abre Chromium visível
2. Navega para `tiktok.com/signup`
3. Preenche automaticamente: email, senha, data de nascimento, username
4. Se `TWOCAPTCHA_API_KEY` estiver configurado: resolve o CAPTCHA sozinho
5. Exibe instruções para verificar o email
6. Aguarda você inserir o código de verificação
7. Salva `cookies.json` com o `sessionid`

> Precisa fazer isso **uma única vez**. Os cookies duram 30–90 dias.

---

### Login em conta existente

```bash
python scraper.py login
```

Abre Chromium visível na página de login. Você faz o login normalmente (email/senha ou QR code), o script detecta o sucesso e salva os cookies.

---

### Verificar validade da sessão

```bash
python scraper.py check
# exit 0 = válida
# exit 1 = expirada (rode login novamente)
```

Útil para cron/health check:

```bash
python scraper.py check || python scraper.py login
```

---

### Scrape de produto

```bash
python scraper.py scrape "https://vt.tiktok.com/ZS92yB9L2jSSV-lvcvt/"
```

Saída JSON:

```json
{
  "title": "BETTDOW AI07 Caneta Stylus para iPad...",
  "description": "...",
  "price": "89.90",
  "price_original": "129.90",
  "currency": "BRL",
  "images": [
    "https://p16-oec-sg.ibyteimg.com/...",
    "https://p16-oec-sg.ibyteimg.com/..."
  ],
  "seller_name": "silviosaczuck",
  "seller_rating": "4.8",
  "product_id": "1735493909540079467",
  "source_url": "https://vt.tiktok.com/ZS92yB9L2jSSV-lvcvt/",
  "platform": "tiktok"
}
```

---

### Servidor HTTP (integração com Rust)

```bash
SCRAPER_PORT=4000 python scraper.py server
```

Compatível com o extrator Rust (`apicash-importer`) via `SCRAPER_URL=http://localhost:4000`.

**Endpoints:**

```
GET  /health
     → {"status":"ok","session":"valid"|"missing"}

POST /scrape
     Content-Type: application/json
     x-api-key: <SCRAPER_API_KEY>   (só se configurado)
     {"url": "https://vt.tiktok.com/..."}
     → {"ok": true, "data": {...}, "elapsed_ms": 4200}
     → {"ok": false, "error": "..."} em caso de falha
```

---

## Resolução automática de CAPTCHA (2captcha)

O TikTok usa um CAPTCHA do tipo **slider puzzle** (arraste a peça para encaixar). Quando `TWOCAPTCHA_API_KEY` está configurado, o scraper resolve automaticamente.

### Configurar 2captcha

1. Criar conta em [2captcha.com](https://2captcha.com)
2. Depositar crédito mínimo (~$3 dura meses para uso pessoal)
3. Copiar a API key no painel
4. Adicionar no `.env`:

```bash
TWOCAPTCHA_API_KEY=sua_chave_aqui
```

### Como funciona internamente

```
detecta CAPTCHA na página
    ↓
screenshot do container
    ↓
POST /in.php  (base64 da imagem, método coordinates)
    ↓
poll /res.php até resolver (~10–30s)
    ↓
recebe x,y do destino do slider
    ↓
Playwright faz drag com movimento gradual (simula humano)
    ↓
verifica se CAPTCHA sumiu
```

Arquivo responsável: `captcha.py`

### Custo estimado

| Situação | Frequência | Custo |
|---|---|---|
| Criar conta (registro) | 1x | ~$0.002 |
| Login após expiração de cookie | A cada 30–90 dias | ~$0.002 |
| Scrape de produto | **Zero** (sem CAPTCHA com sessão válida) | $0 |

> Para uso pessoal, $3 de crédito dura praticamente para sempre.

### Sem 2captcha

Se `TWOCAPTCHA_API_KEY` não estiver configurado:
- CAPTCHA no **login/registro**: você resolve manualmente no browser visível
- CAPTCHA no **scrape**: não acontece (sessão autenticada não recebe CAPTCHA)

---

## Estratégia de extração de dados

O scraper usa três fontes em paralelo, mesclando os resultados:

| Fonte | Dados disponíveis |
|---|---|
| **Interceptação de API** (`/api/commerce`, `/api/shop`) | Preço real, lista completa de imagens da API |
| **DOM após render JS** | Título, preço exibido, seller, rating, imagens visíveis |
| **og_info do redirect** | Fallback: título + 1 imagem (sem JS necessário) |

O scroll automático da página força o lazy-load de todas as imagens antes da extração.

---

## Gerenciamento de sessão

Os cookies ficam em `cookies.json` (gitignored). O cookie crítico é `sessionid`.

**Duração típica:** 30–90 dias dependendo da atividade da conta.

**Renovação:**

```bash
# Verificar e renovar se necessário (ideal para cron diário)
python scraper.py check || python scraper.py login
```

**Com 2captcha configurado**, o login pode ser completamente automatizado se a senha estiver no `.env`:

```bash
TT_EMAIL=conta@email.com TT_PASSWORD=Senha123 TWOCAPTCHA_API_KEY=xxx \
  python scraper.py check || python scraper.py login
```

---

## Integração com APICash

No `money/.env`:

```bash
SCRAPER_URL=http://127.0.0.1:4000
SCRAPER_API_KEY=                    # deixe vazio se não quiser auth
```

O extrator Rust (`apicash-importer`) chama `POST /scrape` automaticamente para URLs do TikTok antes de tentar o fallback de redirect.

Subir junto com os outros serviços:

```bash
cd money
./runapp.sh start all   # inclui scraper-service se presente
```

---

## Arquivos

```
scraper-service/
├── scraper.py          # CLI + HTTP server + lógica de extração
├── captcha.py          # Integração 2captcha (slide CAPTCHA)
├── requirements.txt    # playwright, aiohttp, 2captcha-python
├── cookies.json        # sessão TikTok (gitignored, gerado em runtime)
└── .venv/              # ambiente virtual Python (gitignored)
```

---

## Troubleshooting

**"sessionid não encontrado"** → execute `python scraper.py login`

**"CAPTCHA detectado no scrape"** → sessão expirou, execute login novamente

**"goto warning: net::ERR_ABORTED"** → normal no TikTok, o scraper continua mesmo assim

**Imagens não carregam** → o TikTok pode ter mudado os seletores CSS — abra `scraper.py` e atualize `DOM_EXTRACTION_SCRIPT`
