-- ============================================================================
-- Gateway Failover Schema Migration
-- Based on gateboxgo/database/gateway-failover-schema.sql
-- ============================================================================

-- ============================================================================
-- TABELA 1: gateway_config
-- ============================================================================
CREATE TABLE IF NOT EXISTS gateway_config (
    id SERIAL PRIMARY KEY,
    gateway_name VARCHAR(50) NOT NULL UNIQUE,
    display_name VARCHAR(100) NOT NULL,
    priority INT NOT NULL DEFAULT 999,
    status VARCHAR(20) NOT NULL DEFAULT 'active',
    enabled BOOLEAN NOT NULL DEFAULT true,
    base_url VARCHAR(255),
    auth_type VARCHAR(50),
    config_json JSONB,
    max_daily_volume DECIMAL(15,2),
    max_transaction_amount DECIMAL(15,2),
    supports_cash_in BOOLEAN DEFAULT true,
    supports_cash_out BOOLEAN DEFAULT true,
    created_at TIMESTAMP NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP NOT NULL DEFAULT NOW(),
    created_by VARCHAR(100),
    notes TEXT
);

CREATE INDEX IF NOT EXISTS idx_gateway_config_priority ON gateway_config(priority) WHERE enabled = true;
CREATE INDEX IF NOT EXISTS idx_gateway_config_status ON gateway_config(status);
CREATE INDEX IF NOT EXISTS idx_gateway_config_enabled ON gateway_config(enabled);

-- ============================================================================
-- TABELA 2: gateway_health
-- ============================================================================
CREATE TABLE IF NOT EXISTS gateway_health (
    id SERIAL PRIMARY KEY,
    gateway_name VARCHAR(50) NOT NULL UNIQUE REFERENCES gateway_config(gateway_name),
    circuit_state VARCHAR(20) NOT NULL DEFAULT 'CLOSED',
    is_healthy BOOLEAN NOT NULL DEFAULT true,
    last_health_check TIMESTAMP,
    next_health_check TIMESTAMP,
    avg_response_time_ms INT,
    success_rate DECIMAL(5,2),
    total_requests_today INT DEFAULT 0,
    total_errors_today INT DEFAULT 0,
    total_volume_today DECIMAL(15,2) DEFAULT 0,
    consecutive_errors INT DEFAULT 0,
    consecutive_successes INT DEFAULT 0,
    unique_clients_with_errors INT DEFAULT 0,
    last_error_at TIMESTAMP,
    last_success_at TIMESTAMP,
    circuit_opened_at TIMESTAMP,
    circuit_half_opened_at TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT NOW(),
    error_details JSONB
);

CREATE INDEX IF NOT EXISTS idx_gateway_health_state ON gateway_health(circuit_state);
CREATE INDEX IF NOT EXISTS idx_gateway_health_healthy ON gateway_health(is_healthy);
CREATE INDEX IF NOT EXISTS idx_gateway_health_last_check ON gateway_health(last_health_check);

-- ============================================================================
-- TABELA 3: gateway_error_log
-- ============================================================================
CREATE TABLE IF NOT EXISTS gateway_error_log (
    id BIGSERIAL PRIMARY KEY,
    gateway_name VARCHAR(50) NOT NULL REFERENCES gateway_config(gateway_name),
    transaction_id BIGINT,
    customer_id BIGINT,
    account_id BIGINT,
    error_type VARCHAR(100),
    error_message TEXT,
    error_code VARCHAR(50),
    http_status_code INT,
    request_type VARCHAR(50),
    request_amount DECIMAL(15,2),
    request_payload JSONB,
    response_payload JSONB,
    response_time_ms INT,
    occurred_at TIMESTAMP NOT NULL DEFAULT NOW(),
    was_retried BOOLEAN DEFAULT false,
    caused_failover BOOLEAN DEFAULT false,
    expires_at TIMESTAMP
);

CREATE INDEX IF NOT EXISTS idx_gateway_error_log_gateway ON gateway_error_log(gateway_name, occurred_at DESC);
CREATE INDEX IF NOT EXISTS idx_gateway_error_log_customer ON gateway_error_log(customer_id, occurred_at DESC);
CREATE INDEX IF NOT EXISTS idx_gateway_error_log_occurred ON gateway_error_log(occurred_at DESC);
CREATE INDEX IF NOT EXISTS idx_gateway_error_log_type ON gateway_error_log(error_type);
CREATE INDEX IF NOT EXISTS idx_gateway_error_log_expires ON gateway_error_log(expires_at) WHERE expires_at IS NOT NULL;

-- ============================================================================
-- TABELA 4: gateway_failover_config
-- ============================================================================
CREATE TABLE IF NOT EXISTS gateway_failover_config (
    id SERIAL PRIMARY KEY,
    config_key VARCHAR(100) NOT NULL UNIQUE,
    config_value VARCHAR(255) NOT NULL,
    value_type VARCHAR(20) NOT NULL,
    description TEXT,
    updated_at TIMESTAMP NOT NULL DEFAULT NOW(),
    updated_by VARCHAR(100)
);

CREATE INDEX IF NOT EXISTS idx_failover_config_key ON gateway_failover_config(config_key);

-- ============================================================================
-- TABELA 5: gateway_failover_history
-- ============================================================================
CREATE TABLE IF NOT EXISTS gateway_failover_history (
    id BIGSERIAL PRIMARY KEY,
    from_gateway VARCHAR(50) REFERENCES gateway_config(gateway_name),
    to_gateway VARCHAR(50) NOT NULL REFERENCES gateway_config(gateway_name),
    reason VARCHAR(50) NOT NULL,
    trigger_type VARCHAR(50),
    error_count INT,
    affected_clients INT,
    from_gateway_health_score DECIMAL(5,2),
    to_gateway_health_score DECIMAL(5,2),
    details JSONB,
    performed_by VARCHAR(100),
    occurred_at TIMESTAMP NOT NULL DEFAULT NOW(),
    was_successful BOOLEAN DEFAULT true,
    rollback_at TIMESTAMP
);

CREATE INDEX IF NOT EXISTS idx_failover_history_from ON gateway_failover_history(from_gateway, occurred_at DESC);
CREATE INDEX IF NOT EXISTS idx_failover_history_to ON gateway_failover_history(to_gateway, occurred_at DESC);
CREATE INDEX IF NOT EXISTS idx_failover_history_occurred ON gateway_failover_history(occurred_at DESC);
CREATE INDEX IF NOT EXISTS idx_failover_history_reason ON gateway_failover_history(reason);

-- ============================================================================
-- VIEW: gateway_status_dashboard
-- ============================================================================
CREATE OR REPLACE VIEW gateway_status_dashboard AS
SELECT
    gc.gateway_name,
    gc.display_name,
    gc.priority,
    gc.status AS config_status,
    gc.enabled,
    gh.circuit_state,
    gh.is_healthy,
    gh.avg_response_time_ms,
    gh.success_rate,
    gh.consecutive_errors,
    gh.total_requests_today,
    gh.total_errors_today,
    gh.total_volume_today,
    gh.last_health_check,
    gh.last_error_at,
    gh.last_success_at,
    (SELECT occurred_at FROM gateway_failover_history
     WHERE from_gateway = gc.gateway_name OR to_gateway = gc.gateway_name
     ORDER BY occurred_at DESC LIMIT 1) AS last_failover_at,
    (SELECT COUNT(*) FROM gateway_error_log
     WHERE gateway_name = gc.gateway_name
     AND occurred_at > NOW() - INTERVAL '24 hours') AS errors_last_24h
FROM gateway_config gc
LEFT JOIN gateway_health gh ON gc.gateway_name = gh.gateway_name
ORDER BY gc.priority, gc.gateway_name;

-- ============================================================================
-- FUNCTION: cleanup_old_gateway_logs
-- ============================================================================
CREATE OR REPLACE FUNCTION cleanup_old_gateway_logs()
RETURNS void AS $$
DECLARE
    retention_days INT;
BEGIN
    SELECT CAST(config_value AS INT) INTO retention_days
    FROM gateway_failover_config
    WHERE config_key = 'error_log_retention_days';

    DELETE FROM gateway_error_log
    WHERE occurred_at < NOW() - (retention_days || ' days')::INTERVAL;

    RAISE NOTICE 'Cleaned up gateway error logs older than % days', retention_days;
END;
$$ LANGUAGE plpgsql;

-- ============================================================================
-- FUNCTION: reset_daily_gateway_metrics
-- ============================================================================
CREATE OR REPLACE FUNCTION reset_daily_gateway_metrics()
RETURNS void AS $$
BEGIN
    UPDATE gateway_health
    SET
        total_requests_today = 0,
        total_errors_today = 0,
        total_volume_today = 0,
        updated_at = NOW();

    RAISE NOTICE 'Reset daily metrics for all gateways';
END;
$$ LANGUAGE plpgsql;

-- ============================================================================
-- INITIAL DATA: gateway_failover_config
-- ============================================================================
INSERT INTO gateway_failover_config (config_key, config_value, value_type, description) VALUES
    ('error_threshold', '5', 'int', 'Número de erros consecutivos para abrir circuit breaker'),
    ('client_threshold', '3', 'int', 'Número mínimo de clientes únicos com erro para failover'),
    ('time_window_minutes', '5', 'int', 'Janela de tempo em minutos para análise de erros'),
    ('health_check_interval_minutes', '2', 'int', 'Intervalo entre health checks (minutos)'),
    ('cooldown_minutes', '10', 'int', 'Tempo de espera antes de voltar ao master (minutos)'),
    ('half_open_success_threshold', '3', 'int', 'Sucessos consecutivos para fechar circuit em HALF_OPEN'),
    ('retry_before_failover', '3', 'int', 'Tentativas antes de registrar erro definitivo'),
    ('error_log_retention_days', '30', 'int', 'Dias para manter logs de erro'),
    ('enable_auto_failover', 'true', 'boolean', 'Habilitar failover automático'),
    ('enable_auto_recovery', 'true', 'boolean', 'Habilitar recuperação automática ao master'),
    ('circuit_open_duration_minutes', '10', 'int', 'Tempo que circuit fica OPEN antes de HALF_OPEN')
ON CONFLICT (config_key) DO NOTHING;

-- ============================================================================
-- INITIAL DATA: gateway_config
-- ============================================================================
INSERT INTO gateway_config (gateway_name, display_name, priority, status, enabled, auth_type, supports_cash_in, supports_cash_out) VALUES
    ('seventrust', 'SevenTrust (7Trust)', 1, 'active', true, 'mtls', true, true),
    ('sulcred', 'Sulcred', 2, 'active', true, 'mtls', true, true),
    ('voluti', 'Volut (Voluti)', 3, 'active', true, 'mtls', true, true),
    ('valorion', 'Valorion Pay', 4, 'active', true, 'bearer', true, true),
    ('fastpay', 'FastPay', 5, 'active', true, 'oauth2', true, true),
    ('asaas', 'ASAAS', 6, 'active', true, 'api_key', false, true)
ON CONFLICT (gateway_name) DO NOTHING;

-- ============================================================================
-- INITIAL DATA: gateway_health
-- ============================================================================
INSERT INTO gateway_health (gateway_name, circuit_state, is_healthy)
SELECT gateway_name, 'CLOSED', true
FROM gateway_config
ON CONFLICT (gateway_name) DO NOTHING;
