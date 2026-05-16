/// Instituições com códigos COMPE (3 dígitos), comuns em TED/DOC e dados bancários.
class BrazilianBank {
  const BrazilianBank(this.code, this.name);

  /// Três dígitos (ex.: "001").
  final String code;
  final String name;

  String get label => '$code — $name';
}

/// Lista curada das principais instituições; use filtro na UI para busca.
const List<BrazilianBank> kBrazilianBanks = [
  BrazilianBank('001', 'Banco do Brasil'),
  BrazilianBank('003', 'Banco da Amazônia'),
  BrazilianBank('004', 'Banco do Nordeste'),
  BrazilianBank('007', 'BNDES'),
  BrazilianBank('012', 'Banco Inbursa'),
  BrazilianBank('021', 'Banestes'),
  BrazilianBank('024', 'Banco Bandepe'),
  BrazilianBank('025', 'Banco Alfa'),
  BrazilianBank('033', 'Banco Santander'),
  BrazilianBank('036', 'Banco Bradesco BBI'),
  BrazilianBank('037', 'Banco do Estado do Pará'),
  BrazilianBank('041', 'Banrisul'),
  BrazilianBank('047', 'Banco do Estado de Sergipe'),
  BrazilianBank('062', 'Hipercard'),
  BrazilianBank('063', 'Banco Bradescard'),
  BrazilianBank('065', 'Banco Andbank'),
  BrazilianBank('069', 'Crefisa'),
  BrazilianBank('070', 'BRB — Banco de Brasília'),
  BrazilianBank('074', 'Banco J. Safra'),
  BrazilianBank('077', 'Banco Inter'),
  BrazilianBank('082', 'Banco Topázio'),
  BrazilianBank('084', 'Uniprime Norte do Paraná'),
  BrazilianBank('085', 'Cooperativa Central Ailos'),
  BrazilianBank('089', 'Credisan'),
  BrazilianBank('091', 'Unicred Central'),
  BrazilianBank('097', 'Credisis Central'),
  BrazilianBank('099', 'Uniprime Central'),
  BrazilianBank('102', 'XP Investimentos'),
  BrazilianBank('104', 'Caixa Econômica Federal'),
  BrazilianBank('107', 'Banco Bocom BBM'),
  BrazilianBank('119', 'Banco Western Union'),
  BrazilianBank('120', 'Banco Rodobens'),
  BrazilianBank('121', 'Banco Agibank'),
  BrazilianBank('125', 'Plural SCM'),
  BrazilianBank('128', 'MS Bank'),
  BrazilianBank('129', 'UBS Brasil'),
  BrazilianBank('136', 'Unicred Cooperativas'),
  BrazilianBank('169', 'Banco Olé Consignado'),
  BrazilianBank('173', 'BRL Trust DTVM'),
  BrazilianBank('174', 'Pefisa'),
  BrazilianBank('177', 'Guide'),
  BrazilianBank('188', 'Ativa Investimentos'),
  BrazilianBank('197', 'Stone Pagamentos'),
  BrazilianBank('208', 'Banco BTG Pactual'),
  BrazilianBank('212', 'Banco Original'),
  BrazilianBank('213', 'Banco Arbi'),
  BrazilianBank('217', 'Banco John Deere'),
  BrazilianBank('218', 'Banco BS2'),
  BrazilianBank('222', 'Banco Credit Agricole'),
  BrazilianBank('224', 'Banco Fibra'),
  BrazilianBank('233', 'Banco Cifra'),
  BrazilianBank('237', 'Banco Bradesco'),
  BrazilianBank('243', 'Banco Master'),
  BrazilianBank('246', 'Banco ABC Brasil'),
  BrazilianBank('254', 'Paraná Banco'),
  BrazilianBank('260', 'Nu Pagamentos (Nubank)'),
  BrazilianBank('265', 'Banco Fator'),
  BrazilianBank('290', 'PagSeguro Internet'),
  BrazilianBank('318', 'Banco BMG'),
  BrazilianBank('320', 'China Construction Bank'),
  BrazilianBank('323', 'Mercado Pago'),
  BrazilianBank('330', 'Banco Bari'),
  BrazilianBank('335', 'Banco Digio'),
  BrazilianBank('336', 'Banco C6'),
  BrazilianBank('341', 'Itaú Unibanco'),
  BrazilianBank('348', 'Banco XP'),
  BrazilianBank('380', 'PicPay'),
  BrazilianBank('389', 'Banco Mercantil do Brasil'),
  BrazilianBank('422', 'Banco Safra'),
  BrazilianBank('623', 'Banco Pan'),
  BrazilianBank('633', 'Banco Rendimento'),
  BrazilianBank('637', 'Banco Sofisa'),
  BrazilianBank('643', 'Banco Pine'),
  BrazilianBank('707', 'Banco Daycoval'),
  BrazilianBank('712', 'Banco Ourinvest'),
  BrazilianBank('739', 'Banco Cetelem'),
  BrazilianBank('741', 'Banco Ribeirão Preto'),
  BrazilianBank('743', 'Banco Semear'),
  BrazilianBank('745', 'Banco Citibank'),
  BrazilianBank('746', 'Banco Modal'),
  BrazilianBank('748', 'Sicredi'),
  BrazilianBank('751', 'Scotiabank Brasil'),
  BrazilianBank('752', 'BNP Paribas Brasil'),
  BrazilianBank('755', 'Bank of America Merrill Lynch'),
  BrazilianBank('756', 'Sicoob'),
  BrazilianBank('757', 'Banco KEB Hana do Brasil'),
];

List<BrazilianBank> filterBrazilianBanks(String query) {
  final q = query.trim().toLowerCase();
  if (q.isEmpty) return kBrazilianBanks.take(15).toList();
  return kBrazilianBanks
      .where((b) => b.code.contains(q) || b.name.toLowerCase().contains(q))
      .take(25)
      .toList();
}
