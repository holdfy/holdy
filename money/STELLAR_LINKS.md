# Stellar Testnet — Contratos e Contas

## Smart Contracts (Soroban)

| Contrato | ID |
|---|---|
| **BRLx SAC** (token) | `CD6HDZ5SXDQXDEEVEFK3CBF3U46K7DPK6BTAZ2X3BWQB5DG7EFQ2KWML` |
| **Escrow** (lock/release) | `CDKH7DSK3BLY53MQBMVATSQ3O2LEJ5MGT7OPJYCTWNXNLO7IQ5TLQD4B` |

### Links stellar.expert

- BRLx SAC: https://stellar.expert/explorer/testnet/contract/CD6HDZ5SXDQXDEEVEFK3CBF3U46K7DPK6BTAZ2X3BWQB5DG7EFQ2KWML
- Escrow:   https://stellar.expert/explorer/testnet/contract/CDKH7DSK3BLY53MQBMVATSQ3O2LEJ5MGT7OPJYCTWNXNLO7IQ5TLQD4B

---

## Contas

| Conta | Endereço |
|---|---|
| **holdfy-deployer** | `GA7F43PVEWTY2SWGL5PIIUG2SPY4YXWZFFMCSSRA7BB4OCITZ5MZFRMS` |
| **holdfy-buyer**    | `GDNKUCMTGITERRALPY7UYKPQCX563AZGRJ5SOTZAFRHEZC64L4XAJG7R` |
| **holdfy-seller**   | `GBCDCBZYASK4UDQ37CGBUDPWEX27NF7JLGQLGNE5GIYDPIC775NHFVOY` |

### Links stellar.expert

- deployer: https://stellar.expert/explorer/testnet/account/GA7F43PVEWTY2SWGL5PIIUG2SPY4YXWZFFMCSSRA7BB4OCITZ5MZFRMS
- buyer:    https://stellar.expert/explorer/testnet/account/GDNKUCMTGITERRALPY7UYKPQCX563AZGRJ5SOTZAFRHEZC64L4XAJG7R
- seller:   https://stellar.expert/explorer/testnet/account/GBCDCBZYASK4UDQ37CGBUDPWEX27NF7JLGQLGNE5GIYDPIC775NHFVOY

---

## Horizon API (REST direto)

- Conta deployer: https://horizon-testnet.stellar.org/accounts/GA7F43PVEWTY2SWGL5PIIUG2SPY4YXWZFFMCSSRA7BB4OCITZ5MZFRMS
- Conta buyer:    https://horizon-testnet.stellar.org/accounts/GDNKUCMTGITERRALPY7UYKPQCX563AZGRJ5SOTZAFRHEZC64L4XAJG7R
- Conta seller:   https://horizon-testnet.stellar.org/accounts/GBCDCBZYASK4UDQ37CGBUDPWEX27NF7JLGQLGNE5GIYDPIC775NHFVOY

---

## Rede

- **Network**: testnet
- **Horizon**: https://horizon-testnet.stellar.org
- **Soroban RPC**: https://soroban-testnet.stellar.org
- **Passphrase**: `Test SDF Network ; September 2015`

---

## Notas

- No stellar.expert, no contrato de **Escrow**, aparecem as invocações `lock` e `release`
  com os parâmetros: `order_id`, `amount`, `buyer`, `seller`.
- O contrato **BRLx SAC** é o token fungível usado como representação do Real na Stellar.
- Todas as transações do fluxo PIX → BRLx → escrow → off-ramp ficam registradas aqui.
