// P2P service: internal transfers with sub_type_transaction_id=3
use async_trait::async_trait;
use rust_decimal::Decimal;
use std::sync::Arc;

use crate::accounts::AccountsRepository;
use crate::customer::CustomerRepository;
use crate::model::Transaction;
use crate::transaction::TransactionRepository;

#[derive(Debug, thiserror::Error)]
pub enum P2PError {
    #[error("Repository: {0}")]
    Repository(#[from] crate::transaction::RepositoryError),
    #[error("Accounts: {0}")]
    Accounts(#[from] crate::accounts::RepositoryError),
    #[error("Customer: {0}")]
    Customer(#[from] crate::customer::RepositoryError),
    #[error("Account not found")]
    SenderAccountNotFound,
    #[error("Receiver account not found")]
    ReceiverAccountNotFound,
    #[error("Insufficient balance")]
    InsufficientBalance,
    #[error("Invalid amount")]
    InvalidAmount,
    #[error("Receiver not found")]
    ReceiverNotFound,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct P2PSendResult {
    pub transfer_id: i64,
    pub status: String,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct P2PSearchResult {
    pub id: i64,
    pub full_name: String,
    pub email: String,
    pub document_number: String,
}

#[async_trait]
pub trait P2PService: Send + Sync {
    async fn p2p_send(
        &self,
        sender_authentication_id: i64,
        receiver_customer_id: i64,
        amount: Decimal,
        description: Option<String>,
    ) -> Result<P2PSendResult, P2PError>;
    async fn p2p_history(
        &self,
        authentication_id: i64,
        offset: i64,
        limit: i64,
    ) -> Result<Vec<Transaction>, P2PError>;
    async fn p2p_status(&self, transfer_id: i64, authentication_id: i64) -> Result<Option<Transaction>, P2PError>;
    async fn p2p_search(&self, query: &str) -> Result<Vec<P2PSearchResult>, P2PError>;
}

pub struct P2PServiceImpl {
    transaction_repo: Arc<dyn TransactionRepository>,
    customer_repo: Arc<dyn CustomerRepository>,
    accounts_repo: Arc<dyn AccountsRepository>,
}

impl P2PServiceImpl {
    pub fn new(
        transaction_repo: Arc<dyn TransactionRepository>,
        customer_repo: Arc<dyn CustomerRepository>,
        accounts_repo: Arc<dyn AccountsRepository>,
    ) -> Self {
        Self {
            transaction_repo,
            customer_repo,
            accounts_repo,
        }
    }
}

#[async_trait]
impl P2PService for P2PServiceImpl {
    async fn p2p_send(
        &self,
        sender_authentication_id: i64,
        receiver_customer_id: i64,
        amount: Decimal,
        description: Option<String>,
    ) -> Result<P2PSendResult, P2PError> {
        if amount <= Decimal::ZERO {
            return Err(P2PError::InvalidAmount);
        }
        let sender_account = self
            .accounts_repo
            .get_by_authentication_id(sender_authentication_id)
            .await?
            .ok_or(P2PError::SenderAccountNotFound)?;
        let receiver_customer = self
            .customer_repo
            .get_by_id(receiver_customer_id)
            .await?
            .ok_or(P2PError::ReceiverNotFound)?;
        let receiver_account = self
            .accounts_repo
            .get_by_authentication_id(receiver_customer.authentication_id)
            .await?
            .ok_or(P2PError::ReceiverAccountNotFound)?;
        if sender_account.id == receiver_account.id {
            return Err(P2PError::InvalidAmount);
        }
        let balance = self.transaction_repo.get_balance(sender_account.id).await?;
        if balance < amount {
            return Err(P2PError::InsufficientBalance);
        }
        let external_id = format!("p2p_{}", chrono::Utc::now().timestamp_millis());
        let desc = description.unwrap_or_else(|| "P2P transfer".to_string());
        let (debit_id, _) = self.transaction_repo.insert_p2p_transfer(
            sender_account.id,
            receiver_account.id,
            &external_id,
            &receiver_customer.full_name,
            &receiver_customer.document_number,
            &receiver_customer.full_name,
            &receiver_customer.document_number,
            amount,
            &desc,
        ).await?;
        Ok(P2PSendResult {
            transfer_id: debit_id,
            status: "completed".to_string(),
        })
    }

    async fn p2p_history(
        &self,
        authentication_id: i64,
        offset: i64,
        limit: i64,
    ) -> Result<Vec<Transaction>, P2PError> {
        let account = self
            .accounts_repo
            .get_by_authentication_id(authentication_id)
            .await?
            .ok_or(P2PError::SenderAccountNotFound)?;
        let items = self
            .transaction_repo
            .list_p2p_by_account(account.id, offset, limit)
            .await?;
        Ok(items)
    }

    async fn p2p_status(&self, transfer_id: i64, authentication_id: i64) -> Result<Option<Transaction>, P2PError> {
        let account = self
            .accounts_repo
            .get_by_authentication_id(authentication_id)
            .await?
            .ok_or(P2PError::SenderAccountNotFound)?;
        let tx = self.transaction_repo.get_by_id(transfer_id).await?;
        let Some(ref t) = tx else { return Ok(None) };
        if t.sub_type_transaction_id != 3 {
            return Ok(None);
        }
        if t.account_id == account.id {
            return Ok(tx); // user sent (debit) or received (credit)
        }
        if t.parent_id > 0 {
            let parent = self.transaction_repo.get_by_id(t.parent_id).await?;
            if parent.as_ref().map(|p| p.account_id == account.id).unwrap_or(false) {
                return Ok(tx); // user sent, we're looking at the credit tx
            }
        }
        Ok(None)
    }

    async fn p2p_search(&self, query: &str) -> Result<Vec<P2PSearchResult>, P2PError> {
        let query = query.trim();
        if query.is_empty() {
            return Ok(vec![]);
        }
        let customers = self.customer_repo.search(query).await?;
        let results: Vec<P2PSearchResult> = customers
            .into_iter()
            .map(|c| P2PSearchResult {
                id: c.id,
                full_name: c.full_name,
                email: c.email,
                document_number: c.document_number,
            })
            .collect();
        Ok(results)
    }
}
