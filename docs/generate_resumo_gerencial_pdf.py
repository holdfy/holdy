#!/usr/bin/env python3
"""Gera PDF: resumo gerencial Holdfy — tecnologias e funcionalidades."""

from datetime import date
from pathlib import Path

from reportlab.lib import colors
from reportlab.lib.enums import TA_CENTER, TA_JUSTIFY
from reportlab.lib.pagesizes import A4
from reportlab.lib.styles import ParagraphStyle, getSampleStyleSheet
from reportlab.lib.units import cm
from reportlab.graphics.shapes import Drawing, Line, Rect, String
from reportlab.platypus import (
    PageBreak,
    Paragraph,
    SimpleDocTemplate,
    Spacer,
    Table,
    TableStyle,
)

OUT = Path(__file__).resolve().parent / "holdfy-resumo-gerencial.pdf"

BLUE = colors.HexColor("#1a365d")
ACCENT = colors.HexColor("#2c5282")
LIGHT = colors.HexColor("#ebf8ff")
GREY = colors.grey


def make_architecture_diagram() -> Drawing:
    """Visão simplificada das camadas do monorepo."""
    w, h = 17 * cm, 9.5 * cm
    d = Drawing(w, h)

    def layer(x, y, bw, bh, title, items, fill):
        d.add(Rect(x, y, bw, bh, fillColor=fill, strokeColor=ACCENT, strokeWidth=1))
        d.add(String(x + bw / 2, y + bh - 14, title, fontSize=9, textAnchor="middle", fillColor=ACCENT))
        iy = y + bh - 28
        for item in items:
            d.add(String(x + 8, iy, f"• {item}", fontSize=7, fillColor=colors.black))
            iy -= 11

    layer(0.3 * cm, 6.8 * cm, 16.4 * cm, 2.2 * cm, "Frontends", [
        "site/ (React + Vite + Tailwind/shadcn)",
        "front-gatebox, holdfy-admin (React + MUI)",
        "app_banco, apprastreio (Flutter)",
        "apicash-frontend (Leptos SSR)",
    ], colors.HexColor("#f0fff4"))

    layer(0.3 * cm, 3.6 * cm, 16.4 * cm, 2.8 * cm, "Backends", [
        "APICash — 15 crates Rust (Axum, Tokio, SQLx) :3000",
        "Gatebox — gateway PIX Rust :8081",
        "wallet, backend_banco (Go) | apprastreio, simuladores (Rust)",
        "scraper-service (Python Playwright / Bun)",
    ], LIGHT)

    layer(0.3 * cm, 0.5 * cm, 16.4 * cm, 2.6 * cm, "Infra + Externos", [
        "Postgres 16, Redis 7, NATS, MinIO, MongoDB (Docker Compose)",
        "Stellar Testnet + Soroban (BRLx, escrow) | Parceiros PIX",
        "WhatsApp multi-device | x402 (Base Sepolia, opcional)",
    ], colors.HexColor("#edf2f7"))

    d.add(String(0.2 * cm, h - 0.35 * cm, "Figura 1 — Arquitetura em camadas do monorepo Holdfy", fontSize=8, fillColor=GREY))
    return d


def make_flow_diagram() -> Drawing:
    """Fluxo de negócio principal."""
    w, h = 17 * cm, 3.2 * cm
    d = Drawing(w, h)

    steps = [
        "Proposta",
        "Anti-fraude",
        "PIX IN",
        "BRLx + Escrow",
        "Entrega",
        "Release",
        "PIX OUT",
    ]
    box_w = 2.0 * cm
    gap = 0.35 * cm
    y = 1.2 * cm
    x = 0.4 * cm

    for i, step in enumerate(steps):
        d.add(Rect(x, y, box_w, 0.9 * cm, fillColor=LIGHT, strokeColor=ACCENT, strokeWidth=0.8))
        d.add(String(x + box_w / 2, y + 0.32 * cm, step, fontSize=6.5, textAnchor="middle", fillColor=ACCENT))
        if i < len(steps) - 1:
            x2 = x + box_w
            d.add(Line(x2, y + 0.45 * cm, x2 + gap, y + 0.45 * cm, strokeColor=ACCENT, strokeWidth=1))
        x += box_w + gap

    d.add(String(0.2 * cm, h - 0.35 * cm, "Figura 2 — Fluxo P2P: pagamento protegido até liberação ao vendedor", fontSize=8, fillColor=GREY))
    return d


def _table(data, col_widths):
    t = Table(data, colWidths=col_widths)
    t.setStyle(
        TableStyle(
            [
                ("BACKGROUND", (0, 0), (-1, 0), ACCENT),
                ("TEXTCOLOR", (0, 0), (-1, 0), colors.whitesmoke),
                ("FONTNAME", (0, 0), (-1, 0), "Helvetica-Bold"),
                ("FONTSIZE", (0, 0), (-1, -1), 8),
                ("GRID", (0, 0), (-1, -1), 0.5, colors.grey),
                ("VALIGN", (0, 0), (-1, -1), "TOP"),
                ("ROWBACKGROUNDS", (0, 1), (-1, -1), [colors.white, colors.HexColor("#edf2f7")]),
                ("LEFTPADDING", (0, 0), (-1, -1), 6),
                ("RIGHTPADDING", (0, 0), (-1, -1), 6),
                ("TOPPADDING", (0, 0), (-1, -1), 5),
                ("BOTTOMPADDING", (0, 0), (-1, -1), 5),
            ]
        )
    )
    return t


def build():
    doc = SimpleDocTemplate(
        str(OUT),
        pagesize=A4,
        rightMargin=2 * cm,
        leftMargin=2 * cm,
        topMargin=2 * cm,
        bottomMargin=2 * cm,
        title="Holdfy — Resumo Gerencial",
        author="Holdfy",
    )
    styles = getSampleStyleSheet()
    title_style = ParagraphStyle(
        "Title",
        parent=styles["Heading1"],
        fontSize=22,
        spaceAfter=10,
        alignment=TA_CENTER,
        textColor=BLUE,
    )
    h2 = ParagraphStyle("H2", parent=styles["Heading2"], fontSize=13, spaceBefore=12, spaceAfter=6, textColor=ACCENT)
    h3 = ParagraphStyle("H3", parent=styles["Heading3"], fontSize=11, spaceBefore=8, spaceAfter=4, textColor=BLUE)
    body = ParagraphStyle("Body", parent=styles["Normal"], fontSize=10, leading=14, alignment=TA_JUSTIFY, spaceAfter=6)
    bullet = ParagraphStyle("Bullet", parent=body, leftIndent=12, spaceAfter=3)

    story = []
    story.append(Paragraph("Holdfy — Resumo Gerencial", title_style))
    story.append(
        Paragraph(
            f"Tecnologias e funcionalidades do monorepo · {date.today().strftime('%d/%m/%Y')}",
            ParagraphStyle("Sub", parent=styles["Normal"], fontSize=9, alignment=TA_CENTER, textColor=GREY),
        )
    )
    story.append(Spacer(1, 0.4 * cm))

    story.append(Paragraph("Visão do produto", h2))
    story.append(
        Paragraph(
            "O <b>Holdfy</b> (repositório pos-nearx) é um <b>monorepo fintech/marketplace</b> que integra "
            "pagamentos PIX, custódia com blockchain Stellar/Soroban e experiência multicanal "
            "(site web, WhatsApp e apps mobile). O fluxo central é a <b>compra P2P com proteção</b>: "
            "o comprador paga via PIX, o valor permanece em custódia e só é liberado ao vendedor "
            "após confirmação de entrega.",
            body,
        )
    )

    story.append(Paragraph("Arquitetura", h2))
    story.append(make_architecture_diagram())
    story.append(Spacer(1, 0.2 * cm))
    story.append(make_flow_diagram())
    story.append(Spacer(1, 0.3 * cm))
    story.append(
        Paragraph(
            "Configuração centralizada em <b>money/.env</b>. Infraestrutura via Docker Compose "
            "e scripts <b>runinfra.sh</b> / <b>runapp.sh</b>.",
            body,
        )
    )

    story.append(Paragraph("Aplicações principais", h2))

    story.append(Paragraph("1. APICash — Plataforma fintech (Rust)", h3))
    story.append(_table([
        ["Aspecto", "Detalhe"],
        ["Stack", "Rust 1.85+, Axum, Tokio, SQLx, Leptos SSR, JWT, Redis, NATS"],
        ["Blockchain", "Stellar Testnet, token BRLx, contratos Soroban (escrow)"],
        ["Portas", "Core :3000 · Admin :3001 · Frontend :3002 · WhatsApp :3010"],
    ], [3.5 * cm, 12.5 * cm]))
    story.append(Spacer(1, 0.15 * cm))
    for item in [
        "Pedidos P2P: propostas, pagamento PIX, custódia em escrow",
        "Anti-fraude com score 0–1000 (CPF/CNPJ, redes sociais, comportamento)",
        "On/off-ramp PIX ↔ BRLx via Anchor Stellar",
        "Bot WhatsApp conversacional (pareamento multi-device)",
        "Disputas, notificações, importador de produtos (TikTok Shop)",
        "Logística integrada (rastreio simulado ou APIs reais)",
        "Protocolo x402 — micropagamentos USDC (Base Sepolia, opcional)",
    ]:
        story.append(Paragraph(f"• {item}", bullet))

    story.append(Paragraph("2. Gatebox — Gateway PIX (Rust)", h3))
    story.append(_table([
        ["Aspecto", "Detalhe"],
        ["Stack", "Rust, Axum, SQLx, Redis, Prometheus, OpenAPI/Swagger"],
        ["Porta", ":8081 · Postgres (dubai-cash)"],
    ], [3.5 * cm, 12.5 * cm]))
    story.append(Spacer(1, 0.15 * cm))
    for item in [
        "API PIX: entrada/saída, QR EMV, reversões",
        "Gestão de contas, saldos, extratos e taxas por cliente/parceiro",
        "Reserva de segurança (MED): percentual bloqueado por 90 dias",
        "Idempotência e hierarquia de fees",
        "Integração Sulcred/SevenTrust via simuladores Rust",
        "Dashboard admin (front-gatebox — React + Material UI)",
    ]:
        story.append(Paragraph(f"• {item}", bullet))

    story.append(Paragraph("3. Site — Marketplace web (React)", h3))
    story.append(_table([
        ["Aspecto", "Detalhe"],
        ["Stack", "React 18, Vite, TypeScript, Tailwind, shadcn/ui, TanStack Query"],
        ["Idiomas", "PT, EN, ES (i18next)"],
        ["Integração", "API APICash (:3000) com JWT e refresh automático"],
    ], [3.5 * cm, 12.5 * cm]))
    story.append(Spacer(1, 0.15 * cm))
    for item in [
        "Fluxo comprador: pedidos, PIX, carteira, confirmação, disputas",
        "Fluxo vendedor: dashboard, propostas, disputas, carteira",
        "Cadastro PF/PJ (CPF/CNPJ) e rastreamento de encomendas",
    ]:
        story.append(Paragraph(f"• {item}", bullet))

    story.append(PageBreak())

    story.append(Paragraph("4. Banco Saczuck — Simulador bancário externo", h3))
    story.append(
        Paragraph(
            "<b>Stack:</b> Flutter (app mobile) + Go (API). Integração exclusivamente via API Gatebox, "
            "sem acesso ao banco interno. Simula instituição financeira parceira para testes PIX end-to-end.",
            body,
        )
    )

    story.append(Paragraph("5. Serviços auxiliares", h2))
    story.append(_table([
        ["Serviço", "Stack", "Função"],
        ["wallet/", "Go, Echo, pgx, OpenTelemetry", "Carteira digital"],
        ["apprastreio/", "Rust (Axum) + Flutter", "Simulador rastreio logístico (Correios)"],
        ["scraper-service/", "Python Playwright + Bun/TS", "Extração produtos TikTok Shop"],
        ["holdfy-admin/", "React, Vite, MUI", "Painel administrativo Holdfy"],
        ["simulador_rust/", "Rust", "Simuladores parceiros PIX"],
    ], [3.2 * cm, 5.3 * cm, 7.5 * cm]))

    story.append(Spacer(1, 0.3 * cm))
    story.append(Paragraph("Infraestrutura compartilhada", h2))
    story.append(_table([
        ["Componente", "Tecnologia", "Uso"],
        ["Banco relacional", "PostgreSQL 16", "APICash, Gatebox, Banco"],
        ["Cache", "Redis 7", "Sessões, filas"],
        ["Mensageria", "NATS JetStream (Pulsar opcional)", "Eventos assíncronos"],
        ["Object storage", "MinIO", "Imagens de produtos"],
        ["NoSQL", "MongoDB 7", "Dados WhatsApp"],
        ["Containers", "Docker Compose", "Dev e staging local"],
    ], [4 * cm, 5.5 * cm, 6.5 * cm]))

    story.append(Spacer(1, 0.3 * cm))
    story.append(Paragraph("Maturidade do projeto", h2))
    story.append(_table([
        ["Área", "Estado"],
        ["APICash core (escrow, PIX, antifraude)", "~85% funcional"],
        ["Gatebox PIX gateway", "~85% funcional"],
        ["Site integrado ao backend", "Integrado"],
        ["WhatsApp bot", "~70%"],
        ["Stellar/Soroban testnet", "Operacional (tx reais)"],
        ["PF/PJ (CPF/CNPJ)", "Implementado"],
        ["Disputas Gatebox, logística real, testes auto.", "Pendente ou parcial"],
    ], [8 * cm, 8 * cm]))

    story.append(Spacer(1, 0.4 * cm))
    story.append(Paragraph("Síntese executiva", h2))
    story.append(
        Paragraph(
            "O <b>Holdfy</b> é uma plataforma de marketplace com pagamentos protegidos, construída "
            "com foco em <b>Rust</b> no backend (performance e segurança), <b>React</b> nos frontends "
            "web e <b>Flutter</b> nos apps mobile.",
            body,
        )
    )
    story.append(Paragraph("<b>Tecnologias centrais</b>", body))
    for item in [
        "Backend: Rust (Axum, Tokio, SQLx) — APICash e Gatebox",
        "Blockchain: Stellar + Soroban (token BRLx, escrow on-chain)",
        "Frontend web: React + Vite + TypeScript + Tailwind/shadcn",
        "Mobile: Flutter",
        "Auxiliares: Go (wallet, banco), Python/Playwright (scraping)",
        "Infra: Docker, PostgreSQL, Redis, NATS, MinIO, MongoDB",
    ]:
        story.append(Paragraph(f"• {item}", bullet))

    story.append(Spacer(1, 0.15 * cm))
    story.append(Paragraph("<b>Funcionalidades de negócio</b>", body))
    for item in [
        "Marketplace P2P — propostas, pedidos, confirmação de entrega",
        "Pagamentos PIX — gateway com taxas, MED e idempotência",
        "Custódia/escrow — proteção do comprador via blockchain",
        "Anti-fraude — score de risco com validação CPF/CNPJ",
        "WhatsApp — canal conversacional para todo o fluxo",
        "Logística — rastreamento de encomendas",
        "Importador — produtos de TikTok Shop e outras fontes",
    ]:
        story.append(Paragraph(f"• {item}", bullet))

    story.append(Spacer(1, 0.3 * cm))
    story.append(
        Paragraph(
            "<i>Documento gerado automaticamente a partir da análise do monorepo Holdfy. "
            "Referências: CLAUDE.md, PLANO_EXECUCAO.md, README.md.</i>",
            ParagraphStyle("Foot", parent=styles["Normal"], fontSize=8, textColor=GREY),
        )
    )

    doc.build(story)
    print(f"PDF gerado: {OUT}")


if __name__ == "__main__":
    build()
