// ProviderSelector - simplified from gateboxgo GetProviderForAccount
use std::sync::Arc;

use async_trait::async_trait;

use crate::accounts::AccountsRepository;
use crate::company::CompanyRepository;
use crate::customer::CustomerRepository;
use crate::fees::FeesRepository;
use crate::management::ManagementRepository;
use crate::model::{AdminData, AdminProviderInfo, CustomerData, CustomerProviderInfo, PartnersInfo, ProviderInfo};
use crate::partners::PartnersRepository;
use crate::partners_list::PartnersListRepository;

#[derive(Debug, thiserror::Error)]
pub enum ProviderSelectorError {
    #[error("Database: {0}")]
    Database(String),
    #[error("Provider not found: {0}")]
    NotFound(String),
}

impl From<crate::accounts::RepositoryError> for ProviderSelectorError {
    fn from(e: crate::accounts::RepositoryError) -> Self {
        ProviderSelectorError::Database(e.to_string())
    }
}
impl From<crate::customer::RepositoryError> for ProviderSelectorError {
    fn from(e: crate::customer::RepositoryError) -> Self {
        ProviderSelectorError::Database(e.to_string())
    }
}
impl From<crate::company::RepositoryError> for ProviderSelectorError {
    fn from(e: crate::company::RepositoryError) -> Self {
        ProviderSelectorError::Database(e.to_string())
    }
}
impl From<crate::management::RepositoryError> for ProviderSelectorError {
    fn from(e: crate::management::RepositoryError) -> Self {
        ProviderSelectorError::Database(e.to_string())
    }
}
impl From<crate::partners::RepositoryError> for ProviderSelectorError {
    fn from(e: crate::partners::RepositoryError) -> Self {
        ProviderSelectorError::Database(e.to_string())
    }
}
impl From<crate::partners_list::RepositoryError> for ProviderSelectorError {
    fn from(e: crate::partners_list::RepositoryError) -> Self {
        ProviderSelectorError::Database(e.to_string())
    }
}
impl From<crate::fees::RepositoryError> for ProviderSelectorError {
    fn from(e: crate::fees::RepositoryError) -> Self {
        ProviderSelectorError::Database(e.to_string())
    }
}

#[async_trait]
pub trait ProviderSelector: Send + Sync {
    async fn get_provider_for_account(&self, account_id: i64) -> Result<ProviderInfo, ProviderSelectorError>;
}

pub struct ProviderSelectorImpl {
    accounts_repo: Arc<dyn AccountsRepository>,
    customer_repo: Arc<dyn CustomerRepository>,
    company_repo: Arc<dyn CompanyRepository>,
    management_repo: Arc<dyn ManagementRepository>,
    partners_repo: Arc<dyn PartnersRepository>,
    partners_list_repo: Arc<dyn PartnersListRepository>,
    fees_repo: Arc<dyn FeesRepository>,
}

impl ProviderSelectorImpl {
    pub fn new(
        accounts_repo: Arc<dyn AccountsRepository>,
        customer_repo: Arc<dyn CustomerRepository>,
        company_repo: Arc<dyn CompanyRepository>,
        management_repo: Arc<dyn ManagementRepository>,
        partners_repo: Arc<dyn PartnersRepository>,
        partners_list_repo: Arc<dyn PartnersListRepository>,
        fees_repo: Arc<dyn FeesRepository>,
    ) -> Self {
        Self {
            accounts_repo,
            customer_repo,
            company_repo,
            management_repo,
            partners_repo,
            partners_list_repo,
            fees_repo,
        }
    }
}

#[async_trait]
impl ProviderSelector for ProviderSelectorImpl {
    async fn get_provider_for_account(&self, account_id: i64) -> Result<ProviderInfo, ProviderSelectorError> {
        let account = self
            .accounts_repo
            .get_by_id(account_id)
            .await?
            .ok_or_else(|| ProviderSelectorError::NotFound(format!("account {}", account_id)))?;

        let auth_id = account.authentication_id;

        let customer = self.customer_repo.get_by_authentication_id(auth_id).await?;
        let (customer_data, customer_acc_id, customer_rates, customer_person_type_id) = if let Some(ref c) = customer {
            let data = CustomerData {
                authentication_id: c.authentication_id,
                full_name: c.full_name.clone(),
                document_number: c.document_number.clone(),
                email: c.email.clone(),
                phone_number: c.phone_number.clone(),
            };
            // Prefer per-account fee; fall back to person-type default fee.
            let fees = match self.fees_repo.get_by_account_id(account_id).await? {
                Some(f) => Some(f),
                None => self.fees_repo.get_by_person_type(c.type_person_id).await?,
            };
            let (fix_in, fix_out, pct_in, pct_out, pct_sec, fix_ref_in, fix_ref_out, pct_ref_in, pct_ref_out) =
                fees.map(|f| {
                    (
                        f.fixed_cash_in.to_string().parse::<f64>().unwrap_or(0.0),
                        f.fixed_cash_out.to_string().parse::<f64>().unwrap_or(0.0),
                        f.percent_cashin.to_string().parse::<f64>().unwrap_or(0.0),
                        f.percent_cashout.to_string().parse::<f64>().unwrap_or(0.0),
                        f.percentsec_med.to_string().parse::<f64>().unwrap_or(0.0),
                        f.fixed_ref_cash_in.to_string().parse::<f64>().unwrap_or(0.0),
                        f.fixed_ref_cash_out.to_string().parse::<f64>().unwrap_or(0.0),
                        f.percent_ref_cashin.to_string().parse::<f64>().unwrap_or(0.0),
                        f.percent_ref_cashout.to_string().parse::<f64>().unwrap_or(0.0),
                    )
                }).unwrap_or((0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0));
            (data, account_id, (fix_in, fix_out, pct_in, pct_out, pct_sec, fix_ref_in, fix_ref_out, pct_ref_in, pct_ref_out), c.type_person_id)
        } else {
            (
                CustomerData {
                    authentication_id: 0,
                    full_name: String::new(),
                    document_number: String::new(),
                    email: String::new(),
                    phone_number: String::new(),
                },
                account_id,
                (0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0),
                0i64,
            )
        };

        let (admin_auth_id, admin_full_name, admin_doc, company_auth_id) = if let Some(ref c) = customer {
            let company = self.company_repo.get_by_id(c.company_id).await?;
            let auth_id = company.as_ref().map(|co| co.authentication_id).unwrap_or(auth_id);
            let management = self.management_repo.get_by_authentication_id(auth_id).await?;
            let (_mid, full_name, doc) = management
                .map(|m| (m.id, m.full_name.clone(), m.document_number.clone()))
                .unwrap_or((0, String::new(), String::new()));
            (auth_id, full_name, doc, auth_id)
        } else {
            (auth_id, String::new(), String::new(), auth_id)
        };

        let partners = self.partners_repo.get_by_authentication_id(company_auth_id).await?;
        let (partners_info, admin_account_id) = if let Some(ref p) = partners {
            let pl = self.partners_list_repo.get_by_id(p.partners_list_id).await?;
            let name = pl.as_ref().map(|x| x.description.clone()).unwrap_or_default();
            let admin_acc = self.accounts_repo.get_by_authentication_id(admin_auth_id).await?;
            let acc_id = admin_acc.map(|a| a.id).unwrap_or(account_id);
            (
                PartnersInfo {
                    id: p.id,
                    name,
                    client_id: Some(p.client_id.clone()).filter(|s| !s.is_empty()),
                    client_secret: Some(p.client_secret.clone()).filter(|s| !s.is_empty()),
                    fixed_cash_in: p.fixed_cash_in.to_string().parse().unwrap_or(0.0),
                    fixed_cash_out: p.fixed_cash_out.to_string().parse().unwrap_or(0.0),
                    percent_cashin: p.percent_cashin.to_string().parse().unwrap_or(0.0),
                    percent_cashout: p.percent_cashout.to_string().parse().unwrap_or(0.0),
                    fixed_ref_cash_in: p.fixed_ref_cash_in.to_string().parse().unwrap_or(0.0),
                    fixed_ref_cash_out: p.fixed_ref_cash_out.to_string().parse().unwrap_or(0.0),
                    percent_ref_cashin: p.percent_ref_cashin.to_string().parse().unwrap_or(0.0),
                    percent_ref_cashout: p.percent_ref_cashout.to_string().parse().unwrap_or(0.0),
                    active: p.active,
                },
                acc_id,
            )
        } else {
            (
                PartnersInfo {
                    id: 0,
                    name: String::new(),
                    client_id: None,
                    client_secret: None,
                    fixed_cash_in: 0.0,
                    fixed_cash_out: 0.0,
                    percent_cashin: 0.0,
                    percent_cashout: 0.0,
                    fixed_ref_cash_in: 0.0,
                    fixed_ref_cash_out: 0.0,
                    percent_ref_cashin: 0.0,
                    percent_ref_cashout: 0.0,
                    active: false,
                },
                account_id,
            )
        };

        let (fix_in, fix_out, pct_in, pct_out, pct_sec, fix_ref_in, fix_ref_out, pct_ref_in, pct_ref_out) = customer_rates;

        Ok(ProviderInfo {
            admin: AdminProviderInfo {
                fixed_cash_in: partners_info.fixed_cash_in,
                fixed_cash_out: partners_info.fixed_cash_out,
                percent_cashin: partners_info.percent_cashin,
                percent_cashout: partners_info.percent_cashout,
                fixed_ref_cashin: partners_info.fixed_ref_cash_in,
                fixed_ref_cashout: partners_info.fixed_ref_cash_out,
                percent_ref_cashin: partners_info.percent_ref_cashin,
                percent_ref_cashout: partners_info.percent_ref_cashout,
                data: AdminData {
                    authentication_id: admin_auth_id,
                    full_name: admin_full_name,
                    document_number: admin_doc,
                },
                account_id: admin_account_id,
                partners: partners_info,
            },
            customer: CustomerProviderInfo {
                data: customer_data,
                account_id: customer_acc_id,
                person_type_id: customer_person_type_id,
                fixed_cash_in: fix_in,
                fixed_cash_out: fix_out,
                percent_cashin: pct_in,
                percent_cashout: pct_out,
                percent_sec_med: pct_sec,
                fixed_ref_cash_in: fix_ref_in,
                fixed_ref_cashout: fix_ref_out,
                percent_ref_cashin: pct_ref_in,
                percent_ref_cashout: pct_ref_out,
            },
        })
    }
}
