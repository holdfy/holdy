#!/usr/bin/env python3
"""Gera PDF: papel da Stellar no projeto Holdfy / APICash (com diagramas)."""

from datetime import date
from pathlib import Path

from reportlab.lib import colors
from reportlab.lib.enums import TA_CENTER, TA_JUSTIFY
from reportlab.lib.pagesizes import A4
from reportlab.lib.styles import ParagraphStyle, getSampleStyleSheet
from reportlab.lib.units import cm
from reportlab.graphics.shapes import Drawing, Line, Polygon, Rect, String
from reportlab.platypus import (
    PageBreak,
    Paragraph,
    SimpleDocTemplate,
    Spacer,
    Table,
    TableStyle,
)

OUT = Path(__file__).resolve().parent / "stellar-papel-no-projeto-holdfy.pdf"

BLUE = colors.HexColor("#2c5282")
LIGHT = colors.HexColor("#bee3f8")
NOTE_BG = colors.HexColor("#fffff0")
GREY = colors.grey


def _arrow(d: Drawing, x1: float, y1: float, x2: float, y2: float, dashed: bool = False) -> None:
    dash = [3, 2] if dashed else None
    d.add(Line(x1, y1, x2, y2, strokeColor=BLUE, strokeWidth=1, strokeDashArray=dash))
    # ponta da seta
    if abs(x2 - x1) < 1:
        return
    direction = 1 if x2 > x1 else -1
    d.add(
        Polygon(
            [x2, y2, x2 - 4 * direction, y2 + 3, x2 - 4 * direction, y2 - 3],
            fillColor=BLUE,
            strokeColor=BLUE,
        )
    )


def _msg_label(d: Drawing, x1: float, x2: float, y: float, text: str) -> None:
    mid = (x1 + x2) / 2
    d.add(
        String(
            mid,
            y + 4,
            text,
            fontSize=6.5,
            textAnchor="middle",
            fillColor=colors.black,
        )
    )


def make_sequence_diagram() -> Drawing:
    """Diagrama de sequência: PIX → BRLx → escrow → release → off-ramp."""
    w, h = 17 * cm, 13.5 * cm
    d = Drawing(w, h)

    names = [
        "Comprador\n(PIX)",
        "Gatebox /\nAnchor fiat",
        "APICash\n(core)",
        "Stellar",
        "Soroban\nescrow",
    ]
    xs = [1.6 * cm, 4.8 * cm, 8.2 * cm, 11.6 * cm, 14.8 * cm]
    box_top = h - 1.8 * cm
    lifeline_bottom = 1.0 * cm

    for name, x in zip(names, xs):
        d.add(
            Rect(
                x - 1.1 * cm,
                box_top - 1.0 * cm,
                2.2 * cm,
                1.0 * cm,
                fillColor=LIGHT,
                strokeColor=BLUE,
                strokeWidth=1,
            )
        )
        for i, line in enumerate(name.split("\n")):
            d.add(
                String(
                    x,
                    box_top - 0.35 * cm - i * 11,
                    line,
                    fontSize=7,
                    textAnchor="middle",
                    fillColor=BLUE,
                )
            )
        d.add(
            Line(
                x,
                box_top - 1.0 * cm,
                x,
                lifeline_bottom,
                strokeColor=GREY,
                strokeWidth=0.5,
                strokeDashArray=[4, 3],
            )
        )

    y = box_top - 1.5 * cm
    step = 0.95 * cm
    messages = [
        (0, 1, "1. Paga PIX (on-ramp)"),
        (1, 2, "2. Funding confirmado"),
        (2, 3, "3. Emite / transfere BRLx"),
        (2, 4, "4. lock_funds"),
    ]
    for frm, to, label in messages:
        _arrow(d, xs[frm], y, xs[to], y)
        _msg_label(d, xs[frm], xs[to], y, label)
        y -= step

    # nota: custódia
    note_y = y - 0.15 * cm
    d.add(
        Rect(
            3.5 * cm,
            note_y - 0.55 * cm,
            10 * cm,
            0.5 * cm,
            fillColor=NOTE_BG,
            strokeColor=GREY,
        )
    )
    d.add(
        String(
            8.5 * cm,
            note_y - 0.35 * cm,
            "Pedido InCustody — valor protegido (DB + opcional on-chain)",
            fontSize=6.5,
            textAnchor="middle",
        )
    )
    y = note_y - step

    messages2 = [
        (0, 2, "5. Confirma recebimento"),
        (2, 4, "6. confirm_release"),
        (2, 3, "7. Principal + yield (70/10/20)"),
        (2, 1, "8. Off-ramp BRLx → PIX vendedor"),
    ]
    for frm, to, label in messages2:
        _arrow(d, xs[frm], y, xs[to], y)
        _msg_label(d, xs[frm], xs[to], y, label)
        y -= step

    d.add(
        String(
            0.2 * cm,
            h - 0.35 * cm,
            "Figura 1 — Fluxo de sequência APICash + Stellar/Soroban",
            fontSize=8,
            fillColor=GREY,
        )
    )
    return d


def make_architecture_diagram() -> Drawing:
    """Camadas do monorepo: quem usa Stellar."""
    w, h = 17 * cm, 8 * cm
    d = Drawing(w, h)

    def box(x, y, bw, bh, title, subtitle, fill, stellar: bool):
        d.add(Rect(x, y, bw, bh, fillColor=fill, strokeColor=BLUE, strokeWidth=1))
        d.add(String(x + bw / 2, y + bh - 14, title, fontSize=8, textAnchor="middle", fillColor=BLUE))
        d.add(
            String(
                x + bw / 2,
                y + bh - 28,
                subtitle,
                fontSize=6.5,
                textAnchor="middle",
                fillColor=colors.black,
            )
        )
        if stellar:
            d.add(
                String(
                    x + bw / 2,
                    y + 6,
                    "★ Stellar / Soroban",
                    fontSize=6,
                    textAnchor="middle",
                    fillColor=colors.HexColor("#c05621"),
                )
            )

    y0 = 4.2 * cm
    box(0.3 * cm, y0, 5 * cm, 2.2 * cm, "Gatebox", "PIX gateway (Postgres)", colors.HexColor("#e2e8f0"), False)
    box(6 * cm, y0, 5 * cm, 2.2 * cm, "Banco simulador", "API Gatebox apenas", colors.HexColor("#e2e8f0"), False)
    box(
        11.7 * cm,
        y0,
        5 * cm,
        2.2 * cm,
        "Admin Gatebox",
        "Auditoria Polygon",
        colors.HexColor("#e2e8f0"),
        False,
    )

    box(
        2.5 * cm,
        0.8 * cm,
        12 * cm,
        2.8 * cm,
        "APICash",
        "anchor · custody · core · WhatsApp → API",
        colors.HexColor("#ebf8ff"),
        True,
    )

    _arrow(d, 8.5 * cm, 3.6 * cm, 8.5 * cm, 3.15 * cm)
    d.add(
        String(
            8.5 * cm,
            3.35 * cm,
            "orders / settle / release",
            fontSize=6,
            textAnchor="middle",
        )
    )

    d.add(
        String(
            0.2 * cm,
            h - 0.35 * cm,
            "Figura 2 — Stellar só no stack APICash (não no Gatebox nem no banco simulador)",
            fontSize=8,
            fillColor=GREY,
        )
    )
    return d


def build():
    doc = SimpleDocTemplate(
        str(OUT),
        pagesize=A4,
        rightMargin=2 * cm,
        leftMargin=2 * cm,
        topMargin=2 * cm,
        bottomMargin=2 * cm,
        title="Stellar no projeto Holdfy",
        author="Holdfy / APICash",
    )
    styles = getSampleStyleSheet()
    title_style = ParagraphStyle(
        "CustomTitle",
        parent=styles["Heading1"],
        fontSize=20,
        spaceAfter=12,
        alignment=TA_CENTER,
        textColor=colors.HexColor("#1a365d"),
    )
    h2 = ParagraphStyle(
        "H2",
        parent=styles["Heading2"],
        fontSize=14,
        spaceBefore=14,
        spaceAfter=8,
        textColor=BLUE,
    )
    h3 = ParagraphStyle(
        "H3",
        parent=styles["Heading3"],
        fontSize=11,
        spaceBefore=10,
        spaceAfter=6,
    )
    body = ParagraphStyle(
        "Body",
        parent=styles["Normal"],
        fontSize=10,
        leading=14,
        alignment=TA_JUSTIFY,
        spaceAfter=8,
    )
    bullet = ParagraphStyle(
        "Bullet",
        parent=body,
        leftIndent=14,
        bulletIndent=6,
    )
    mono = ParagraphStyle(
        "Mono",
        parent=styles["Code"],
        fontSize=8,
        leading=11,
        backColor=colors.HexColor("#f7fafc"),
    )
    caption = ParagraphStyle(
        "Caption",
        parent=styles["Normal"],
        fontSize=9,
        alignment=TA_CENTER,
        textColor=GREY,
        spaceAfter=10,
    )

    story = []
    story.append(Paragraph("Stellar no projeto Holdfy", title_style))
    story.append(
        Paragraph(
            f"Documento de referência — gerado em {date.today().strftime('%d/%m/%Y')}",
            ParagraphStyle("Sub", parent=styles["Normal"], fontSize=9, alignment=TA_CENTER),
        )
    )
    story.append(Spacer(1, 0.4 * cm))

    story.append(Paragraph("Resumo executivo", h2))
    story.append(
        Paragraph(
            "No monorepo Holdfy, a <b>Stellar</b> aparece exclusivamente no stack "
            "<b>APICash</b> (<i>money/apicash</i>). Gatebox, o simulador de banco (app_banco) "
            "e a auditoria blockchain do admin (Polygon) <b>não</b> usam Stellar.",
            body,
        )
    )
    story.append(
        Paragraph(
            "Em uma frase: <b>PIX é o trilho fiat; Stellar/Soroban é o trilho de custódia "
            "e liquidação tokenizada do escrow</b> (ativo BRLx na rede).",
            body,
        )
    )

    story.append(Paragraph("Diagrama de arquitetura", h2))
    story.append(
        Paragraph(
            "Visão de quais componentes do monorepo dependem da rede Stellar.",
            body,
        )
    )
    story.append(make_architecture_diagram())
    story.append(Spacer(1, 0.3 * cm))

    story.append(Paragraph("Diagrama de sequência", h2))
    story.append(
        Paragraph(
            "Equivalente ao diagrama Mermaid da documentação: do pagamento PIX até "
            "a liberação e o off-ramp ao vendedor.",
            body,
        )
    )
    story.append(make_sequence_diagram())
    story.append(
        Paragraph(
            "<i>Participantes: comprador (PIX), Gatebox/anchor fiat, APICash core, "
            "rede Stellar (BRLx) e contrato Soroban (escrow).</i>",
            caption,
        )
    )

    story.append(Paragraph("Onde entra e onde não entra", h2))
    table_data = [
        ["Área", "Stellar?", "Função"],
        ["APICash (anchor, custody, soroban-contracts)", "Sim", "BRLx, on/off-ramp, escrow, yield on-chain"],
        ["Gatebox (gateway PIX)", "Não", "PIX em Postgres; sem ledger Stellar"],
        ["Banco simulador (Flutter + Go)", "Não", "Integra só API externa do Gatebox"],
        ["WhatsApp HoldFy", "Indireto", "Chama apicash-core (orders, custódia, PIX)"],
        ["x402 (micropagamentos)", "Não", "Base Sepolia (Ethereum), protocolo à parte"],
    ]
    t = Table(table_data, colWidths=[5.5 * cm, 2 * cm, 7.5 * cm])
    t.setStyle(
        TableStyle(
            [
                ("BACKGROUND", (0, 0), (-1, 0), BLUE),
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
    story.append(t)
    story.append(Spacer(1, 0.3 * cm))

    story.append(Paragraph("Fluxo de negócio (passos)", h2))
    for step in [
        "<b>1. Criar pedido</b> — antifraude → anchor gera instrução PIX (on-ramp) → status pending_funding.",
        "<b>2. PIX pago</b> — settle confirma funding → transfere BRLx para escrow → lock_funds (DB + Soroban opcional).",
        "<b>3. Comprador confirma entrega</b> — custody/release → contrato libera → off-ramp PIX ao vendedor.",
    ]:
        story.append(Paragraph(f"• {step}", bullet))

    story.append(PageBreak())

    story.append(Paragraph("Componentes Rust e responsabilidades", h2))

    story.append(Paragraph("apicash-anchor — ponte fiat ↔ Stellar", h3))
    for item in [
        "On-ramp: PIX → BRLx (token na Stellar; APICASH_STELLAR_ASSET_CODE, padrão BRLx).",
        "Off-ramp: BRLx → PIX ao vendedor.",
        "Horizon (APICASH_STELLAR_HORIZON_URL) para consultar transações.",
        "Dev: APICASH_FIAT_RAIL=simulated sem rede real.",
    ]:
        story.append(Paragraph(f"• {item}", bullet))

    story.append(Paragraph("apicash-custody — escrow e yield", h3))
    for item in [
        "Trava principal, calcula rendimento, libera ao vendedor.",
        "Split do yield: 70% vendedor / 10% comprador / 20% plataforma.",
        "Com APICASH_SOROBAN_ENABLED=1: invoca contrato via CLI stellar (lock/release).",
        "Sem Soroban: modo mock (hashes mock_*) — comum em desenvolvimento local.",
    ]:
        story.append(Paragraph(f"• {item}", bullet))

    story.append(Paragraph("soroban-contracts — smart contracts", h3))
    story.append(
        Paragraph(
            "Contratos Soroban: lock_funds, confirm_release, mark_disputed, split de yield. "
            "Wasm: target/wasm32v1-none/release/apicash_soroban_contracts.wasm. "
            "Deploy via scripts/soroban-testnet-deploy.sh.",
            body,
        )
    )

    story.append(Paragraph("apicash-core — orquestração", h3))
    story.append(
        Paragraph(
            "Persiste por pedido: anchor_tx_hash, brlx_escrow_transfer_tx_hash, "
            "soroban_escrow_contract_id, soroban_lock_tx_hash, soroban_mode (mock | soroban | pending_funding).",
            body,
        )
    )

    story.append(Paragraph("Rede e configuração", h2))
    story.append(
        Paragraph(
            "Por padrão usa <b>Stellar testnet pública</b> (não é nó local):",
            body,
        )
    )
    story.append(Paragraph("• Horizon: https://horizon-testnet.stellar.org", mono))
    story.append(Paragraph("• Soroban RPC: https://soroban-testnet.stellar.org", mono))
    story.append(Spacer(1, 0.2 * cm))
    story.append(
        Paragraph(
            "Variáveis em money/.env.example: APICASH_SOROBAN_*, APICASH_STELLAR_*, "
            "emissor BRLx, contrato de escrow. Com APICASH_REQUIRE_TESTNET=1 a API rejeita hashes mock.",
            body,
        )
    )

    story.append(Paragraph("Por que Stellar (e não só Postgres)?", h2))
    for item in [
        "Custódia auditável — escrow com hash on-chain.",
        "Ativo programável (BRLx) — token SEP-41 / SAC na Soroban.",
        "Yield on-chain — rendimento com split automático no contrato.",
        "Padrão Anchor (SEP-24) — ramp fiat ↔ crypto na rede Stellar.",
    ]:
        story.append(Paragraph(f"• {item}", bullet))
    story.append(
        Paragraph(
            "O Gatebox continua sendo o PIX operacional; a Stellar é a camada de settlement "
            "tokenizado do APICash (ou simulada em dev).",
            body,
        )
    )

    story.append(Paragraph("Estado prático do projeto", h2))
    story.append(
        Paragraph(
            "Segundo PLANO_EXECUCAO.md: escrow/PIX/yield ~85% funcional; Soroban frequentemente "
            "em mock até Postgres + deploy testnet + APICASH_SOROBAN_ENABLED=1. Gatebox e banco "
            "simulador não dependem de Stellar.",
            body,
        )
    )

    story.append(Paragraph("Perguntas frequentes", h2))
    faq = [
        ("Stellar é obrigatória para o PIX do Gatebox?", "Não."),
        ("Stellar é o quê no HoldFy?", "Ledger + escrow tokenizado (BRLx) do APICash."),
        ("Soroban faz o quê?", "Contrato de lock/release e split de yield."),
        ("WhatsApp usa Stellar direto?", "Não — usa API APICash, que pode acionar anchor/custódia."),
        ("Blockchain do admin Gatebox?", "Polygon (âncora de auditoria), outro propósito."),
    ]
    faq_table = Table([["Pergunta", "Resposta"]] + list(faq), colWidths=[8 * cm, 7 * cm])
    faq_table.setStyle(
        TableStyle(
            [
                ("BACKGROUND", (0, 0), (-1, 0), BLUE),
                ("TEXTCOLOR", (0, 0), (-1, 0), colors.whitesmoke),
                ("FONTNAME", (0, 0), (-1, 0), "Helvetica-Bold"),
                ("FONTSIZE", (0, 0), (-1, -1), 9),
                ("GRID", (0, 0), (-1, -1), 0.5, colors.grey),
                ("VALIGN", (0, 0), (-1, -1), "TOP"),
                ("ROWBACKGROUNDS", (0, 1), (-1, -1), [colors.white, colors.HexColor("#edf2f7")]),
                ("LEFTPADDING", (0, 0), (-1, -1), 6),
                ("TOPPADDING", (0, 0), (-1, -1), 5),
                ("BOTTOMPADDING", (0, 0), (-1, -1), 5),
            ]
        )
    )
    story.append(faq_table)

    story.append(Spacer(1, 0.5 * cm))
    story.append(
        Paragraph(
            "<i>Referências: CLAUDE.md, money/.env.example, crates apicash-anchor, "
            "apicash-custody, soroban-contracts, apicash-core (order_handler). "
            "Diagramas gerados em vector via ReportLab (equivalentes ao Mermaid do chat).</i>",
            ParagraphStyle("Foot", parent=styles["Normal"], fontSize=8, textColor=colors.grey),
        )
    )

    doc.build(story)
    print(f"PDF gerado: {OUT}")


if __name__ == "__main__":
    build()
