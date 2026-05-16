# Referência: Fase 15.8 – Batches por período (Merkle)

Documento de referência para a implementação da fase posterior de ancoragem por período com raízes Merkle (plano integração Ethereum/Polygon, checklist 15.8).

---

## 1. Tabela `period_anchors`

**Arquivo:** [gateboxgo/database/anchor-blockchain-schema.sql](../gateboxgo/database/anchor-blockchain-schema.sql)

- `CREATE TABLE period_anchors` com: `id`, `period_type`, `period_id`, `merkle_root`, `tx_hash`, `block_number`, `chain_id`, `anchored_at`, `created_at`.
- UNIQUE (`period_type`, `period_id`).
- Executar o script no mesmo schema onde está `transaction_anchors`.

---

## 2. Contrato Solidity e ABI

| Item | Caminho |
|------|---------|
| Contrato | [gateboxgo/contracts/AnchorRoots.sol](../gateboxgo/contracts/AnchorRoots.sol) |
| ABI | [gateboxgo/contracts/abi/AnchorRoots.json](../gateboxgo/contracts/abi/AnchorRoots.json) |
| Documentação | [gateboxgo/contracts/README.md](../gateboxgo/contracts/README.md) |

- **Funções:** `setRoot(string periodType, string periodId, bytes32 root)`, `getRoot(string periodType, string periodId) returns (bytes32)`.
- **Variável de ambiente:** `ANCHOR_CONTRACT_ADDRESS` (por rede: testnet/mainnet).
- Deploy: exemplo com Foundry em `contracts/README.md`.

---

## 3. Go – API e Merkle

| Item | Caminho |
|------|---------|
| Merkle (raiz + proof) | [gateboxgo/internal/anchor/merkle.go](../gateboxgo/internal/anchor/merkle.go) |
| Repository (period_anchors, ListByPeriod) | [gateboxgo/app/anchor/repository/anchor_repository.go](../gateboxgo/app/anchor/repository/anchor_repository.go) |
| Service GetMerkleProof | [gateboxgo/app/anchor/service/anchor_service.go](../gateboxgo/app/anchor/service/anchor_service.go) |
| Handler e rota | [gateboxgo/app/anchor/handler/anchor_handler.go](../gateboxgo/app/anchor/handler/anchor_handler.go), [register.go](../gateboxgo/app/anchor/handler/register.go) |

**Endpoint:** `GET /anchor/proof/:entity_type/:entity_id/merkle?period_type=<>&period_id=<>`  
- Exemplo: `GET /api/v1/anchor/proof/pix_tx/123/merkle?period_type=day&period_id=2025-03-01`  
- Retorna: `entity_type`, `entity_id`, `payload_hash`, `period_type`, `period_id`, `merkle_root`, `proof` (array de hashes irmãos em hex), `leaf_index`, `period_tx_hash`, `block_number`.  
- Protegido por admin/backoffice (mesmo grupo das outras rotas `/anchor`).  
- A prova é calculada mesmo que o período ainda não tenha `period_anchors` (nesse caso `period_tx_hash` e `block_number` vêm null).

---

## 4. Job period-closer (removido)

O componente `period-closer` em Rust (antes hospedado em `ethereum_poc/polygon-rust`) foi removido deste repositório.
Se essa etapa voltar a ser necessária, a implementação deve ser reintroduzida em um módulo próprio com documentação atualizada.

---

## 5. Plano e checklist

- Plano principal: [.cursor/plans/integração_ethereum_polygon_rust_cfd9a13a.plan.md](../.cursor/plans/integração_ethereum_polygon_rust_cfd9a13a.plan.md)  
- Seção **15.8** do checklist está marcada como concluída, com referências aos arquivos acima.

---

## 6. Ordem sugerida para uso

1. Aplicar o script SQL (inclui `period_anchors`).
2. Deploy do contrato AnchorRoots; preencher `ANCHOR_CONTRACT_ADDRESS` no README e na config do job (quando for usar on-chain).
3. Agendar o **period-closer** (ex.: uma vez por dia ou por hora).
4. Consultar Merkle proof pela API: `GET .../anchor/proof/:entity_type/:entity_id/merkle?period_type=&period_id=`.

---

## 7. O que falta (para referência futura)

| Item | Descrição |
|------|-----------|
| **Chamada on-chain no period-closer** | O job hoje só grava `merkle_root` em `period_anchors`; não envia tx para o contrato. Falta implementar no Rust a chamada `setRoot(periodType, periodId, root)` usando ABI encoding (ex.: `alloy_sol_types`) e `ANCHOR_CONTRACT_ADDRESS`. |
| **Verificação da prova** | Documentar como verificar a Merkle proof: começar com o `payload_hash` da folha (32 bytes), para cada elemento de `proof` fazer `SHA256(concat(current, sibling))` ou `SHA256(concat(sibling, current))` conforme a ordem na árvore, até obter a raiz e comparar com `merkle_root`. |
| **Swagger** | As rotas `/anchor` (audit, proof, merkle) não têm anotações Swagger no handler; se o projeto documentar outras rotas com Swagger, incluir esses endpoints. |
| **Testes** | Não há testes unitários para `internal/anchor/merkle.go` (MerkleTree, ProofForIndex) nem para o service `GetMerkleProof` (conforme plano “nível profissional”). |
| **Front** | Se existir front de transação/MED: exibir link ou seção para “Prova Merkle do período” (ex.: link para a API de proof com `period_type` e `period_id` do evento). |
| **Script de deploy do contrato** | Há exemplo com `forge create` no README do contrato; um script (ex.: `contracts/deploy.sh`) com rede e chave por env facilita o deploy. |

---

*Última atualização: referente à implementação da fase 15.8 (Batches por período / Merkle).*
