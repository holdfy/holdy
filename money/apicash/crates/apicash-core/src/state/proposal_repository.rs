//! Persistence boundary for proposals (two-party escrow negotiation).

use std::collections::HashMap;
use std::sync::Arc;

use async_trait::async_trait;
use tokio::sync::RwLock;
use uuid::Uuid;

use crate::dto::StoredProposal;

#[async_trait]
pub trait ProposalRepository: Send + Sync {
    async fn save(&self, proposal: StoredProposal) -> Result<(), String>;
    async fn get(&self, id: Uuid) -> Result<Option<StoredProposal>, String>;
    async fn update(&self, proposal: StoredProposal) -> Result<(), String>;
}

pub struct InMemoryProposalRepository {
    by_id: Arc<RwLock<HashMap<Uuid, StoredProposal>>>,
}

impl InMemoryProposalRepository {
    pub fn new() -> Self {
        Self {
            by_id: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

impl Default for InMemoryProposalRepository {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl ProposalRepository for InMemoryProposalRepository {
    async fn save(&self, proposal: StoredProposal) -> Result<(), String> {
        self.by_id.write().await.insert(proposal.id, proposal);
        Ok(())
    }

    async fn get(&self, id: Uuid) -> Result<Option<StoredProposal>, String> {
        Ok(self.by_id.read().await.get(&id).cloned())
    }

    async fn update(&self, proposal: StoredProposal) -> Result<(), String> {
        let mut g = self.by_id.write().await;
        if !g.contains_key(&proposal.id) {
            return Err(format!("proposal not found: {}", proposal.id));
        }
        g.insert(proposal.id, proposal);
        Ok(())
    }
}
