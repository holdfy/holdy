

CREATE TABLE account_status_types (
    id                                BIGSERIAL PRIMARY KEY,
    code                              VARCHAR(50) UNIQUE NOT NULL,
    description                       VARCHAR(100) NOT NULL
);

CREATE TABLE account_types (
    id                                BIGSERIAL PRIMARY KEY,
    code                              VARCHAR(50) UNIQUE NOT NULL,
    description                       VARCHAR(100) NOT NULL
);

CREATE TABLE address_types (
    id                                BIGSERIAL PRIMARY KEY,
    code                              VARCHAR(50) UNIQUE NOT NULL,
    description                       VARCHAR(100) NOT NULL
);

CREATE TABLE customer_status_types (
    id                                BIGSERIAL PRIMARY KEY,
    code                              VARCHAR(50) UNIQUE NOT NULL,
    description                       VARCHAR(100) NOT NULL
);

CREATE TABLE invoice_status_types (
    id                                BIGSERIAL PRIMARY KEY,
    code                              VARCHAR(50) UNIQUE NOT NULL,
    description                       VARCHAR(100) NOT NULL
);

CREATE TABLE invoice_types (
    id                                BIGSERIAL PRIMARY KEY,
    code                              VARCHAR(50) UNIQUE NOT NULL,
    description                       VARCHAR(100) NOT NULL
);

CREATE TABLE pix_key_types (
    id                                BIGSERIAL PRIMARY KEY,
    code                              VARCHAR(50) UNIQUE NOT NULL,
    description                       VARCHAR(100) NOT NULL
);

CREATE TABLE type_person_types (
    id                                BIGSERIAL PRIMARY KEY,
    code                              VARCHAR(50) UNIQUE NOT NULL,
    description                       VARCHAR(100) NOT NULL
);

CREATE TABLE type_auth_types (
    id                                BIGSERIAL PRIMARY KEY,
    code                              VARCHAR(50) UNIQUE NOT NULL,
    description                       VARCHAR(100) NOT NULL
);

CREATE TABLE kyc_risk_types (
    id                                BIGSERIAL PRIMARY KEY,
    code                              VARCHAR(50) UNIQUE NOT NULL,
    description                       VARCHAR(100) NOT NULL
);

CREATE TABLE status_controle_med_types (
    id                                BIGSERIAL PRIMARY KEY,
    code                              VARCHAR(50) UNIQUE NOT NULL,
    description                       VARCHAR(100) NOT NULL
);

CREATE TABLE status_sec_med_types (
    id                                BIGSERIAL PRIMARY KEY,
    code                              VARCHAR(50) UNIQUE NOT NULL,
    description                       VARCHAR(100) NOT NULL
);

CREATE TABLE status_transaction_types (
    id                                BIGSERIAL PRIMARY KEY,
    code                              VARCHAR(50) UNIQUE NOT NULL,
    description                       VARCHAR(100) NOT NULL
);

CREATE TABLE styled_types (
    id                                BIGSERIAL PRIMARY KEY,
    code                              VARCHAR(50) UNIQUE NOT NULL,
    description                       VARCHAR(100) NOT NULL
);

CREATE TABLE sub_type_transaction_types (
    id                                BIGSERIAL PRIMARY KEY,
    code                              VARCHAR(50) UNIQUE NOT NULL,
    description                       VARCHAR(100) NOT NULL
);

CREATE TABLE type_authorize_types (
    id                                BIGSERIAL PRIMARY KEY,
    code                              VARCHAR(50) UNIQUE NOT NULL,
    description                       VARCHAR(100) NOT NULL
);

CREATE TABLE type_transaction_types (
    id                                BIGSERIAL PRIMARY KEY,
    code                              VARCHAR(50) UNIQUE NOT NULL,
    description                       VARCHAR(100) NOT NULL
);

CREATE TABLE webhook_types (
    id                                BIGSERIAL PRIMARY KEY,
    code                              VARCHAR(50) UNIQUE NOT NULL,
    description                       VARCHAR(100) NOT NULL
);

CREATE TABLE type_external_types (
    id                                BIGSERIAL PRIMARY KEY,
    code                              VARCHAR(50) UNIQUE NOT NULL,
    description                       VARCHAR(100) NOT NULL
);

CREATE TABLE authentication (
    id                                BIGSERIAL PRIMARY KEY,
    name                              VARCHAR(100) NOT NULL,
    username                          VARCHAR(50) NOT NULL,
    password                          VARCHAR(255) NOT NULL,
    type_auth_id                      BIGINT DEFAULT 1 NOT NULL,
    active                            BOOLEAN DEFAULT true NOT NULL,
    force_reset                       BOOLEAN DEFAULT true NOT NULL,
    created_at                        TIMESTAMP(3) without time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
    updated_at                        TIMESTAMP(3) without time zone,
    deleted_at                        TIMESTAMP(3) without time zone,
    CONSTRAINT fk_authentication_type_auth_id FOREIGN KEY (type_auth_id) REFERENCES type_auth_types(id) ON UPDATE CASCADE ON DELETE RESTRICT
);

CREATE TABLE accounts (
    id                                BIGSERIAL PRIMARY KEY,
    account_number                    VARCHAR(20) NOT NULL,
    branch                            VARCHAR(10) NOT NULL,
    account_type_id                   BIGINT NOT NULL,
    account_status_id                 BIGINT NOT NULL,
    deleted_at                        TIMESTAMP(3) without time zone,
    authentication_id                 BIGINT NOT NULL,
    CONSTRAINT fk_accounts_authentication_id FOREIGN KEY (authentication_id) REFERENCES authentication(id) ON UPDATE CASCADE ON DELETE RESTRICT,
    CONSTRAINT fk_accounts_account_type_id FOREIGN KEY (account_type_id) REFERENCES account_types(id) ON UPDATE CASCADE ON DELETE RESTRICT,
    CONSTRAINT fk_accounts_account_status_id FOREIGN KEY (account_status_id) REFERENCES account_status_types(id) ON UPDATE CASCADE ON DELETE RESTRICT
);

CREATE TABLE company (
    id                                BIGSERIAL PRIMARY KEY,
    full_name                         VARCHAR(150) NOT NULL,
    social_name                       VARCHAR(100) NOT NULL,
    type_person_id                    BIGINT NOT NULL,
    document_number                   VARCHAR(18) NOT NULL,
    birth_date                        VARCHAR(10),
    responsible_name                  VARCHAR(100) NOT NULL,
    phone_number                      VARCHAR(20) NOT NULL,
    email                             VARCHAR(120) NOT NULL,
    telegram_chat_id                  VARCHAR(20),
    domanin                           VARCHAR(100),
    customer_status_id                BIGINT NOT NULL,
    is_politically_exposed_person     BOOLEAN DEFAULT false NOT NULL,
    authentication_id                 BIGINT NOT NULL,
    deleted_at                        TIMESTAMP(3) without time zone,
    CONSTRAINT fk_company_authentication_id FOREIGN KEY (authentication_id) REFERENCES authentication(id) ON UPDATE CASCADE ON DELETE RESTRICT,
    CONSTRAINT fk_company_type_person_id FOREIGN KEY (type_person_id) REFERENCES type_person_types(id) ON UPDATE CASCADE ON DELETE RESTRICT,
    CONSTRAINT fk_company_customer_status_id FOREIGN KEY (customer_status_id) REFERENCES customer_status_types(id) ON UPDATE CASCADE ON DELETE RESTRICT
);

CREATE TABLE customer (
    id                                BIGSERIAL PRIMARY KEY,
    full_name                         VARCHAR(150) NOT NULL,
    social_name                       VARCHAR(100) NOT NULL,
    type_person_id                    BIGINT NOT NULL,
    document_number                   VARCHAR(18) NOT NULL,
    birth_date                        VARCHAR(10),
    company_id                        BIGINT,
    responsible_name                  VARCHAR(100) NOT NULL,
    phone_number                      VARCHAR(20) NOT NULL,
    email                             VARCHAR(120) NOT NULL,
    customer_status_id                BIGINT NOT NULL,
    is_politically_exposed_person     BOOLEAN DEFAULT false NOT NULL,
    authentication_id                 BIGINT NOT NULL,
    deleted_at                        TIMESTAMP(3) without time zone,
    CONSTRAINT fk_customer_company_id FOREIGN KEY (company_id) REFERENCES company(id) ON UPDATE CASCADE ON DELETE RESTRICT,
    CONSTRAINT fk_customer_authentication_id FOREIGN KEY (authentication_id) REFERENCES authentication(id) ON UPDATE CASCADE ON DELETE RESTRICT,
    CONSTRAINT fk_customer_type_person_id FOREIGN KEY (type_person_id) REFERENCES type_person_types(id) ON UPDATE CASCADE ON DELETE RESTRICT,
    CONSTRAINT fk_customer_customer_status_id FOREIGN KEY (customer_status_id) REFERENCES customer_status_types(id) ON UPDATE CASCADE ON DELETE RESTRICT
);

CREATE TABLE address (
    id                                BIGSERIAL PRIMARY KEY,
    postal_code                       VARCHAR(10) NOT NULL,
    street                            VARCHAR(100) NOT NULL,
    number                            VARCHAR(20) NOT NULL,
    address_complement                VARCHAR(100),
    neighborhood                      VARCHAR(80) NOT NULL,
    city                              VARCHAR(80) NOT NULL,
    state                             VARCHAR(2) NOT NULL,
    address_type_id                   BIGINT DEFAULT 1 NOT NULL,
    customer_id                       BIGINT,
    business_id                       BIGINT,
    deleted_at                        TIMESTAMP(3) without time zone,
    company_id                        BIGINT,
    CONSTRAINT fk_address_customer_id FOREIGN KEY (customer_id) REFERENCES customer(id) ON UPDATE CASCADE ON DELETE SET NULL,
    CONSTRAINT fk_address_company_id FOREIGN KEY (company_id) REFERENCES company(id) ON UPDATE CASCADE ON DELETE SET NULL,
    CONSTRAINT fk_address_address_type_id FOREIGN KEY (address_type_id) REFERENCES address_types(id) ON UPDATE CASCADE ON DELETE RESTRICT
);

CREATE TABLE partners_list (
    id                                BIGSERIAL PRIMARY KEY,
    description                       VARCHAR(200) NOT NULL,
    site                              VARCHAR(200),
    contact                           VARCHAR(100),
    active                            BOOLEAN DEFAULT true NOT NULL
);

CREATE TABLE partners (
    id                                BIGSERIAL PRIMARY KEY,
    partners_list_id                  BIGINT NOT NULL,
    description                       VARCHAR(100) NOT NULL,
    document                          VARCHAR(18),
    account                           VARCHAR(20),
    branch                            VARCHAR(10),
    authentication_id                 BIGINT NOT NULL,
    client_id                         VARCHAR(100),
    client_secret                     VARCHAR(255),
    authentication                    VARCHAR(255),
    password                          VARCHAR(255),
    whpix_in_id                       VARCHAR(50),
    whpix_out_id                      VARCHAR(50),
    type_authorize_id                 BIGINT DEFAULT 1 NOT NULL,
    fixed_cash_in                     NUMERIC(16,2) DEFAULT 0.00,
    fixed_cash_out                    NUMERIC(16,2) DEFAULT 0.00,
    percent_cashin                    NUMERIC(16,2) DEFAULT 0.00,
    percent_cashout                   NUMERIC(16,2) DEFAULT 0.00,
    fixed_ref_cash_in                 NUMERIC(16,2) DEFAULT 0.00,
    fixed_ref_cash_out                NUMERIC(16,2) DEFAULT 0.00,
    percent_ref_cashin                NUMERIC(16,2) DEFAULT 0.00,
    percent_ref_cashout               NUMERIC(16,2) DEFAULT 0.00,
    priority                          INT DEFAULT 999,
    active                            BOOLEAN DEFAULT true NOT NULL
);

CREATE TABLE account_rules (
    id                                BIGSERIAL PRIMARY KEY,
    account_id                        BIGINT NOT NULL,
    receive_external                  BOOLEAN DEFAULT true NOT NULL,
    deposit_external                  BOOLEAN DEFAULT true NOT NULL,
    descricao                         VARCHAR(50),
    CONSTRAINT fk_account_rules_account_id FOREIGN KEY (account_id) REFERENCES accounts(id) ON UPDATE CASCADE ON DELETE RESTRICT
);

CREATE TABLE invoice (
    id                                BIGSERIAL PRIMARY KEY,
    identifier                        VARCHAR(50),
    key                               VARCHAR(77) NOT NULL,
    pix_key_type_id                   BIGINT NOT NULL,
    invoice_type_id                   BIGINT DEFAULT 2 NOT NULL,
    timeout                           INTEGER,
    expire                            INTEGER,
    partners_list_id                  BIGINT,
    amount                            NUMERIC(16,2) NOT NULL,
    invoice_status_id                 BIGINT DEFAULT 1 NOT NULL,
    external_id                       VARCHAR(50),
    document_number                   VARCHAR(18) NOT NULL,
    description                       VARCHAR(100) NOT NULL,
    account_id                        BIGINT NOT NULL,
    deleted_at                        TIMESTAMP(3) without time zone,
    CONSTRAINT fk_invoice_account_id FOREIGN KEY (account_id) REFERENCES accounts(id) ON UPDATE CASCADE ON DELETE RESTRICT,
    CONSTRAINT fk_invoice_partners_list_id FOREIGN KEY (partners_list_id) REFERENCES partners_list(id) ON UPDATE CASCADE ON DELETE SET NULL,
    CONSTRAINT fk_invoice_pix_key_type_id FOREIGN KEY (pix_key_type_id) REFERENCES pix_key_types(id) ON UPDATE CASCADE ON DELETE RESTRICT,
    CONSTRAINT fk_invoice_invoice_type_id FOREIGN KEY (invoice_type_id) REFERENCES invoice_types(id) ON UPDATE CASCADE ON DELETE RESTRICT,
    CONSTRAINT fk_invoice_invoice_status_id FOREIGN KEY (invoice_status_id) REFERENCES invoice_status_types(id) ON UPDATE CASCADE ON DELETE RESTRICT
);

CREATE TABLE control_med (
    id                                BIGSERIAL PRIMARY KEY,
    account_id                        BIGINT,
    invoice_id                        BIGINT,
    partners_id                       BIGINT,
    bank_id                           VARCHAR(10) NOT NULL,
    endtoend                          VARCHAR(32) NOT NULL,
    details                           VARCHAR(500),
    status_controle_med_id            BIGINT DEFAULT 1 NOT NULL,
    amount                            NUMERIC(10,4) NOT NULL,
    data_med                          TIMESTAMP(3) without time zone,
    deleted_at                        TIMESTAMP(3) without time zone,
    CONSTRAINT fk_control_med_account_id FOREIGN KEY (account_id) REFERENCES accounts(id) ON UPDATE CASCADE ON DELETE SET NULL,
    CONSTRAINT fk_control_med_invoice_id FOREIGN KEY (invoice_id) REFERENCES invoice(id) ON UPDATE CASCADE ON DELETE SET NULL,
    CONSTRAINT fk_control_med_partners_id FOREIGN KEY (partners_id) REFERENCES partners(id) ON UPDATE CASCADE ON DELETE SET NULL,
    CONSTRAINT fk_control_med_status_controle_med_id FOREIGN KEY (status_controle_med_id) REFERENCES status_controle_med_types(id) ON UPDATE CASCADE ON DELETE RESTRICT
);

CREATE TABLE key_pix (
    id                                BIGSERIAL PRIMARY KEY,
    key                               VARCHAR(77) NOT NULL,
    pix_key_type_id                   BIGINT NOT NULL,
    document_number                   VARCHAR(18) NOT NULL,
    description                       VARCHAR(100) NOT NULL,
    account_id                        BIGINT NOT NULL,
    partners_id                       BIGINT,
    deleted_at                        TIMESTAMP(3) without time zone,
    CONSTRAINT fk_key_pix_pix_key_type_id FOREIGN KEY (pix_key_type_id) REFERENCES pix_key_types(id) ON UPDATE CASCADE ON DELETE RESTRICT,
    CONSTRAINT fk_key_pix_account_id FOREIGN KEY (account_id) REFERENCES accounts(id) ON UPDATE CASCADE ON DELETE RESTRICT,
    CONSTRAINT fk_key_pix_partners_id FOREIGN KEY (partners_id) REFERENCES partners(id) ON UPDATE CASCADE ON DELETE SET NULL
);

CREATE TABLE key_pix_cache (
    id                                BIGSERIAL PRIMARY KEY,
    key                               VARCHAR(77) NOT NULL,
    pix_key_type_id                   BIGINT NOT NULL,
    document_number                   VARCHAR(18) NOT NULL,
    description                       VARCHAR(100) NOT NULL,
    bank_name                         VARCHAR(80),
    account_number                    VARCHAR(20),
    branch                            VARCHAR(10),
    ispb                              VARCHAR(8),
    hide_document                     BOOLEAN NOT NULL,
    deleted_at                        TIMESTAMP(3) without time zone,
    CONSTRAINT fk_key_pix_cache_pix_key_type_id FOREIGN KEY (pix_key_type_id) REFERENCES pix_key_types(id) ON UPDATE CASCADE ON DELETE RESTRICT
);

CREATE TABLE management (
    id                                BIGSERIAL PRIMARY KEY,
    full_name                         VARCHAR(150) NOT NULL,
    social_name                       VARCHAR(100) NOT NULL,
    type_person_id                    BIGINT NOT NULL,
    document_number                   VARCHAR(18) NOT NULL,
    phone_number                      VARCHAR(20) NOT NULL,
    email                             VARCHAR(120) NOT NULL,
    telegram_chat_id                  VARCHAR(20),
    customer_status_id                BIGINT NOT NULL,
    is_politically_exposed_person     BOOLEAN DEFAULT false NOT NULL,
    authentication_id                 BIGINT NOT NULL,
    deleted_at                        TIMESTAMP(3) without time zone,
    CONSTRAINT fk_management_type_person_id FOREIGN KEY (type_person_id) REFERENCES type_person_types(id) ON UPDATE CASCADE ON DELETE RESTRICT,
    CONSTRAINT fk_management_customer_status_id FOREIGN KEY (customer_status_id) REFERENCES customer_status_types(id) ON UPDATE CASCADE ON DELETE RESTRICT,
    CONSTRAINT fk_management_authentication_id FOREIGN KEY (authentication_id) REFERENCES authentication(id) ON UPDATE CASCADE ON DELETE RESTRICT
);

CREATE TABLE fees (
    id                                BIGSERIAL PRIMARY KEY,
    account_id                        BIGINT NOT NULL,
    fixed_cash_in                     NUMERIC(16,2) DEFAULT 0.00,
    fixed_cash_out                    NUMERIC(16,2) DEFAULT 0.00,
    percent_cashin                    NUMERIC(16,2) DEFAULT 0.00,
    percent_cashout                   NUMERIC(16,2) DEFAULT 0.00,
    percentsec_med                    NUMERIC(16,2) DEFAULT 0.00,
    fixed_ref_cash_in                 NUMERIC(16,2) DEFAULT 0.00,
    fixed_ref_cash_out                NUMERIC(16,2) DEFAULT 0.00,
    apagar                            VARCHAR(10),
    percent_ref_cashin                NUMERIC(16,2) DEFAULT 0.00,
    percent_ref_cashout               NUMERIC(16,2) DEFAULT 0.00,
    deleted_at                        TIMESTAMP(3) without time zone,
    CONSTRAINT fk_fees_account_id FOREIGN KEY (account_id) REFERENCES accounts(id) ON UPDATE CASCADE ON DELETE RESTRICT
);

CREATE TABLE history_med (
    id                                BIGSERIAL PRIMARY KEY,
    account_id                        BIGINT,
    control_med_id                    BIGINT,
    sec_med_id                        BIGINT,
    apagar                            VARCHAR(10),
    amount                            NUMERIC(10,4) NOT NULL,
    deleted_at                        TIMESTAMP(3) without time zone
);

CREATE TABLE sec_med (
    id                                BIGSERIAL PRIMARY KEY,
    account_id                        BIGINT NOT NULL,
    invoice_id                        BIGINT,
    partners_id                       BIGINT,
    apagar                            VARCHAR(10),
    transaction_id                    BIGINT NOT NULL,
    status_sec_med_id                 BIGINT DEFAULT 1 NOT NULL,
    amount                            NUMERIC(16,2) NOT NULL,
    scheduled_date                    TIMESTAMP(3) without time zone,
    deleted_at                        TIMESTAMP(3) without time zone
);

CREATE TABLE shared_key (
    id                                BIGSERIAL PRIMARY KEY,
    key                               VARCHAR(77) NOT NULL,
    pix_key_type_id                   BIGINT NOT NULL,
    description                       VARCHAR(100) NOT NULL,
    deleted_at                        TIMESTAMP(3) without time zone
);

CREATE TABLE styled (
    id                                BIGSERIAL PRIMARY KEY,
    url                               VARCHAR(300) NOT NULL,
    application_name                  VARCHAR(100) NOT NULL,
    title                             VARCHAR(100) NOT NULL,
    primary_color                     VARCHAR(7) NOT NULL,
    secondary_color                   VARCHAR(7) NOT NULL,
    font_color                        VARCHAR(7) NOT NULL,
    img                               VARCHAR(500) NOT NULL,
    favicon                           VARCHAR(500) NOT NULL,
    styled_type_id                    BIGINT DEFAULT 1 NOT NULL,
    company_id                        BIGINT,
    active                            BOOLEAN DEFAULT true NOT NULL
);

CREATE TABLE token_service (
    id                                BIGSERIAL PRIMARY KEY,
    description                       VARCHAR(100) NOT NULL,
    token                             VARCHAR(500),
    expire_in                         VARCHAR(20),
    authentication_id                 BIGINT NOT NULL,
    TIMESTAMP                         TIMESTAMP(3) without time zone,
    active                            BOOLEAN DEFAULT true NOT NULL
);

CREATE TABLE transaction (
    id                                BIGSERIAL PRIMARY KEY,
    account_id                        BIGINT NOT NULL,
    invoice_id                        BIGINT,
    partners_id                       BIGINT,
    transaction_id                    VARCHAR(64),
    charger_back_id                   VARCHAR(50),
    parent_id                         BIGINT,
    external_id                       VARCHAR(100),
    name                              VARCHAR(100),
    email                             VARCHAR(120),
    document_number                   VARCHAR(18),
    description                       VARCHAR(220),
    phone                             VARCHAR(20),
    amount                            NUMERIC(16,2) NOT NULL,
    isbp                              VARCHAR(8),
    bank_name                         VARCHAR(80),
    branch                            VARCHAR(10),
    account                           VARCHAR(20),
    endtoend_id                       VARCHAR(32),
    pix_key_type_id                   BIGINT,
    key                               VARCHAR(77),
    type_transaction_id               BIGINT NOT NULL,
    sub_type_transaction_id           BIGINT NOT NULL,
    remittance_information            VARCHAR(200),
    status_transaction_id             BIGINT NOT NULL,
    msg_error                         VARCHAR(500),
    telegram_notification             BOOLEAN DEFAULT false NOT NULL,
    try_count                         INTEGER DEFAULT 0 NOT NULL,
    created_at                        TIMESTAMP(3) without time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
    updated_at                        TIMESTAMP(3) without time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
    deleted_at                        TIMESTAMP(3) without time zone,
    endtoend_id_temp                  VARCHAR(32),
    -- ============================================================================
    -- CAMPOS DE AUDITORIA E RASTREABILIDADE (adicionados em 2025-11-20)
    -- ============================================================================
    gateway                           VARCHAR(50),                                 -- Gateway usado (seventrust, sulcred, etc.)
    pix_operation_type                VARCHAR(20),                                 -- Tipo operação (pix_out_key, pix_in_qrcode, etc.)
    -- ============================================================================
    -- CAMPOS DE TAXAS E VALORES DETALHADOS (adicionados em 2025-11-20)
    -- ============================================================================
    -- Valores base da transação
    requested_amount                  NUMERIC(16,2) DEFAULT 0.00 NOT NULL,    -- Valor solicitado pelo usuário
    net_amount                        NUMERIC(16,2) DEFAULT 0.00 NOT NULL,    -- Valor líquido enviado/recebido
    total_amount                      NUMERIC(16,2) DEFAULT 0.00 NOT NULL,    -- Valor total (amount + taxas)
    -- Taxas aplicadas (customer)
    fee_fixed                         NUMERIC(16,2) DEFAULT 0.00 NOT NULL,    -- Taxa fixa (ex: R$ 0,10)
    fee_percent_rate                  NUMERIC(16,4) DEFAULT 0.0000 NOT NULL,  -- Taxa percentual (ex: 0.50 = 0,5%)
    fee_percent_amount                NUMERIC(16,2) DEFAULT 0.00 NOT NULL,    -- Valor em R$ da taxa %
    fee_total                         NUMERIC(16,2) DEFAULT 0.00 NOT NULL,    -- Taxa total (fixed + percent)
    -- Comissões/taxas de referência (repasse)
    ref_fee_fixed                     NUMERIC(16,2) DEFAULT 0.00 NOT NULL,    -- Comissão fixa referência
    ref_fee_percent_rate              NUMERIC(16,4) DEFAULT 0.0000 NOT NULL,  -- Comissão % referência
    ref_fee_percent_amount            NUMERIC(16,2) DEFAULT 0.00 NOT NULL,    -- Valor comissão %
    ref_fee_total                     NUMERIC(16,2) DEFAULT 0.00 NOT NULL,    -- Comissão total
    -- Taxas do gateway (reconciliação)
    gateway_fee                       NUMERIC(16,2) DEFAULT 0.00 NOT NULL,    -- Taxa cobrada pelo gateway
    platform_fee                      NUMERIC(16,2) DEFAULT 0.00 NOT NULL,    -- Receita líquida da plataforma
    -- Rastreabilidade
    fees_id                           BIGINT,                                  -- ID do fees usado (se customizado)
    partner_fixed_cash_out            NUMERIC(16,2) DEFAULT 0.00,             -- Snapshot taxa fixa parceiro
    partner_percent_cashout           NUMERIC(16,4) DEFAULT 0.0000,           -- Snapshot taxa % parceiro
    fees_calculated_at                TIMESTAMP(3) without time zone,         -- Quando as taxas foram calculadas
    fee_calculation_version           VARCHAR(20) DEFAULT 'v1.0'              -- Versão do algoritmo de cálculo
);

CREATE TABLE webhook_manager (
    id                                BIGSERIAL PRIMARY KEY,
    callback_url                      VARCHAR(500) NOT NULL,
    username                          VARCHAR(50),
    password                          VARCHAR(255),
    api_key                           VARCHAR(100),
    webhook_type_id                   BIGINT NOT NULL,
    account_id                        BIGINT NOT NULL,
    deleted_at                        TIMESTAMP(3) without time zone
);

CREATE TABLE with_list_accounts (
    id                                BIGSERIAL PRIMARY KEY,
    type_external_id                  BIGINT NOT NULL,
    account_id                        BIGINT NOT NULL,
    document                          VARCHAR(18) NOT NULL
);

