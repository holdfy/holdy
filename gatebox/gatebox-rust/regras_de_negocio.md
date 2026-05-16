# 📋 REGRAS DE NEGÓCIO - GATEBOXGO

## 📌 Índice

1. [Visão Geral](#visão-geral)
2. [Estrutura do Banco de Dados](#estrutura-do-banco-de-dados)
3. [Regras de Transações PIX](#regras-de-transações-pix)
4. [Sistema de Taxas](#sistema-de-taxas)
5. [Sistema MED (Reserva de Segurança)](#sistema-med-reserva-de-segurança)
6. [Cálculo de Saldo](#cálculo-de-saldo)
7. [Validações e Regras de Negócio](#validações-e-regras-de-negócio)
8. [Reversals (Devolução PIX)](#reversals-devolução-pix)
9. [Webhooks e Callbacks](#webhooks-e-callbacks)
10. [Invoice (Cobrança QR Code)](#invoice-cobrança-qr-code)

---

## 🎯 Visão Geral

Este documento detalha todas as regras de negócio do sistema GateboxGo, baseadas no sistema original Node.js. O objetivo é garantir que todas as regras críticas estejam corretamente implementadas no sistema Go.

### Princípios Fundamentais

1. **Integridade Financeira**: Todas as transações devem manter o balanço contábil correto
2. **Idempotência**: Transações com mesmo `externalId` no mesmo dia não podem ser duplicadas
3. **Rastreabilidade**: Todas as taxas e comissões devem ser rastreáveis e auditáveis
4. **Reserva de Segurança**: MED (Margem de Execução Diferida) deve ser calculada e bloqueada
5. **Transparência**: Cliente deve ver exatamente quanto está sendo debitado e por quê

---

## 🗄️ Estrutura do Banco de Dados

### Tabelas Principais

#### `transaction`
Tabela central que armazena todas as transações financeiras.

**Campos Críticos:**
- `id` (BIGSERIAL): ID único da transação
- `account_id` (BIGINT): Conta que realizou/recebeu a transação
- `type_transaction_id` (BIGINT): 1=DEBIT, 2=CREDIT
- `sub_type_transaction_id` (BIGINT): 1=PIX, 2=DPIX, 3=TTO, 4=TPO, 5=SMD, etc.
- `status_transaction_id` (BIGINT): 1=NEW, 2=QUEUED, 3=AWAITING, 4=COMPLETED, 5=ERROR, 6=REFUNDED, 7=FAILED, 8=DROP, 9=CANCEL, 10=TESTING, 11=PROCESSING
- `amount` (NUMERIC(16,2)): Valor da transação
- `external_id` (VARCHAR(100)): ID externo para idempotência
- `parent_id` (BIGINT): ID da transação pai (para TTO/TPO)
- `endtoend_id` (VARCHAR(32)): ID end-to-end do PIX
- `partners_id` (BIGINT): ID do parceiro/gateway usado

**Campos de Taxas (adicionados para auditoria):**
- `requested_amount` (NUMERIC(16,2)): Valor solicitado pelo cliente
- `net_amount` (NUMERIC(16,2)): Valor líquido enviado/recebido
- `total_amount` (NUMERIC(16,2)): Valor total debitado (amount + taxas)
- `fee_fixed` (NUMERIC(16,2)): Taxa fixa aplicada
- `fee_percent_rate` (NUMERIC(16,4)): Taxa percentual (ex: 0.50 = 0,5%)
- `fee_percent_amount` (NUMERIC(16,2)): Valor em R$ da taxa percentual
- `fee_total` (NUMERIC(16,2)): Taxa total (fixed + percent)
- `ref_fee_fixed` (NUMERIC(16,2)): Comissão fixa de referência
- `ref_fee_percent_rate` (NUMERIC(16,4)): Comissão percentual de referência
- `ref_fee_percent_amount` (NUMERIC(16,2)): Valor da comissão percentual
- `ref_fee_total` (NUMERIC(16,2)): Comissão total
- `gateway_fee` (NUMERIC(16,2)): Taxa cobrada pelo gateway
- `platform_fee` (NUMERIC(16,2)): Receita líquida da plataforma
- `partner_fixed_cash_out` (NUMERIC(16,2)): Snapshot taxa fixa do parceiro
- `partner_percent_cashout` (NUMERIC(16,4)): Snapshot taxa % do parceiro
- `fees_id` (BIGINT): ID do fees usado (se customizado)
- `fees_calculated_at` (TIMESTAMP): Quando as taxas foram calculadas
- `fee_calculation_version` (VARCHAR(20)): Versão do algoritmo (ex: 'v1.0')

#### `fees`
Taxas customizadas por customer (sobrescreve taxas do parceiro).

**Campos:**
- `id` (BIGSERIAL): ID único
- `account_id` (BIGINT): Conta do customer
- `fixed_cash_in` (NUMERIC(16,2)): Taxa fixa PIX IN
- `fixed_cash_out` (NUMERIC(16,2)): Taxa fixa PIX OUT
- `percent_cashin` (NUMERIC(16,2)): Taxa percentual PIX IN
- `percent_cashout` (NUMERIC(16,2)): Taxa percentual PIX OUT
- `percentsec_med` (NUMERIC(16,2)): Taxa percentual MED (reserva de segurança)
- `fixed_ref_cash_in` (NUMERIC(16,2)): Comissão fixa referência IN
- `fixed_ref_cash_out` (NUMERIC(16,2)): Comissão fixa referência OUT
- `percent_ref_cashin` (NUMERIC(16,2)): Comissão percentual referência IN
- `percent_ref_cashout` (NUMERIC(16,2)): Comissão percentual referência OUT

**Regra:** Se não existir registro em `fees` para o customer, usa taxas do `partners`.

#### `partners`
Configuração de taxas dos gateways/parceiros.

**Campos de Taxas:**
- `fixed_cash_in` (NUMERIC(16,2)): Taxa fixa PIX IN padrão
- `fixed_cash_out` (NUMERIC(16,2)): Taxa fixa PIX OUT padrão
- `percent_cashin` (NUMERIC(16,2)): Taxa percentual PIX IN padrão
- `percent_cashout` (NUMERIC(16,2)): Taxa percentual PIX OUT padrão
- `fixed_ref_cash_in` (NUMERIC(16,2)): Comissão fixa referência IN
- `fixed_ref_cash_out` (NUMERIC(16,2)): Comissão fixa referência OUT
- `percent_ref_cashin` (NUMERIC(16,2)): Comissão percentual referência IN
- `percent_ref_cashout` (NUMERIC(16,2)): Comissão percentual referência OUT

#### `sec_med`
Reserva de segurança (MED) bloqueada por conta.

**Campos:**
- `id` (BIGSERIAL): ID único
- `account_id` (BIGINT): Conta que tem o MED bloqueado
- `transaction_id` (BIGINT): ID da transação que gerou o MED
- `invoice_id` (BIGINT): ID da invoice (se aplicável)
- `partners_id` (BIGINT): ID do parceiro
- `amount` (NUMERIC(16,2)): Valor do MED bloqueado
- `status_sec_med_id` (BIGINT): 1=OPEN (bloqueado), 2=RETURNED, 3=BLOCKED, 4=ACCEPTED
- `scheduled_date` (TIMESTAMP): Data prevista para liberação (hoje + 90 dias)

**Regra:** MED bloqueado (status OPEN) deve ser subtraído do saldo disponível.

#### `accounts`
Contas dos usuários.

**Campos Críticos:**
- `id` (BIGSERIAL): ID único
- `account_number` (VARCHAR(20)): Número da conta
- `branch` (VARCHAR(10)): Agência
- `account_status_id` (BIGINT): 1=ACTIVE, 2=INATIVE, 3=DELETED, 4=PENDING_KYC, 5=REJECTED_KYC
- `authentication_id` (BIGINT): ID do usuário

**Regras de Status:**
- Apenas contas com `account_status_id = 1` (ACTIVE) podem realizar transações
- Contas com `account_status_id = 4` (PENDING_KYC) são rejeitadas

#### `invoice`
Cobranças QR Code PIX.

**Campos:**
- `id` (BIGSERIAL): ID único
- `identifier` (VARCHAR(50)): ID único da invoice (txid)
- `key` (VARCHAR(77)): Código PIX Copia e Cola
- `invoice_type_id` (BIGINT): 1=DYNAMIC, 2=STATIC, 3=FIXED
- `invoice_status_id` (BIGINT): 1=CREATED, 2=DONE, 3=CANCEL
- `amount` (NUMERIC(16,2)): Valor da cobrança
- `account_id` (BIGINT): Conta que criou a invoice

**Regra:** Após pagamento, `invoice_status_id` deve ser atualizado para 2 (DONE), exceto para invoices tipo FIXED.

---

## 💸 Regras de Transações PIX

### PIX OUT (Envio de PIX)

#### Fluxo Completo

1. **Validação de Conta**
   - Conta deve existir e estar ativa (`account_status_id = 1`)
   - Rejeitar se `account_status_id = 4` (PENDING_KYC)
   - Rejeitar se `account_status_id != 1` (ACTIVE)

2. **Validação de Idempotência**
   - Se `externalId` fornecido, verificar duplicidade:
     - Mesmo `external_id`
     - Mesmo `account_id`
     - `status_transaction_id = 4` (COMPLETED)
     - Mesmo `amount`
     - `created_at` no mesmo dia (00:00:00 até 23:59:59)
   - Se encontrar duplicata, rejeitar com erro

3. **Cálculo de Taxas**
   ```
   fixed = fees.fixed_cash_out OU partners.fixed_cash_out (se fees não existir)
   percent = fees.percent_cashout OU partners.percent_cashout (se fees não existir)
   
   markup = 1 - (percent / 100)
   net_amount = requested_amount * markup - fixed
   fee_total = requested_amount - net_amount
   
   fee_percent_amount = requested_amount * (percent / 100)
   fee_fixed = fixed
   ```

4. **Validação de Saldo**
   ```
   balance = SUM(CREDIT) - SUM(DEBIT) 
            WHERE status_transaction_id IN (3, 4) 
            AND account_id = X
   
   med_blocked = SUM(amount) 
                 FROM sec_med 
                 WHERE account_id = X 
                 AND status_sec_med_id = 1 (OPEN)
   
   available_balance = balance - med_blocked
   
   total_required = requested_amount + fee_total
   
   Se available_balance < total_required:
       REJEITAR: "Insufficient balance"
   ```

5. **Criação da Transação Principal**
   - `type_transaction_id = 1` (DEBIT)
   - `sub_type_transaction_id = 1` (PIX)
   - `status_transaction_id = 1` (NEW)
   - `amount = requested_amount + fee_total` (valor total debitado)
   - Preencher todos os campos de taxas para auditoria
   - `gateway`: nome do gateway usado
   - `pix_operation_type`: "pix_out_key" ou "pix_out_qrcode"

6. **Criação de Transações TTO (Taxa Operacional)**
   - **DEBIT na conta do customer:**
     - `type_transaction_id = 1` (DEBIT)
     - `sub_type_transaction_id = 3` (TTO)
     - `status_transaction_id = 3` (AWAITING)
     - `amount = fee_total`
     - `parent_id = transaction.id` (ID da transação principal)
     - `description = "Debit for operational transaction fee pixOut."`
   
   - **CREDIT na conta do admin (account_id = 1):**
     - `type_transaction_id = 2` (CREDIT)
     - `sub_type_transaction_id = 3` (TTO)
     - `status_transaction_id = 3` (AWAITING)
     - `amount = fee_total`
     - `parent_id = transaction.id` (ID da transação principal)
     - `description = "Credit for operational transaction fee pixOut."`

7. **Criação de Transação TPO (Taxa Parceiro) - Se Aplicável**
   - Se `partners.fixed_cash_out > 0` OU `partners.percent_cashout > 0`:
     ```
     fixed_partner = partners.fixed_cash_out
     percent_partner = partners.percent_cashout
     
     markup_partner = 1 - (percent_partner / 100)
     calc_partner = requested_amount * markup_partner - fixed_partner
     rate_partner = requested_amount - calc_partner
     
     Se rate_partner < 0:
         rate_partner = 0
     ```
   
   - **DEBIT na conta do admin:**
     - `type_transaction_id = 1` (DEBIT)
     - `sub_type_transaction_id = 4` (TPO)
     - `status_transaction_id = 3` (AWAITING)
     - `amount = rate_partner`
     - `partners_id = partners.id`
     - `parent_id = transaction_tto_credit.id` (ID do CREDIT TTO)
     - `description = "Debit for operational Parteners rate pixOut"`

8. **Publicação na Fila**
   - Publicar mensagem na fila (Pulsar ou RabbitMQ) para processamento pelo worker
   - Incluir `transaction_id` e `amount` na mensagem

### PIX IN (Recebimento de PIX)

#### Fluxo Completo (Webhook)

1. **Identificação da Conta**
   - Buscar conta pela chave PIX recebida
   - Validar que conta existe e está ativa

2. **Validação de Idempotência**
   - Verificar se `endtoend_id` já foi processado
   - Se já processado, retornar sucesso (idempotência)

3. **Cálculo de Taxas PIX IN**
   ```
   fixed = fees.fixed_cash_in OU partners.fixed_cash_in (se fees não existir)
   percent = fees.percent_cashin OU partners.percent_cashin (se fees não existir)
   
   markup = 1 - (percent / 100)
   net_amount = received_amount * markup - fixed
   fee_total = received_amount - net_amount
   ```

4. **Criação da Transação CREDIT Principal**
   - `type_transaction_id = 2` (CREDIT)
   - `sub_type_transaction_id = 1` (PIX)
   - `status_transaction_id = 4` (COMPLETED)
   - `amount = received_amount` (valor bruto recebido)
   - Preencher campos de taxas
   - `external_id = endtoend_id` (para idempotência)

5. **Criação de Transações TTO (Taxa Operacional)**
   - Se `fee_total > 0`:
     - **DEBIT na conta do customer:**
       - `type_transaction_id = 1` (DEBIT)
       - `sub_type_transaction_id = 3` (TTO)
       - `status_transaction_id = 4` (COMPLETED)
       - `amount = fee_total`
       - `parent_id = transaction_credit.id`
       - `description = "Debit for operational transaction fee."`
     
     - **CREDIT na conta do admin:**
       - `type_transaction_id = 2` (CREDIT)
       - `sub_type_transaction_id = 3` (TTO)
       - `status_transaction_id = 4` (COMPLETED)
       - `amount = fee_total`
       - `parent_id = transaction_tto_debit.id`
       - `description = "Credit for operational transaction fee."`

6. **Criação de Transação TPO (Taxa Parceiro) - Se Aplicável**
   - Se `partners.fixed_cash_in > 0` OU `partners.percent_cashin > 0`:
     ```
     markup_partner = 1 - (partners.percent_cashin / 100)
     calc_partner = received_amount * markup_partner - partners.fixed_cash_in
     rate_partner = received_amount - calc_partner
     
     Se rate_partner < 0:
         rate_partner = 0
     ```
   
   - **DEBIT na conta do admin:**
     - `type_transaction_id = 1` (DEBIT)
     - `sub_type_transaction_id = 4` (TPO)
     - `status_transaction_id = 4` (COMPLETED)
     - `amount = rate_partner`
     - `partners_id = partners.id`
     - `parent_id = transaction_tto_credit.id`
     - `description = "Debit for operational Parteners rate"`

7. **Cálculo e Criação de MED (Reserva de Segurança)**
   - Se `fees.percentsec_med > 0`:
     ```
     percent_med = fees.percentsec_med
     markup_med = 1 - (percent_med / 100)
     amount_med = received_amount * markup_med
     rate_med = received_amount - amount_med
     ```
   
   - **Criar Transação SMD (MED):**
     - `type_transaction_id = 1` (DEBIT)
     - `sub_type_transaction_id = 5` (SMD - MED SEGURANÇA OPERACIONAL)
     - `status_transaction_id = 4` (COMPLETED)
     - `amount = rate_med`
     - `parent_id = transaction_credit.id`
     - `description = "Debit for operational med security."`
   
   - **Criar Registro em `sec_med`:**
     - `account_id = customer_account_id`
     - `transaction_id = transaction_smd.id`
     - `invoice_id = invoice.id` (se aplicável)
     - `partners_id = partners.id`
     - `amount = rate_med`
     - `status_sec_med_id = 1` (OPEN - bloqueado)
     - `scheduled_date = hoje + 90 dias`

8. **Atualização de Invoice**
   - Se invoice vinculada e `invoice_type_id != 3` (FIXED):
     - Atualizar `invoice_status_id = 2` (DONE)

---

## 💰 Sistema de Taxas

### Hierarquia de Taxas

1. **Prioridade 1:** Taxas customizadas em `fees` (por customer)
2. **Prioridade 2:** Taxas padrão em `partners` (do gateway)

### Tipos de Taxas

#### Taxas do Customer (`fees`)
- `fixed_cash_in`: Taxa fixa PIX IN (ex: R$ 0,10)
- `fixed_cash_out`: Taxa fixa PIX OUT (ex: R$ 0,10)
- `percent_cashin`: Taxa percentual PIX IN (ex: 0.50 = 0,5%)
- `percent_cashout`: Taxa percentual PIX OUT (ex: 0.00 = 0%)
- `percentsec_med`: Taxa percentual MED (ex: 2.00 = 2%)
- `fixed_ref_cash_in`: Comissão fixa referência IN
- `fixed_ref_cash_out`: Comissão fixa referência OUT
- `percent_ref_cashin`: Comissão percentual referência IN
- `percent_ref_cashout`: Comissão percentual referência OUT

#### Taxas do Parceiro (`partners`)
- Mesmos campos, mas são valores padrão do gateway

### Fórmulas de Cálculo

#### PIX OUT
```
fixed = fees.fixed_cash_out OU partners.fixed_cash_out
percent = fees.percent_cashout OU partners.percent_cashout

markup = 1 - (percent / 100)
net_amount = requested_amount * markup - fixed
fee_total = requested_amount - net_amount

fee_percent_amount = requested_amount * (percent / 100)
fee_fixed = fixed
```

#### PIX IN
```
fixed = fees.fixed_cash_in OU partners.fixed_cash_in
percent = fees.percent_cashin OU partners.percent_cashin

markup = 1 - (percent / 100)
net_amount = received_amount * markup - fixed
fee_total = received_amount - net_amount
```

#### Taxa Parceiro (TPO)
```
fixed_partner = partners.fixed_cash_out (PIX OUT) OU partners.fixed_cash_in (PIX IN)
percent_partner = partners.percent_cashout (PIX OUT) OU partners.percent_cashin (PIX IN)

markup_partner = 1 - (percent_partner / 100)
calc_partner = amount * markup_partner - fixed_partner
rate_partner = amount - calc_partner

Se rate_partner < 0:
    rate_partner = 0
```

### Transações de Taxa

#### TTO (Tarifa Transferência Operacional)
- Taxa da plataforma
- DEBIT na conta do customer
- CREDIT na conta do admin (account_id = 1)
- `sub_type_transaction_id = 3`

#### TPO (Tarifa Parceiro Operacional)
- Comissão do parceiro/gateway
- DEBIT na conta do admin
- Vinculado ao `partners_id`
- `sub_type_transaction_id = 4`

---

## 🛡️ Sistema MED (Reserva de Segurança)

### Conceito

MED (Margem de Execução Diferida) é uma reserva de segurança que é bloqueada quando um PIX IN é recebido. O valor bloqueado fica indisponível por 90 dias para garantir segurança operacional.

### Regras

1. **Aplicação**
   - MED só é aplicado em PIX IN (recebimentos)
   - Apenas se `fees.percentsec_med > 0`

2. **Cálculo**
   ```
   percent_med = fees.percentsec_med
   markup_med = 1 - (percent_med / 100)
   amount_med = received_amount * markup_med
   rate_med = received_amount - amount_med
   ```

3. **Criação**
   - Criar transação DEBIT tipo SMD (`sub_type_transaction_id = 5`)
   - Criar registro em `sec_med` com:
     - `status_sec_med_id = 1` (OPEN - bloqueado)
     - `scheduled_date = hoje + 90 dias`

4. **Impacto no Saldo**
   - MED bloqueado (status OPEN) deve ser subtraído do saldo disponível
   - Saldo disponível = Saldo total - MED bloqueado

5. **Liberação**
   - Após 90 dias, MED pode ser liberado (status mudado para ACCEPTED)
   - Ou pode ser devolvido (status RETURNED) em caso de infração

### Exemplo

```
PIX IN recebido: R$ 100,00
percentsec_med: 2%

markup_med = 1 - (2 / 100) = 0.98
amount_med = 100 * 0.98 = R$ 98,00
rate_med = 100 - 98 = R$ 2,00

MED bloqueado: R$ 2,00
Saldo creditado: R$ 100,00
Saldo disponível: R$ 98,00 (100 - 2)
```

---

## 💵 Cálculo de Saldo

### Fórmula Completa

```sql
-- Saldo bruto (transações completadas)
balance = (
    SELECT COALESCE(SUM(
        CASE 
            WHEN type_transaction_id = 2 THEN amount  -- CREDIT
            WHEN type_transaction_id = 1 THEN -amount -- DEBIT
            ELSE 0
        END
    ), 0)
    FROM transaction
    WHERE account_id = X
    AND status_transaction_id IN (3, 4)  -- AWAITING, COMPLETED
)

-- MED bloqueado
med_blocked = (
    SELECT COALESCE(SUM(amount), 0)
    FROM sec_med
    WHERE account_id = X
    AND status_sec_med_id = 1  -- OPEN
)

-- Saldo disponível
available_balance = balance - med_blocked
```

### Status Considerados

**Incluídos no Saldo:**
- `status_transaction_id = 3` (AWAITING)
- `status_transaction_id = 4` (COMPLETED)

**Excluídos do Saldo:**
- `status_transaction_id = 1` (NEW)
- `status_transaction_id = 2` (QUEUED)
- `status_transaction_id = 5` (ERROR)
- `status_transaction_id = 6` (REFUNDED)
- `status_transaction_id = 7` (FAILED)
- `status_transaction_id = 8` (DROP)
- `status_transaction_id = 9` (CANCEL)
- `status_transaction_id = 10` (TESTING)
- `status_transaction_id = 11` (PROCESSING)

### Endpoint de Saldo

**Request:**
```
GET /api/v1/account/balance?full=true
```

**Response:**
```json
{
  "statusCode": 200,
  "balance": "1000.00",           // Saldo bruto
  "preventiveBlock": "20.00",     // MED bloqueado
  "availableBalance": "980.00"     // Saldo disponível (balance - preventiveBlock)
}
```

---

## ✅ Validações e Regras de Negócio

### Validação de Conta

1. **Status da Conta**
   - ✅ `account_status_id = 1` (ACTIVE): Permitido
   - ❌ `account_status_id = 4` (PENDING_KYC): Rejeitar com erro "your account is pending kyc"
   - ❌ `account_status_id != 1`: Rejeitar com erro "invalid account status"

2. **Regras de Conta (`account_rules`)**
   - `receive_external`: Se `false`, não pode receber PIX externos
   - `deposit_external`: Se `false`, não pode depositar externamente

### Validação de Idempotência

1. **PIX OUT com `externalId`**
   - Verificar duplicidade no mesmo dia:
     ```sql
     SELECT id FROM transaction
     WHERE external_id = $1
     AND account_id = $2
     AND status_transaction_id = 4  -- COMPLETED
     AND amount = $3
     AND created_at >= $4  -- início do dia
     AND created_at < $5   -- fim do dia
     ```
   - Se encontrar, rejeitar: "transaction {externalId} is duplicate"

2. **PIX IN com `endtoend_id`**
   - Verificar se `endtoend_id` já foi processado
   - Se já processado, retornar sucesso (idempotência)

### Validação de Saldo

1. **Antes de PIX OUT**
   ```
   total_required = requested_amount + fee_total
   
   Se available_balance < total_required:
       REJEITAR: "Insufficient balance, your balance is {balance} your rate {rate} your withdraw {amount}"
   ```

2. **Considerar MED Bloqueado**
   - Saldo disponível = Saldo bruto - MED bloqueado
   - MED bloqueado deve ser subtraído do saldo disponível

### Whitelist (`with_list_accounts`)

- Validar se conta está na whitelist para receber/depositar externamente
- Se não estiver e `account_rules` restringir, rejeitar

---

## 🔄 Reversals (Devolução PIX)

### Conceito

Reversal (DPIX) é a devolução de um PIX OUT que foi enviado. Permite devolver parte ou todo o valor de uma transação original.

### Regras

1. **Validação**
   - Transação original deve existir
   - `reversal_amount <= transaction.amount` (original)
   - Conta deve ter saldo suficiente (incluindo taxa de reversal)

2. **Cálculo de Taxas de Reversal**
   ```
   fixed = fees.fixed_ref_cashout OU partners.fixed_ref_cash_out
   percent = fees.percent_ref_cashout OU partners.percent_ref_cashout
   
   markup = 1 - (percent / 100)
   amount = reversal_amount * markup - fixed
   rate = reversal_amount - amount
   ```

3. **Criação da Transação**
   - `type_transaction_id = 1` (DEBIT)
   - `sub_type_transaction_id = 2` (DPIX)
   - `status_transaction_id = 1` (NEW)
   - `amount = reversal_amount`
   - `charger_back_id = transaction_original.endtoend_id`
   - `external_id`: ID externo (se fornecido)

4. **Criação de Transações TTO/TPO**
   - Mesma lógica do PIX OUT
   - Usar taxas de reversal (`fixed_ref_cashout`, `percent_ref_cashout`)

### Endpoint

**Request:**
```
POST /api/v1/pix/reversal
{
  "end2end": "E12345678901234567890123456789012",
  "amount": 50.00,
  "externalId": "rev-123"
}
```

---

## 📡 Webhooks e Callbacks

### Webhook PIX IN

1. **Processamento**
   - Receber webhook do gateway
   - Identificar conta pela chave PIX
   - Validar idempotência (`endtoend_id`)
   - Calcular taxas
   - Criar transações (CREDIT, TTO, TPO, MED)
   - Atualizar invoice (se aplicável)

2. **Idempotência**
   - Se `endtoend_id` já processado, retornar sucesso
   - Não criar transações duplicadas

### Webhook PIX OUT

1. **Processamento**
   - Receber status do gateway
   - Atualizar `status_transaction_id` da transação
   - Se falhou, atualizar `msg_error`
   - Notificar webhook do customer (se configurado)

2. **Status Possíveis**
   - `COMPLETED`: Pagamento concluído
   - `FAILED`: Pagamento falhou
   - `PROCESSING`: Em processamento
   - `PENDING`: Aguardando

---

## 🧾 Invoice (Cobrança QR Code)

### Regras

1. **Criação**
   - Criar invoice com `invoice_status_id = 1` (CREATED)
   - Gerar QR Code via gateway
   - Armazenar `identifier` (txid) e `key` (código PIX)

2. **Pagamento**
   - Quando PIX IN é recebido vinculado a uma invoice:
     - Atualizar `invoice_status_id = 2` (DONE)
     - **EXCEÇÃO:** Se `invoice_type_id = 3` (FIXED), não atualizar status

3. **Tipos de Invoice**
   - `DYNAMIC`: QR Code dinâmico (valor pode ser alterado)
   - `STATIC`: QR Code estático (valor fixo)
   - `FIXED`: QR Code fixo (não atualiza status após pagamento)

---

## 📊 Sub-tipos de Transação

| ID | Código | Descrição | Uso |
|---|---|---|---|
| 1 | PIX | Transação PIX normal | PIX IN/OUT principal |
| 2 | DPIX | Devolução PIX | Reversals |
| 3 | TTO | Tarifa Transferência Operacional | Taxa da plataforma |
| 4 | TPO | Tarifa Parceiro Operacional | Comissão do gateway |
| 5 | SMD | MED Segurança Operacional | Reserva de segurança |

---

## 🔍 Validações de Integridade

### Validações Automáticas

1. **Balanço Contábil**
   ```
   Para cada transação TTO:
   - DEBIT customer + CREDIT admin = 0 (soma deve ser zero)
   ```

2. **Taxas**
   ```
   fee_total = fee_fixed + fee_percent_amount
   total_amount = requested_amount + fee_total
   ```

3. **MED**
   ```
   Para cada MED criado:
   - Deve existir transação SMD correspondente
   - scheduled_date = hoje + 90 dias
   ```

---

## 📝 Notas Importantes

1. **Modelo de Taxas: Embutido vs Separado**
   - **Node.js:** Cria transações TTO/TPO separadas (filhas)
   - **Go (atual):** Armazena taxas na transação principal
   - **Decisão necessária:** Manter modelo embutido ou criar transações filhas?

2. **Performance**
   - Cálculo de saldo deve ser otimizado (índices em `account_id`, `status_transaction_id`)
   - Cache de saldo pode ser usado (TTL curto: 30 segundos)

3. **Auditoria**
   - Todos os campos de taxas devem ser preenchidos
   - `fees_calculated_at` e `fee_calculation_version` para rastreabilidade
   - Snapshots de taxas do parceiro para histórico

---

## 🎯 Checklist de Implementação

### Prioridade CRÍTICA
- [ ] Sistema MED completo
- [ ] Taxas PIX IN
- [ ] Transações TTO/TPO

### Prioridade ALTA
- [ ] Reversals (DPIX)
- [ ] Cálculo de saldo com MED
- [ ] Atualização de invoice

### Prioridade MÉDIA
- [ ] Validações adicionais
- [ ] Auditoria completa
- [ ] Webhooks completos

---

**Última atualização:** 2025-01-XX
**Versão:** 1.0

