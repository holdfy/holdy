//! Ponte opcional para o contrato Soroban de escrow (Stellar testnet/mainnet).
//!
//! Sem a feature Cargo `soroban`, apenas [`MockSorobanBridge`] existe. Com `soroban` + `APICASH_SOROBAN_ENABLED=1`,
//! usa [`LiveSorobanBridge`] (CLI `stellar`); caso contrário cai no mock.

use std::sync::Arc;

use async_trait::async_trait;
use uuid::Uuid;

use crate::errors::CustodyError;

/// Resultado de um lock on-chain (escrow Soroban).
#[derive(Debug, Clone, Default)]
pub struct SorobanLockOutcome {
    pub escrow_contract_id: Option<String>,
    pub lock_tx_hash: Option<String>,
    pub is_mock: bool,
}

/// Resultado de deploy do Wasm do escrow.
#[derive(Debug, Clone)]
pub struct SorobanDeployOutcome {
    pub contract_id: String,
    pub tx_hash: Option<String>,
    pub is_mock: bool,
}

/// Parâmetros para `lock_funds` no contrato.
#[derive(Debug, Clone)]
pub struct LockInvokeParams {
    pub order_id: Uuid,
    pub order_key: u64,
    pub buyer_stellar: String,
    pub seller_stellar: String,
    pub token_contract_id: String,
    pub amount_stroops: i128,
}

#[async_trait]
pub trait SorobanCustodyBridge: Send + Sync {
    async fn deploy_escrow_contract(&self) -> Result<SorobanDeployOutcome, CustodyError>;

    async fn invoke_lock(
        &self,
        params: LockInvokeParams,
    ) -> Result<SorobanLockOutcome, CustodyError>;

    async fn invoke_confirm_delivery(
        &self,
        order_key: u64,
        escrow_contract_id: &str,
    ) -> Result<Option<String>, CustodyError>;

    async fn invoke_release(
        &self,
        order_key: u64,
        escrow_contract_id: &str,
    ) -> Result<Option<String>, CustodyError>;

    async fn invoke_open_dispute(
        &self,
        order_key: u64,
        escrow_contract_id: &str,
    ) -> Result<Option<String>, CustodyError>;
}

/// Bridge mock: não fala com a rede.
pub struct MockSorobanBridge;

#[async_trait]
impl SorobanCustodyBridge for MockSorobanBridge {
    async fn deploy_escrow_contract(&self) -> Result<SorobanDeployOutcome, CustodyError> {
        Ok(SorobanDeployOutcome {
            contract_id: "mock_escrow_contract".into(),
            tx_hash: Some("mock_deploy_tx".into()),
            is_mock: true,
        })
    }

    async fn invoke_lock(
        &self,
        params: LockInvokeParams,
    ) -> Result<SorobanLockOutcome, CustodyError> {
        let h = format!(
            "mock_lock_{:x}_{}",
            params.order_key,
            params.order_id.simple()
        );
        Ok(SorobanLockOutcome {
            escrow_contract_id: Some("mock_escrow_contract".into()),
            lock_tx_hash: Some(h),
            is_mock: true,
        })
    }

    async fn invoke_confirm_delivery(
        &self,
        order_key: u64,
        _escrow_contract_id: &str,
    ) -> Result<Option<String>, CustodyError> {
        Ok(Some(format!("mock_confirm_delivery_{order_key}")))
    }

    async fn invoke_release(
        &self,
        order_key: u64,
        _escrow_contract_id: &str,
    ) -> Result<Option<String>, CustodyError> {
        Ok(Some(format!("mock_release_{order_key}")))
    }

    async fn invoke_open_dispute(
        &self,
        order_key: u64,
        _escrow_contract_id: &str,
    ) -> Result<Option<String>, CustodyError> {
        Ok(Some(format!("mock_open_dispute_{order_key}")))
    }
}

/// Converte UUID em chave u64 estável para o contrato.
pub fn order_key_from_uuid(id: Uuid) -> u64 {
    let b = id.as_bytes();
    u64::from_ne_bytes(b[0..8].try_into().unwrap_or([0u8; 8]))
}

/// Extrai o hash real da transação da stderr do `stellar` CLI (formato "Signing transaction:
/// <hash>"). A stdout do `contract invoke` carrega o valor de retorno da função serializado em
/// JSON (ex: `"null"` para `Result<(), _>`), não o hash — por isso nunca deve ser usada como tal.
fn extract_tx_hash(stderr: &str) -> Option<String> {
    stderr.lines().find_map(|line| {
        line.trim()
            .rsplit_once("Signing transaction: ")
            .map(|(_, hash)| hash.trim().to_string())
    })
}

fn require_testnet_on_chain() -> bool {
    std::env::var("APICASH_REQUIRE_TESTNET")
        .map(|v| matches!(v.trim().to_ascii_lowercase().as_str(), "1" | "true" | "yes" | "on"))
        .unwrap_or(false)
}

/// Instância padrão: mock, ou live com feature `soroban` e `APICASH_SOROBAN_ENABLED=1`.
pub fn custody_bridge_from_env() -> Arc<dyn SorobanCustodyBridge> {
    let require = require_testnet_on_chain();
    #[cfg(feature = "soroban")]
    {
        if std::env::var("APICASH_SOROBAN_ENABLED")
            .map(|v| v == "1" || v.eq_ignore_ascii_case("true"))
            .unwrap_or(false)
        {
            return Arc::new(LiveSorobanBridge::from_env());
        }
    }
    if require {
        panic!(
            "APICASH_REQUIRE_TESTNET=1: compile with feature soroban and set APICASH_SOROBAN_ENABLED=1"
        );
    }
    Arc::new(MockSorobanBridge)
}

#[cfg(feature = "soroban")]
pub struct LiveSorobanBridge {
    rpc_url: Option<String>,
    network_passphrase: Option<String>,
    escrow_contract_id: Option<String>,
    source_secret: Option<String>,
    stellar_bin: String,
    strict_live: bool,
    // Chamadas concorrentes assinadas pela mesma conta Stellar (buyer/deployer, compartilhada
    // entre pedidos em dev/testnet) competem pelo sequence number da conta e falham com
    // TxBadSeq / transaction submission timeout. O `stellar` CLI não faz essa coordenação
    // sozinho, então serializamos aqui: só uma invocação on-chain por vez.
    chain_lock: tokio::sync::Mutex<()>,
}

#[cfg(feature = "soroban")]
impl LiveSorobanBridge {
    pub fn from_env() -> Self {
        Self {
            rpc_url: std::env::var("APICASH_SOROBAN_RPC_URL")
                .or_else(|_| std::env::var("STELLAR_RPC_URL"))
                .ok(),
            network_passphrase: std::env::var("APICASH_STELLAR_NETWORK_PASSPHRASE")
                .or_else(|_| std::env::var("STELLAR_NETWORK_PASSPHRASE"))
                .ok()
                .or_else(|| Some("Test SDF Network ; September 2015".into())),
            escrow_contract_id: std::env::var("APICASH_SOROBAN_ESCROW_CONTRACT_ID").ok(),
            source_secret: std::env::var("APICASH_SOROBAN_SOURCE_SECRET")
                .or_else(|_| std::env::var("APICASH_STELLAR_SECRET_KEY"))
                .ok(),
            stellar_bin: std::env::var("APICASH_STELLAR_CLI_BIN")
                .unwrap_or_else(|_| "stellar".into()),
            strict_live: std::env::var("APICASH_SOROBAN_STRICT")
                .map(|v| v == "1" || v.eq_ignore_ascii_case("true"))
                .unwrap_or(false)
                || require_testnet_on_chain(),
            chain_lock: tokio::sync::Mutex::new(()),
        }
    }

    fn missing_for_live(&self) -> bool {
        self.rpc_url.is_none() || self.escrow_contract_id.is_none() || self.source_secret.is_none()
    }

    fn missing_for_deploy(&self) -> bool {
        self.rpc_url.is_none() || self.source_secret.is_none()
    }

    fn buyer_source(&self) -> Option<String> {
        std::env::var("APICASH_SOROBAN_BUYER_SOURCE")
            .or_else(|_| std::env::var("APICASH_STELLAR_BUYER_SOURCE"))
            .ok()
            .or_else(|| self.source_secret.clone())
    }

    fn dispute_source(&self) -> Option<String> {
        std::env::var("APICASH_SOROBAN_DISPUTE_SOURCE")
            .or_else(|_| std::env::var("APICASH_STELLAR_SELLER_SOURCE"))
            .ok()
            .or_else(|| self.buyer_source())
    }

    async fn run_stellar_contract_deploy(
        &self,
        wasm_path: &str,
    ) -> Result<(String, String), CustodyError> {
        let rpc = self
            .rpc_url
            .as_ref()
            .ok_or_else(|| CustodyError::Soroban("APICASH_SOROBAN_RPC_URL missing".into()))?;
        let mut cmd = tokio::process::Command::new(&self.stellar_bin);
        cmd.args(["contract", "deploy", "--wasm", wasm_path, "--rpc-url", rpc]);
        if let Some(ref p) = self.network_passphrase {
            cmd.args(["--network-passphrase", p]);
        }
        if let Some(s) = self.buyer_source() {
            cmd.args(["--source", &s, "--sign-with-key", &s]);
        }
        let _guard = self.chain_lock.lock().await;
        let out = cmd
            .output()
            .await
            .map_err(|e| CustodyError::Soroban(e.to_string()))?;
        if !out.status.success() {
            let stderr = String::from_utf8_lossy(&out.stderr);
            return Err(CustodyError::Soroban(format!(
                "stellar deploy failed: {stderr}"
            )));
        }
        let stdout = String::from_utf8_lossy(&out.stdout).to_string();
        let contract_id = stdout
            .lines()
            .find(|l| l.starts_with('C') && l.len() >= 50)
            .map(|s| s.trim().to_string())
            .unwrap_or_else(|| stdout.trim().to_string());
        let tx_hash = stdout
            .lines()
            .find(|l| l.contains("hash") || l.len() == 64)
            .map(|s| s.split_whitespace().last().unwrap_or("").to_string())
            .unwrap_or_default();
        Ok((contract_id, tx_hash))
    }

    async fn run_stellar_invoke_lock(
        &self,
        params: &LockInvokeParams,
    ) -> Result<String, CustodyError> {
        let escrow = self.escrow_contract_id.as_ref().ok_or_else(|| {
            CustodyError::Soroban("APICASH_SOROBAN_ESCROW_CONTRACT_ID missing".into())
        })?;
        let rpc = self
            .rpc_url
            .as_ref()
            .ok_or_else(|| CustodyError::Soroban("APICASH_SOROBAN_RPC_URL missing".into()))?;
        let mut cmd = tokio::process::Command::new(&self.stellar_bin);
        cmd.args(["contract", "invoke", "--id", escrow, "--rpc-url", rpc]);
        if let Some(ref p) = self.network_passphrase {
            cmd.args(["--network-passphrase", p]);
        }
        if let Some(s) = self.buyer_source() {
            cmd.args(["--source", &s, "--sign-with-key", &s]);
        }
        cmd.args([
            "--",
            "lock",
            "--order_id",
            &params.order_key.to_string(),
            "--buyer",
            &params.buyer_stellar,
            "--seller",
            &params.seller_stellar,
            "--token",
            &params.token_contract_id,
            "--amount",
            &params.amount_stroops.to_string(),
        ]);
        let _guard = self.chain_lock.lock().await;
        let out = cmd
            .output()
            .await
            .map_err(|e| CustodyError::Soroban(e.to_string()))?;
        let stderr = String::from_utf8_lossy(&out.stderr);
        if !out.status.success() {
            return Err(CustodyError::Soroban(format!(
                "stellar invoke lock_funds failed: {stderr}"
            )));
        }
        Ok(extract_tx_hash(&stderr)
            .unwrap_or_else(|| String::from_utf8_lossy(&out.stdout).trim().to_string()))
    }
}

#[cfg(feature = "soroban")]
#[async_trait]
impl SorobanCustodyBridge for LiveSorobanBridge {
    async fn deploy_escrow_contract(&self) -> Result<SorobanDeployOutcome, CustodyError> {
        if self.missing_for_deploy() {
            tracing::warn!("Soroban live: credenciais incompletas — deploy mock");
            let mock = MockSorobanBridge;
            return mock.deploy_escrow_contract().await;
        }
        let wasm = std::env::var("APICASH_SOROBAN_WASM_PATH").map_err(|_| {
            CustodyError::Soroban("APICASH_SOROBAN_WASM_PATH required for deploy".into())
        })?;
        let (id, tx) = self.run_stellar_contract_deploy(&wasm).await?;
        Ok(SorobanDeployOutcome {
            contract_id: id,
            tx_hash: Some(tx),
            is_mock: false,
        })
    }

    async fn invoke_lock(
        &self,
        params: LockInvokeParams,
    ) -> Result<SorobanLockOutcome, CustodyError> {
        if self.missing_for_live() {
            tracing::warn!("Soroban live: fallback mock lock");
            let mock = MockSorobanBridge;
            return mock.invoke_lock(params).await;
        }
        match self.run_stellar_invoke_lock(&params).await {
            Ok(tx_out) => Ok(SorobanLockOutcome {
                escrow_contract_id: self.escrow_contract_id.clone(),
                lock_tx_hash: Some(tx_out),
                is_mock: false,
            }),
            Err(e) => {
                if self.strict_live {
                    return Err(e);
                }
                tracing::warn!(error = %e, "Soroban invoke_lock failed — fallback mock");
                let mock = MockSorobanBridge;
                mock.invoke_lock(params).await
            }
        }
    }

    async fn invoke_confirm_delivery(
        &self,
        order_key: u64,
        escrow_contract_id: &str,
    ) -> Result<Option<String>, CustodyError> {
        if self.missing_for_live() {
            let mock = MockSorobanBridge;
            return mock
                .invoke_confirm_delivery(order_key, escrow_contract_id)
                .await;
        }
        let rpc = self
            .rpc_url
            .as_ref()
            .ok_or_else(|| CustodyError::Soroban("APICASH_SOROBAN_RPC_URL missing".into()))?;
        let mut cmd = tokio::process::Command::new(&self.stellar_bin);
        cmd.args([
            "contract",
            "invoke",
            "--id",
            escrow_contract_id,
            "--rpc-url",
            rpc,
        ]);
        if let Some(ref p) = self.network_passphrase {
            cmd.args(["--network-passphrase", p]);
        }
        if let Some(s) = self.buyer_source() {
            cmd.args(["--source", &s, "--sign-with-key", &s]);
        }
        cmd.args([
            "--",
            "confirm_delivery",
            "--order_id",
            &order_key.to_string(),
        ]);
        let _guard = self.chain_lock.lock().await;
        let out = cmd
            .output()
            .await
            .map_err(|e| CustodyError::Soroban(e.to_string()))?;
        let stderr = String::from_utf8_lossy(&out.stderr);
        if !out.status.success() {
            if self.strict_live {
                return Err(CustodyError::Soroban(format!(
                    "confirm_delivery failed: {stderr}"
                )));
            }
            tracing::warn!(%stderr, "confirm_delivery failed — mock");
            let mock = MockSorobanBridge;
            return mock
                .invoke_confirm_delivery(order_key, escrow_contract_id)
                .await;
        }
        Ok(Some(extract_tx_hash(&stderr).unwrap_or_else(|| {
            String::from_utf8_lossy(&out.stdout).trim().to_string()
        })))
    }

    async fn invoke_release(
        &self,
        order_key: u64,
        escrow_contract_id: &str,
    ) -> Result<Option<String>, CustodyError> {
        if self.missing_for_live() {
            let mock = MockSorobanBridge;
            return mock.invoke_release(order_key, escrow_contract_id).await;
        }
        let rpc = self
            .rpc_url
            .as_ref()
            .ok_or_else(|| CustodyError::Soroban("APICASH_SOROBAN_RPC_URL missing".into()))?;
        let mut cmd = tokio::process::Command::new(&self.stellar_bin);
        cmd.args([
            "contract",
            "invoke",
            "--id",
            escrow_contract_id,
            "--rpc-url",
            rpc,
        ]);
        if let Some(ref p) = self.network_passphrase {
            cmd.args(["--network-passphrase", p]);
        }
        if let Some(s) = self.buyer_source() {
            cmd.args(["--source", &s, "--sign-with-key", &s]);
        }
        cmd.args(["--", "release", "--order_id", &order_key.to_string()]);
        let _guard = self.chain_lock.lock().await;
        let out = cmd
            .output()
            .await
            .map_err(|e| CustodyError::Soroban(e.to_string()))?;
        let stderr = String::from_utf8_lossy(&out.stderr);
        if !out.status.success() {
            if self.strict_live {
                return Err(CustodyError::Soroban(format!("release failed: {stderr}")));
            }
            tracing::warn!(%stderr, "release failed — mock");
            let mock = MockSorobanBridge;
            return mock.invoke_release(order_key, escrow_contract_id).await;
        }
        Ok(Some(extract_tx_hash(&stderr).unwrap_or_else(|| {
            String::from_utf8_lossy(&out.stdout).trim().to_string()
        })))
    }

    async fn invoke_open_dispute(
        &self,
        order_key: u64,
        escrow_contract_id: &str,
    ) -> Result<Option<String>, CustodyError> {
        if self.missing_for_live() {
            let mock = MockSorobanBridge;
            return mock
                .invoke_open_dispute(order_key, escrow_contract_id)
                .await;
        }
        let rpc = self
            .rpc_url
            .as_ref()
            .ok_or_else(|| CustodyError::Soroban("APICASH_SOROBAN_RPC_URL missing".into()))?;
        let mut cmd = tokio::process::Command::new(&self.stellar_bin);
        cmd.args([
            "contract",
            "invoke",
            "--id",
            escrow_contract_id,
            "--rpc-url",
            rpc,
        ]);
        if let Some(ref p) = self.network_passphrase {
            cmd.args(["--network-passphrase", p]);
        }
        if let Some(s) = self.dispute_source() {
            cmd.args(["--source", &s, "--sign-with-key", &s]);
        }
        let opened_by = std::env::var("APICASH_STELLAR_BUYER_ADDRESS")
            .or_else(|_| std::env::var("APICASH_STELLAR_SELLER_ADDRESS"))
            .map_err(|_| {
                CustodyError::Soroban(
                    "APICASH_STELLAR_BUYER_ADDRESS or APICASH_STELLAR_SELLER_ADDRESS required for open_dispute".into(),
                )
            })?;
        cmd.args([
            "--",
            "open_dispute",
            "--order_id",
            &order_key.to_string(),
            "--opened_by",
            &opened_by,
        ]);
        let _guard = self.chain_lock.lock().await;
        let out = cmd
            .output()
            .await
            .map_err(|e| CustodyError::Soroban(e.to_string()))?;
        let stderr = String::from_utf8_lossy(&out.stderr);
        if !out.status.success() {
            if self.strict_live {
                return Err(CustodyError::Soroban(format!(
                    "open_dispute failed: {stderr}"
                )));
            }
            tracing::warn!(%stderr, "open_dispute failed — mock");
            let mock = MockSorobanBridge;
            return mock
                .invoke_open_dispute(order_key, escrow_contract_id)
                .await;
        }
        Ok(Some(extract_tx_hash(&stderr).unwrap_or_else(|| {
            String::from_utf8_lossy(&out.stdout).trim().to_string()
        })))
    }
}
