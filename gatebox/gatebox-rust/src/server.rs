// Converted from gateboxgo/server/app.go - wires domains and runs HTTP server
use axum::{
    middleware,
    response::IntoResponse,
    routing::get,
    Json,
    Router,
};
use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
use serde_json::json;
use sqlx::PgPool;
use std::sync::Arc;
use std::time::Duration;
use tokio::signal;
use tower::ServiceBuilder;
use tower_http::cors::{Any, CorsLayer};
use tower_http::timeout::TimeoutLayer;
use tracing::info;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

use crate::metrics_server;
use crate::observabilidade;
use crate::openapi::ApiDoc;

use gatebox_rust::account_rules;
use gatebox_rust::account_rules::{AccountRulesRepositoryImpl, AccountRulesServiceImpl};
use gatebox_rust::account_status_types;
use gatebox_rust::account_status_types::{AccountStatusTypesRepositoryImpl, AccountStatusTypesServiceImpl};
use gatebox_rust::account_types;
use gatebox_rust::account_types::{AccountTypesRepositoryImpl, AccountTypesServiceImpl};
use gatebox_rust::accounts;
use gatebox_rust::accounts::{AccountsRepositoryImpl, AccountsServiceImpl};
use gatebox_rust::address;
use gatebox_rust::address::{AddressRepositoryImpl, AddressServiceImpl};
use gatebox_rust::address_types;
use gatebox_rust::address_types::{AddressTypesRepositoryImpl, AddressTypesServiceImpl};
use gatebox_rust::authentication;
use gatebox_rust::authentication::{AuthenticationRepositoryImpl, AuthenticationServiceImpl};
use gatebox_rust::company;
use gatebox_rust::company::{CompanyRepositoryImpl, CompanyServiceImpl};
use gatebox_rust::control_med;
use gatebox_rust::control_med::{ControlMedRepositoryImpl, ControlMedServiceImpl};
use gatebox_rust::customer;
use gatebox_rust::customer::{CustomerRepositoryImpl, CustomerServiceImpl};
use gatebox_rust::customer_status_types;
use gatebox_rust::customer_status_types::{CustomerStatusTypesRepositoryImpl, CustomerStatusTypesServiceImpl};
use gatebox_rust::fees;
use gatebox_rust::fees::{FeesRepositoryImpl, FeesServiceImpl};
use gatebox_rust::history_med;
use gatebox_rust::history_med::{HistoryMedRepositoryImpl, HistoryMedServiceImpl};
use gatebox_rust::invoice;
use gatebox_rust::invoice::{InvoiceRepositoryImpl, InvoiceServiceImpl};
use gatebox_rust::invoice_status_types;
use gatebox_rust::invoice_status_types::{InvoiceStatusTypesRepositoryImpl, InvoiceStatusTypesServiceImpl};
use gatebox_rust::invoice_types;
use gatebox_rust::invoice_types::{InvoiceTypesRepositoryImpl, InvoiceTypesServiceImpl};
use gatebox_rust::key_pix;
use gatebox_rust::key_pix::{KeyPixRepositoryImpl, KeyPixServiceImpl};
use gatebox_rust::key_pix_cache;
use gatebox_rust::key_pix_cache::{KeyPixCacheRepositoryImpl, KeyPixCacheServiceImpl};
use gatebox_rust::kyc_risk_types;
use gatebox_rust::kyc_risk_types::{KycRiskTypesRepositoryImpl, KycRiskTypesServiceImpl};
use gatebox_rust::management;
use gatebox_rust::management::{ManagementRepositoryImpl, ManagementServiceImpl};
use gatebox_rust::partners;
use gatebox_rust::partners::{PartnersRepositoryImpl, PartnersServiceImpl};
use gatebox_rust::partners_list;
use gatebox_rust::partners_list::{PartnersListRepositoryImpl, PartnersListServiceImpl};
use gatebox_rust::pix_key_types;
use gatebox_rust::pix_key_types::{PixKeyTypesRepositoryImpl, PixKeyTypesServiceImpl};
use gatebox_rust::status_controle_med_types;
use gatebox_rust::status_controle_med_types::{StatusControleMedTypesRepositoryImpl, StatusControleMedTypesServiceImpl};
use gatebox_rust::sec_med;
use gatebox_rust::sec_med::{SecMedRepositoryImpl, SecMedServiceImpl};
use gatebox_rust::shared_key;
use gatebox_rust::shared_key::{SharedKeyRepositoryImpl, SharedKeyServiceImpl};
use gatebox_rust::status_sec_med_types;
use gatebox_rust::status_sec_med_types::{StatusSecMedTypesRepositoryImpl, StatusSecMedTypesServiceImpl};
use gatebox_rust::status_transaction_types;
use gatebox_rust::status_transaction_types::{StatusTransactionTypesRepositoryImpl, StatusTransactionTypesServiceImpl};
use gatebox_rust::styled;
use gatebox_rust::styled::{StyledRepositoryImpl, StyledServiceImpl};
use gatebox_rust::styled_types;
use gatebox_rust::styled_types::{StyledTypesRepositoryImpl, StyledTypesServiceImpl};
use gatebox_rust::sub_type_transaction_types;
use gatebox_rust::sub_type_transaction_types::{SubTypeTransactionTypesRepositoryImpl, SubTypeTransactionTypesServiceImpl};
use gatebox_rust::type_auth_types;
use gatebox_rust::type_auth_types::{TypeAuthTypesRepositoryImpl, TypeAuthTypesServiceImpl};
use gatebox_rust::type_authorize_types;
use gatebox_rust::type_authorize_types::{TypeAuthorizeTypesRepositoryImpl, TypeAuthorizeTypesServiceImpl};
use gatebox_rust::type_external_types;
use gatebox_rust::type_external_types::{TypeExternalTypesRepositoryImpl, TypeExternalTypesServiceImpl};
use gatebox_rust::type_person_types;
use gatebox_rust::type_person_types::{TypePersonTypesRepositoryImpl, TypePersonTypesServiceImpl};
use gatebox_rust::token_service;
use gatebox_rust::token_service::{TokenServiceRepositoryImpl, TokenServiceServiceImpl};
use gatebox_rust::transaction;
use gatebox_rust::transaction::{TransactionRepositoryImpl, TransactionServiceImpl};
use gatebox_rust::type_transaction_types;
use gatebox_rust::type_transaction_types::{TypeTransactionTypesRepositoryImpl, TypeTransactionTypesServiceImpl};
use gatebox_rust::webhook_manager;
use gatebox_rust::webhook_manager::{WebhookManagerRepositoryImpl, WebhookManagerServiceImpl};
use gatebox_rust::webhook_types;
use gatebox_rust::webhook_types::{WebhookTypesRepositoryImpl, WebhookTypesServiceImpl};
use gatebox_rust::with_list_accounts;
use gatebox_rust::with_list_accounts::{WithListAccountsRepositoryImpl, WithListAccountsServiceImpl};
use gatebox_rust::modules::{admin, backoffice};

use gatebox_rust::core::messaging::{PaymentMessageHandler, PaymentMessageHandlerStub, PulsarPaymentPublisher, RabbitMQPaymentPublisher};
use gatebox_rust::core::pix_principal::PixPrincipalServiceAsync;
use gatebox_rust::core::pulsar::{Config as PulsarConfig, ProducerPool as PulsarProducerPool, ResilientConsumer};
use gatebox_rust::core::rabbitmq::{ProducerPool as RabbitMQProducerPool, ProducerPoolConfig as RabbitMQProducerPoolConfig, RabbitMQConfig, WorkerPool, WorkerPoolConfig};

pub struct App {
    read_pool: Option<Arc<PgPool>>,
    write_pool: Option<Arc<PgPool>>,
}

impl App {
    pub fn new() -> Self {
        Self {
            read_pool: None,
            write_pool: None,
        }
    }

    pub async fn start(&mut self) -> anyhow::Result<()> {
        let write_url = std::env::var("POSTGRESQL_WRITE_URL")
            .unwrap_or_else(|_| "postgres://localhost/gatebox".to_string());
        let read_url = std::env::var("POSTGRESQL_READ_URL")
            .unwrap_or_else(|_| "postgres://localhost/gatebox".to_string());

        let write_pool = PgPool::connect(&write_url).await?;
        let read_pool = PgPool::connect(&read_url).await?;

        // Run migrations (gateway_failover, etc.)
        let migrator = sqlx::migrate!("./migrations");
        migrator.run(&write_pool).await?;

        write_pool.acquire().await?;
        read_pool.acquire().await?;

        let write_pool = Arc::new(write_pool);
        let read_pool = Arc::new(read_pool);

        self.write_pool = Some(write_pool.clone());
        self.read_pool = Some(read_pool.clone());

        info!("Database connected");
        Ok(())
    }

    pub async fn run(&mut self, port: &str) -> anyhow::Result<()> {
        let read_pool = self.read_pool.clone().expect("call start() first");
        let write_pool = self.write_pool.clone().expect("call start() first");

        let accounts_repo: Arc<dyn accounts::AccountsRepository> =
            Arc::new(AccountsRepositoryImpl::new(read_pool.clone(), write_pool.clone()));
        let accounts_svc: Arc<dyn accounts::AccountsService> =
            Arc::new(AccountsServiceImpl::new(accounts_repo.clone()));

        let account_rules_repo: Arc<dyn account_rules::AccountRulesRepository> =
            Arc::new(AccountRulesRepositoryImpl::new(read_pool.clone(), write_pool.clone()));
        let account_rules_svc: Arc<dyn account_rules::AccountRulesService> =
            Arc::new(AccountRulesServiceImpl::new(account_rules_repo.clone()));

        let account_types_repo: Arc<dyn account_types::AccountTypesRepository> =
            Arc::new(AccountTypesRepositoryImpl::new(read_pool.clone(), write_pool.clone()));
        let account_types_svc: Arc<dyn account_types::AccountTypesService> =
            Arc::new(AccountTypesServiceImpl::new(account_types_repo));

        let customer_repo: Arc<dyn customer::CustomerRepository> =
            Arc::new(CustomerRepositoryImpl::new(read_pool.clone(), write_pool.clone()));
        let customer_svc: Arc<dyn customer::CustomerService> =
            Arc::new(CustomerServiceImpl::new(customer_repo.clone()));

        let account_status_types_repo: Arc<dyn account_status_types::AccountStatusTypesRepository> =
            Arc::new(AccountStatusTypesRepositoryImpl::new(read_pool.clone(), write_pool.clone()));
        let account_status_types_svc: Arc<dyn account_status_types::AccountStatusTypesService> =
            Arc::new(AccountStatusTypesServiceImpl::new(account_status_types_repo));

        let address_types_repo: Arc<dyn address_types::AddressTypesRepository> =
            Arc::new(AddressTypesRepositoryImpl::new(read_pool.clone(), write_pool.clone()));
        let address_types_svc: Arc<dyn address_types::AddressTypesService> =
            Arc::new(AddressTypesServiceImpl::new(address_types_repo));

        let address_repo: Arc<dyn address::AddressRepository> =
            Arc::new(AddressRepositoryImpl::new(read_pool.clone(), write_pool.clone()));
        let address_svc: Arc<dyn address::AddressService> =
            Arc::new(AddressServiceImpl::new(address_repo));

        let authentication_repo: Arc<dyn authentication::AuthenticationRepository> =
            Arc::new(AuthenticationRepositoryImpl::new(read_pool.clone(), write_pool.clone()));
        let authentication_svc: Arc<dyn authentication::AuthenticationService> =
            Arc::new(AuthenticationServiceImpl::new(authentication_repo));

        let company_repo: Arc<dyn company::CompanyRepository> =
            Arc::new(CompanyRepositoryImpl::new(read_pool.clone(), write_pool.clone()));
        let company_svc: Arc<dyn company::CompanyService> =
            Arc::new(CompanyServiceImpl::new(company_repo.clone()));

        let control_med_repo: Arc<dyn control_med::ControlMedRepository> =
            Arc::new(ControlMedRepositoryImpl::new(read_pool.clone(), write_pool.clone()));
        let control_med_svc: Arc<dyn control_med::ControlMedService> =
            Arc::new(ControlMedServiceImpl::new(control_med_repo));

        let management_repo: Arc<dyn management::ManagementRepository> =
            Arc::new(ManagementRepositoryImpl::new(read_pool.clone(), write_pool.clone()));
        let management_svc: Arc<dyn management::ManagementService> =
            Arc::new(ManagementServiceImpl::new(management_repo.clone()));

        let customer_status_types_repo: Arc<dyn customer_status_types::CustomerStatusTypesRepository> =
            Arc::new(CustomerStatusTypesRepositoryImpl::new(read_pool.clone(), write_pool.clone()));
        let customer_status_types_svc: Arc<dyn customer_status_types::CustomerStatusTypesService> =
            Arc::new(CustomerStatusTypesServiceImpl::new(customer_status_types_repo));

        let fees_repo: Arc<dyn fees::FeesRepository> =
            Arc::new(FeesRepositoryImpl::new(read_pool.clone(), write_pool.clone()));
        let fees_svc: Arc<dyn fees::FeesService> =
            Arc::new(FeesServiceImpl::new(fees_repo.clone()));

        let history_med_repo: Arc<dyn history_med::HistoryMedRepository> =
            Arc::new(HistoryMedRepositoryImpl::new(read_pool.clone(), write_pool.clone()));
        let history_med_svc: Arc<dyn history_med::HistoryMedService> =
            Arc::new(HistoryMedServiceImpl::new(history_med_repo));

        let invoice_repo: Arc<dyn invoice::InvoiceRepository> =
            Arc::new(InvoiceRepositoryImpl::new(read_pool.clone(), write_pool.clone()));
        let invoice_svc: Arc<dyn invoice::InvoiceService> =
            Arc::new(InvoiceServiceImpl::new(invoice_repo.clone()));

        let invoice_status_types_repo: Arc<dyn invoice_status_types::InvoiceStatusTypesRepository> =
            Arc::new(InvoiceStatusTypesRepositoryImpl::new(read_pool.clone(), write_pool.clone()));
        let invoice_status_types_svc: Arc<dyn invoice_status_types::InvoiceStatusTypesService> =
            Arc::new(InvoiceStatusTypesServiceImpl::new(invoice_status_types_repo));

        let invoice_types_repo: Arc<dyn invoice_types::InvoiceTypesRepository> =
            Arc::new(InvoiceTypesRepositoryImpl::new(read_pool.clone(), write_pool.clone()));
        let invoice_types_svc: Arc<dyn invoice_types::InvoiceTypesService> =
            Arc::new(InvoiceTypesServiceImpl::new(invoice_types_repo));

        let kyc_risk_types_repo: Arc<dyn kyc_risk_types::KycRiskTypesRepository> =
            Arc::new(KycRiskTypesRepositoryImpl::new(read_pool.clone(), write_pool.clone()));
        let kyc_risk_types_svc: Arc<dyn kyc_risk_types::KycRiskTypesService> =
            Arc::new(KycRiskTypesServiceImpl::new(kyc_risk_types_repo));

        let partners_repo: Arc<dyn partners::PartnersRepository> =
            Arc::new(PartnersRepositoryImpl::new(read_pool.clone(), write_pool.clone()));
        let partners_svc: Arc<dyn partners::PartnersService> =
            Arc::new(PartnersServiceImpl::new(partners_repo.clone()));

        let partners_list_repo: Arc<dyn partners_list::PartnersListRepository> =
            Arc::new(PartnersListRepositoryImpl::new(read_pool.clone(), write_pool.clone()));
        let partners_list_svc: Arc<dyn partners_list::PartnersListService> =
            Arc::new(PartnersListServiceImpl::new(partners_list_repo.clone()));

        let pix_key_types_repo: Arc<dyn pix_key_types::PixKeyTypesRepository> =
            Arc::new(PixKeyTypesRepositoryImpl::new(read_pool.clone(), write_pool.clone()));
        let pix_key_types_svc: Arc<dyn pix_key_types::PixKeyTypesService> =
            Arc::new(PixKeyTypesServiceImpl::new(pix_key_types_repo));

        let key_pix_repo: Arc<dyn key_pix::KeyPixRepository> =
            Arc::new(KeyPixRepositoryImpl::new(read_pool.clone(), write_pool.clone()));
        let key_pix_svc: Arc<dyn key_pix::KeyPixService> =
            Arc::new(KeyPixServiceImpl::new(key_pix_repo.clone()));

        let key_pix_cache_repo: Arc<dyn key_pix_cache::KeyPixCacheRepository> =
            Arc::new(KeyPixCacheRepositoryImpl::new(read_pool.clone(), write_pool.clone()));
        let key_pix_cache_svc: Arc<dyn key_pix_cache::KeyPixCacheService> =
            Arc::new(KeyPixCacheServiceImpl::new(key_pix_cache_repo));

        let status_controle_med_types_repo: Arc<dyn status_controle_med_types::StatusControleMedTypesRepository> =
            Arc::new(StatusControleMedTypesRepositoryImpl::new(read_pool.clone(), write_pool.clone()));
        let status_controle_med_types_svc: Arc<dyn status_controle_med_types::StatusControleMedTypesService> =
            Arc::new(StatusControleMedTypesServiceImpl::new(status_controle_med_types_repo));

        let sec_med_repo: Arc<dyn sec_med::SecMedRepository> =
            Arc::new(SecMedRepositoryImpl::new(read_pool.clone(), write_pool.clone()));
        let sec_med_svc: Arc<dyn sec_med::SecMedService> =
            Arc::new(SecMedServiceImpl::new(sec_med_repo.clone()));

        let shared_key_repo: Arc<dyn shared_key::SharedKeyRepository> =
            Arc::new(SharedKeyRepositoryImpl::new(read_pool.clone(), write_pool.clone()));
        let shared_key_svc: Arc<dyn shared_key::SharedKeyService> =
            Arc::new(SharedKeyServiceImpl::new(shared_key_repo));

        let status_sec_med_types_repo: Arc<dyn status_sec_med_types::StatusSecMedTypesRepository> =
            Arc::new(StatusSecMedTypesRepositoryImpl::new(read_pool.clone(), write_pool.clone()));
        let status_sec_med_types_svc: Arc<dyn status_sec_med_types::StatusSecMedTypesService> =
            Arc::new(StatusSecMedTypesServiceImpl::new(status_sec_med_types_repo));

        let status_transaction_types_repo: Arc<dyn status_transaction_types::StatusTransactionTypesRepository> =
            Arc::new(StatusTransactionTypesRepositoryImpl::new(read_pool.clone(), write_pool.clone()));
        let status_transaction_types_svc: Arc<dyn status_transaction_types::StatusTransactionTypesService> =
            Arc::new(StatusTransactionTypesServiceImpl::new(status_transaction_types_repo));

        let styled_repo: Arc<dyn styled::StyledRepository> =
            Arc::new(StyledRepositoryImpl::new(read_pool.clone(), write_pool.clone()));
        let styled_svc: Arc<dyn styled::StyledService> =
            Arc::new(StyledServiceImpl::new(styled_repo));

        let styled_types_repo: Arc<dyn styled_types::StyledTypesRepository> =
            Arc::new(StyledTypesRepositoryImpl::new(read_pool.clone(), write_pool.clone()));
        let styled_types_svc: Arc<dyn styled_types::StyledTypesService> =
            Arc::new(StyledTypesServiceImpl::new(styled_types_repo));

        let transaction_repo: Arc<dyn transaction::TransactionRepository> =
            Arc::new(TransactionRepositoryImpl::new(read_pool.clone(), write_pool.clone()));
        let transaction_svc: Arc<dyn transaction::TransactionService> =
            Arc::new(TransactionServiceImpl::new(transaction_repo.clone()));

        let token_service_repo: Arc<dyn token_service::TokenServiceRepository> =
            Arc::new(TokenServiceRepositoryImpl::new(read_pool.clone(), write_pool.clone()));
        let token_service_svc: Arc<dyn token_service::TokenServiceService> =
            Arc::new(TokenServiceServiceImpl::new(token_service_repo));

        let sub_type_transaction_types_repo: Arc<dyn sub_type_transaction_types::SubTypeTransactionTypesRepository> =
            Arc::new(SubTypeTransactionTypesRepositoryImpl::new(read_pool.clone(), write_pool.clone()));
        let sub_type_transaction_types_svc: Arc<dyn sub_type_transaction_types::SubTypeTransactionTypesService> =
            Arc::new(SubTypeTransactionTypesServiceImpl::new(sub_type_transaction_types_repo));

        let type_auth_types_repo: Arc<dyn type_auth_types::TypeAuthTypesRepository> =
            Arc::new(TypeAuthTypesRepositoryImpl::new(read_pool.clone(), write_pool.clone()));
        let type_auth_types_svc: Arc<dyn type_auth_types::TypeAuthTypesService> =
            Arc::new(TypeAuthTypesServiceImpl::new(type_auth_types_repo));

        let type_authorize_types_repo: Arc<dyn type_authorize_types::TypeAuthorizeTypesRepository> =
            Arc::new(TypeAuthorizeTypesRepositoryImpl::new(read_pool.clone(), write_pool.clone()));
        let type_authorize_types_svc: Arc<dyn type_authorize_types::TypeAuthorizeTypesService> =
            Arc::new(TypeAuthorizeTypesServiceImpl::new(type_authorize_types_repo));

        let type_external_types_repo: Arc<dyn type_external_types::TypeExternalTypesRepository> =
            Arc::new(TypeExternalTypesRepositoryImpl::new(read_pool.clone(), write_pool.clone()));
        let type_external_types_svc: Arc<dyn type_external_types::TypeExternalTypesService> =
            Arc::new(TypeExternalTypesServiceImpl::new(type_external_types_repo));

        let type_person_types_repo: Arc<dyn type_person_types::TypePersonTypesRepository> =
            Arc::new(TypePersonTypesRepositoryImpl::new(read_pool.clone(), write_pool.clone()));
        let type_person_types_svc: Arc<dyn type_person_types::TypePersonTypesService> =
            Arc::new(TypePersonTypesServiceImpl::new(type_person_types_repo));

        let type_transaction_types_repo: Arc<dyn type_transaction_types::TypeTransactionTypesRepository> =
            Arc::new(TypeTransactionTypesRepositoryImpl::new(read_pool.clone(), write_pool.clone()));
        let type_transaction_types_svc: Arc<dyn type_transaction_types::TypeTransactionTypesService> =
            Arc::new(TypeTransactionTypesServiceImpl::new(type_transaction_types_repo));

        let webhook_manager_repo: Arc<dyn webhook_manager::WebhookManagerRepository> =
            Arc::new(WebhookManagerRepositoryImpl::new(read_pool.clone(), write_pool.clone()));
        let webhook_manager_svc: Arc<dyn webhook_manager::WebhookManagerService> =
            Arc::new(WebhookManagerServiceImpl::new(webhook_manager_repo));

        let webhook_types_repo: Arc<dyn webhook_types::WebhookTypesRepository> =
            Arc::new(WebhookTypesRepositoryImpl::new(read_pool.clone(), write_pool.clone()));
        let webhook_types_svc: Arc<dyn webhook_types::WebhookTypesService> =
            Arc::new(WebhookTypesServiceImpl::new(webhook_types_repo));

        let with_list_accounts_repo: Arc<dyn with_list_accounts::WithListAccountsRepository> =
            Arc::new(WithListAccountsRepositoryImpl::new(read_pool.clone(), write_pool.clone()));
        let with_list_accounts_svc: Arc<dyn with_list_accounts::WithListAccountsService> =
            Arc::new(WithListAccountsServiceImpl::new(with_list_accounts_repo.clone()));

        let dispute_repo: Arc<dyn gatebox_rust::disputes::DisputeRepository> =
            Arc::new(gatebox_rust::disputes::DisputeRepositoryImpl::new(read_pool.clone(), write_pool.clone()));
        let dispute_state = gatebox_rust::disputes::DisputeState { repo: dispute_repo };

        let app_log_repo: Arc<dyn gatebox_rust::app_log::AppLogRepository> =
            Arc::new(gatebox_rust::app_log::AppLogRepositoryImpl::new(read_pool.clone(), write_pool.clone()));
        let hospital_message_repo: Arc<dyn gatebox_rust::hospital_message::HospitalMessageRepository> =
            Arc::new(gatebox_rust::hospital_message::HospitalMessageRepositoryImpl::new(write_pool.clone()));
        let backoffice_state = backoffice::BackofficeState {
            accounts_svc: accounts_svc.clone(),
            app_log_repo: Some(app_log_repo.clone()),
        };

        // MESSAGING_BACKEND: pulsar (default) ou rabbitmq
        let messaging_backend = std::env::var("MESSAGING_BACKEND")
            .unwrap_or_else(|_| "pulsar".to_string())
            .to_lowercase();
        let has_sulcred = std::env::var("PIX_GATEWAY_SULCRED").is_ok();
        let sulcred_client_id = std::env::var("SULCRED_CLIENT_ID").unwrap_or_default();
        let sulcred_client_secret = std::env::var("SULCRED_CLIENT_SECRET").unwrap_or_default();
        let has_seventrust = std::env::var("PIX_GATEWAY_SEVENTRUST").is_ok();
        let seventrust_client_id = std::env::var("SEVENTRUST_CLIENT_ID").unwrap_or_default();
        let seventrust_client_secret = std::env::var("SEVENTRUST_CLIENT_SECRET").unwrap_or_default();
        let has_next = std::env::var("PIX_GATEWAY_NEXT").is_ok();
        let next_client_id = std::env::var("NEXT_CLIENT_ID").unwrap_or_default();
        let next_client_secret = std::env::var("NEXT_CLIENT_SECRET").unwrap_or_default();
        let default_partners_id: i64 = std::env::var("DEFAULT_PARTNERS_ID").ok().and_then(|s| s.parse().ok()).unwrap_or(1);
        let gateway_recorder: Arc<dyn gatebox_rust::core::gateway_failover::GatewayRecorder> =
            Arc::new(gatebox_rust::core::gateway_failover::GatewayRecorderImpl::new(write_pool.clone()));
        let gateway_selector: Arc<dyn gatebox_rust::core::gateway_failover::GatewaySelector> =
            Arc::new(gatebox_rust::core::gateway_failover::GatewaySelectorImpl::new(write_pool.clone()));
        // Prefer SevenTrust > Next > Sulcred
        let (gateway_name, has_gateway) = if has_seventrust {
            ("seventrust".to_string(), true)
        } else if has_next {
            ("next".to_string(), true)
        } else if has_sulcred {
            ("sulcred".to_string(), true)
        } else {
            ("sulcred".to_string(), false)
        };
        let _health_check_worker = if has_gateway {
            Some(gatebox_rust::core::gateway_failover::HealthCheckWorker::new(
                write_pool.clone(),
                120,
            ))
        } else {
            None
        };

        let (pix_principal_svc, rabbitmq_worker, pulsar_cancel_tx): (
            Arc<dyn gatebox_rust::core::pix_principal::PixPrincipalService>,
            Option<WorkerPool>,
            Option<tokio::sync::oneshot::Sender<()>>,
        ) = if messaging_backend == "rabbitmq" && std::env::var("RBMQ_URI").is_ok() {
            let uri = std::env::var("RBMQ_URI").unwrap_or_else(|_| "amqp://localhost:5672/%2f".to_string());
            let rb_config = RabbitMQConfig {
                uri: uri.clone(),
                queue_name: gatebox_rust::core::rabbitmq::QUEUE_NAME_PAYMENT.to_string(),
                reconnect_delay: std::time::Duration::from_secs(5),
            };
            let producer_pool = Arc::new(RabbitMQProducerPool::new(rb_config.clone(), RabbitMQProducerPoolConfig::default()));
            if producer_pool.start().await.is_err() {
                tracing::warn!("RabbitMQ ProducerPool failed to start");
                let fallback = if has_gateway {
                    let (gw, cid, csec) = if has_seventrust {
                        (Arc::new(gatebox_rust::core::gateways::services::SevenTrustHttpService::default()) as Arc<dyn gatebox_rust::core::gateways::services::GatewayHttpService>,
                         seventrust_client_id, seventrust_client_secret)
                    } else if has_next {
                        (Arc::new(gatebox_rust::core::gateways::services::NextHttpService::default()) as Arc<dyn gatebox_rust::core::gateways::services::GatewayHttpService>,
                         next_client_id, next_client_secret)
                    } else {
                        (Arc::new(gatebox_rust::core::gateways::services::SulcredHttpService::default()) as Arc<dyn gatebox_rust::core::gateways::services::GatewayHttpService>,
                         sulcred_client_id, sulcred_client_secret)
                    };
                    Arc::new(gatebox_rust::core::pix_principal::PixPrincipalServiceImpl::new(gw, cid, csec)) as Arc<dyn gatebox_rust::core::pix_principal::PixPrincipalService>
                } else {
                    Arc::new(gatebox_rust::core::pix_principal::PixPrincipalServiceStub)
                };
                let handler = Arc::new(PaymentMessageHandlerStub::new()) as Arc<dyn gatebox_rust::core::messaging::MessageHandler>;
                let pool = WorkerPool::new(WorkerPoolConfig::default(), handler).with_config(rb_config);
                let _ = pool.start().await;
                (fallback, Some(pool), None)
            } else {
                let publisher = Arc::new(RabbitMQPaymentPublisher::new(producer_pool)) as Arc<dyn gatebox_rust::core::messaging::PaymentPublisher>;
                let async_svc = Arc::new(PixPrincipalServiceAsync::new(
                    transaction_repo.clone(),
                    accounts_repo.clone(),
                    account_rules_repo.clone(),
                    with_list_accounts_repo.clone(),
                    fees_repo.clone(),
                    publisher,
                    gateway_name.clone(),
                    default_partners_id,
                ));
                let handler: Arc<dyn gatebox_rust::core::messaging::MessageHandler> = if has_gateway {
                    let (gw, cid, csec, gw_name) = if has_seventrust {
                        (Arc::new(gatebox_rust::core::gateways::services::SevenTrustHttpService::default()) as Arc<dyn gatebox_rust::core::gateways::services::GatewayHttpService>,
                         seventrust_client_id.clone(), seventrust_client_secret.clone(), "seventrust".to_string())
                    } else if has_next {
                        (Arc::new(gatebox_rust::core::gateways::services::NextHttpService::default()) as Arc<dyn gatebox_rust::core::gateways::services::GatewayHttpService>,
                         next_client_id.clone(), next_client_secret.clone(), "next".to_string())
                    } else {
                        (Arc::new(gatebox_rust::core::gateways::services::SulcredHttpService::default()) as Arc<dyn gatebox_rust::core::gateways::services::GatewayHttpService>,
                         sulcred_client_id.clone(), sulcred_client_secret.clone(), "sulcred".to_string())
                    };
                    let mut h = PaymentMessageHandler::new(
                        transaction_repo.clone(),
                        gw,
                        cid,
                        csec,
                        gw_name.clone(),
                    )
                    .with_gateway_recorder(gateway_recorder.clone())
                    .with_gateway_selector(gateway_selector.clone());
                    if has_seventrust && has_sulcred {
                        let (gw2, cid2, csec2, gw_name2) = if gw_name == "seventrust" {
                            (Arc::new(gatebox_rust::core::gateways::services::SulcredHttpService::default()) as Arc<dyn gatebox_rust::core::gateways::services::GatewayHttpService>,
                             sulcred_client_id.clone(), sulcred_client_secret.clone(), "sulcred".to_string())
                        } else {
                            (Arc::new(gatebox_rust::core::gateways::services::SevenTrustHttpService::default()) as Arc<dyn gatebox_rust::core::gateways::services::GatewayHttpService>,
                             seventrust_client_id.clone(), seventrust_client_secret.clone(), "seventrust".to_string())
                        };
                        h = h.with_fallback_gateway(gw2, cid2, csec2, gw_name2);
                    }
                    Arc::new(h)
                } else {
                    Arc::new(PaymentMessageHandlerStub::new())
                };
                let pool = WorkerPool::new(WorkerPoolConfig::default(), handler).with_config(rb_config);
                if pool.start().await.is_ok() {
                    info!("RabbitMQ WorkerPool + ProducerPool started (async SendPix)");
                }
                (async_svc, Some(pool), None)
            }
        } else if messaging_backend == "pulsar" && std::env::var("PULSAR_URL").is_ok() {
            let (pix_svc, rabbitmq_worker, pulsar_tx) = {
                let config = PulsarConfig::default();
                let producer_pool = Arc::new(PulsarProducerPool::new(config.clone()));
                if producer_pool.start().await.is_err() {
                    tracing::warn!("Pulsar ProducerPool failed to start");
                    let fallback = if has_gateway {
                        let (gw, cid, csec) = if has_seventrust {
                            (Arc::new(gatebox_rust::core::gateways::services::SevenTrustHttpService::default()) as Arc<dyn gatebox_rust::core::gateways::services::GatewayHttpService>,
                             seventrust_client_id, seventrust_client_secret)
                        } else if has_next {
                            (Arc::new(gatebox_rust::core::gateways::services::NextHttpService::default()) as Arc<dyn gatebox_rust::core::gateways::services::GatewayHttpService>,
                             next_client_id, next_client_secret)
                        } else {
                            (Arc::new(gatebox_rust::core::gateways::services::SulcredHttpService::default()) as Arc<dyn gatebox_rust::core::gateways::services::GatewayHttpService>,
                             sulcred_client_id, sulcred_client_secret)
                        };
                        Arc::new(gatebox_rust::core::pix_principal::PixPrincipalServiceImpl::new(gw, cid, csec)) as Arc<dyn gatebox_rust::core::pix_principal::PixPrincipalService>
                    } else {
                        Arc::new(gatebox_rust::core::pix_principal::PixPrincipalServiceStub)
                    };
                    let (tx, rx) = tokio::sync::oneshot::channel();
                    let handler = Arc::new(PaymentMessageHandlerStub::new()) as Arc<dyn gatebox_rust::core::messaging::MessageHandler>;
                    let consumer = ResilientConsumer::new(config.clone(), handler);
                    tokio::spawn(async move { let _ = consumer.run(rx).await; });
                    (fallback, None, Some(tx))
                } else {
                    let publisher = Arc::new(PulsarPaymentPublisher::new(producer_pool)) as Arc<dyn gatebox_rust::core::messaging::PaymentPublisher>;
                    let async_svc = Arc::new(PixPrincipalServiceAsync::new(
                        transaction_repo.clone(),
                        accounts_repo.clone(),
                        account_rules_repo.clone(),
                        with_list_accounts_repo.clone(),
                        fees_repo.clone(),
                        publisher,
                        gateway_name.clone(),
                        default_partners_id,
                    ));
                    let handler: Arc<dyn gatebox_rust::core::messaging::MessageHandler> = if has_gateway {
                    let (gw, cid, csec, gw_name) = if has_seventrust {
                        (Arc::new(gatebox_rust::core::gateways::services::SevenTrustHttpService::default()) as Arc<dyn gatebox_rust::core::gateways::services::GatewayHttpService>,
                         seventrust_client_id.clone(), seventrust_client_secret.clone(), "seventrust".to_string())
                    } else if has_next {
                        (Arc::new(gatebox_rust::core::gateways::services::NextHttpService::default()) as Arc<dyn gatebox_rust::core::gateways::services::GatewayHttpService>,
                         next_client_id.clone(), next_client_secret.clone(), "next".to_string())
                    } else {
                        (Arc::new(gatebox_rust::core::gateways::services::SulcredHttpService::default()) as Arc<dyn gatebox_rust::core::gateways::services::GatewayHttpService>,
                         sulcred_client_id.clone(), sulcred_client_secret.clone(), "sulcred".to_string())
                    };
                    let mut h = PaymentMessageHandler::new(
                            transaction_repo.clone(),
                            gw,
                            cid,
                            csec,
                            gw_name.clone(),
                        )
                        .with_gateway_recorder(gateway_recorder.clone())
                        .with_gateway_selector(gateway_selector.clone());
                        if has_seventrust && has_sulcred {
                            let (gw2, cid2, csec2, gw_name2) = if gw_name == "seventrust" {
                                (Arc::new(gatebox_rust::core::gateways::services::SulcredHttpService::default()) as Arc<dyn gatebox_rust::core::gateways::services::GatewayHttpService>,
                                 sulcred_client_id.clone(), sulcred_client_secret.clone(), "sulcred".to_string())
                            } else {
                                (Arc::new(gatebox_rust::core::gateways::services::SevenTrustHttpService::default()) as Arc<dyn gatebox_rust::core::gateways::services::GatewayHttpService>,
                                 seventrust_client_id.clone(), seventrust_client_secret.clone(), "seventrust".to_string())
                            };
                            h = h.with_fallback_gateway(gw2, cid2, csec2, gw_name2);
                        }
                        Arc::new(h)
                    } else {
                        Arc::new(PaymentMessageHandlerStub::new())
                    };
                    let (tx, rx) = tokio::sync::oneshot::channel();
                    let consumer = ResilientConsumer::new(config.clone(), handler);
                    tokio::spawn(async move {
                        if let Err(e) = consumer.run(rx).await {
                            tracing::error!("Pulsar ResilientConsumer error: {}", e);
                        }
                    });
                    info!("Pulsar ResilientConsumer + ProducerPool started (async SendPix)");
                    let pix_svc: Arc<dyn gatebox_rust::core::pix_principal::PixPrincipalService> = async_svc;
                    (pix_svc, None, Some(tx))
                }
            };
            (pix_svc, rabbitmq_worker, pulsar_tx)
        } else {
            let sync_svc = if has_gateway {
                let (gw, cid, csec) = if has_seventrust {
                    (Arc::new(gatebox_rust::core::gateways::services::SevenTrustHttpService::default()) as Arc<dyn gatebox_rust::core::gateways::services::GatewayHttpService>,
                     seventrust_client_id, seventrust_client_secret)
                } else if has_next {
                    (Arc::new(gatebox_rust::core::gateways::services::NextHttpService::default()) as Arc<dyn gatebox_rust::core::gateways::services::GatewayHttpService>,
                     next_client_id, next_client_secret)
                } else {
                    (Arc::new(gatebox_rust::core::gateways::services::SulcredHttpService::default()) as Arc<dyn gatebox_rust::core::gateways::services::GatewayHttpService>,
                     sulcred_client_id, sulcred_client_secret)
                };
                Arc::new(gatebox_rust::core::pix_principal::PixPrincipalServiceImpl::new(gw, cid, csec)) as Arc<dyn gatebox_rust::core::pix_principal::PixPrincipalService>
            } else {
                Arc::new(gatebox_rust::core::pix_principal::PixPrincipalServiceStub)
            };
            (sync_svc, None::<WorkerPool>, None)
        };

        // Hospital consumer: persiste mensagens falhadas em DB quando RBMQ_URI está definido
        let hospital_cancel_tx: Option<tokio::sync::oneshot::Sender<()>> = if std::env::var("RBMQ_URI").is_ok() {
            let (tx, rx) = tokio::sync::oneshot::channel();
            let repo = Some(hospital_message_repo.clone());
            tokio::spawn(async move {
                if let Err(e) = gatebox_rust::rabbitmq_dead_letter::run_hospital_consumer(rx, repo).await {
                    tracing::error!("Hospital consumer error: {}", e);
                }
            });
            info!("Hospital consumer started (persisting to hospital_message table)");
            Some(tx)
        } else {
            None
        };

        let provider_selector: Arc<dyn gatebox_rust::core::pix_principal::ProviderSelector> =
            Arc::new(gatebox_rust::core::pix_principal::ProviderSelectorImpl::new(
                accounts_repo.clone(),
                customer_repo.clone(),
                company_repo.clone(),
                management_repo.clone(),
                partners_repo.clone(),
                partners_list_repo.clone(),
                fees_repo.clone(),
            ));
        let webhook_batch_processor = Arc::new(gatebox_rust::core::pix_principal::WebhookBatchProcessor::new(
            write_pool.clone(),
            20,
            std::time::Duration::from_secs(1),
        ));
        let webhook_svc: Arc<dyn gatebox_rust::core::pix_principal::PixWebhookService> =
            Arc::new(
                gatebox_rust::core::pix_principal::PixWebhookServiceImpl::new(
                    transaction_repo.clone(),
                    key_pix_repo.clone(),
                    account_rules_repo.clone(),
                    with_list_accounts_repo.clone(),
                    fees_repo.clone(),
                    accounts_repo.clone(),
                    provider_selector,
                    invoice_repo.clone(),
                    sec_med_repo.clone(),
                    default_partners_id,
                )
                .with_batch_processor(webhook_batch_processor)
                .with_gateway_recorder(gateway_recorder.clone()),
            );

        let admin_state = admin::AdminState {
            customer_svc: customer_svc.clone(),
            accounts_svc: accounts_svc.clone(),
            partners_svc: partners_svc.clone(),
            webhook_manager_svc: webhook_manager_svc.clone(),
            transaction_svc: transaction_svc.clone(),
            auth_svc: Some(authentication_svc.clone()),
            pix_svc: Some(pix_principal_svc.clone()),
            customer_status_types_svc: Some(customer_status_types_svc.clone()),
            app_log_repo: Some(app_log_repo.clone()),
            login_limiter: admin::LoginRateLimiter::new(),
        };
        let qr_cache_early: gatebox_rust::bank_bridge::QrRefCache =
            Arc::new(std::sync::RwLock::new(std::collections::HashMap::new()));

        let pix_principal_state = gatebox_rust::core::pix_principal::PixPrincipalState {
            service: pix_principal_svc.clone(),
            webhook_service: Some(webhook_svc.clone()),
            qr_cache: qr_cache_early.clone(),
        };
        let p2p_svc: Arc<dyn gatebox_rust::p2p::P2PService> = Arc::new(
            gatebox_rust::p2p::P2PServiceImpl::new(
                transaction_repo.clone(),
                customer_repo.clone(),
                accounts_repo.clone(),
            ),
        );
        let customers_state = gatebox_rust::modules::customers::CustomersState {
            pix_svc: pix_principal_svc,
            webhook_svc: Some(webhook_svc),
            auth_svc: Some(authentication_svc.clone()),
            accounts_svc: Some(accounts_svc.clone()),
            p2p_svc: Some(p2p_svc),
            webhook_manager_svc: Some(webhook_manager_svc.clone()),
        };

        let bank_bridge_state = gatebox_rust::bank_bridge::BankBridgeState {
            api_key: std::env::var("GATEBOX_BANK_BRIDGE_API_KEY")
                .or_else(|_| std::env::var("GATEBOX_API_KEY"))
                .unwrap_or_else(|_| "sandbox-key".into()),
            tx_repo: transaction_repo.clone(),
            qr_cache: qr_cache_early.clone(),
        };

        let api = Router::new()
            .merge(gatebox_rust::bank_bridge::routes(bank_bridge_state))
            .nest("/v1/admin", admin::routes(admin_state))
            .nest("/v1/admin/disputes", gatebox_rust::disputes::admin_routes(dispute_state.clone()))
            .nest("/v1", gatebox_rust::disputes::customer_routes(dispute_state))
            .nest("/v1/backoffice", backoffice::routes(backoffice_state))
            .nest("/v1/pix", gatebox_rust::core::pix_principal::register(pix_principal_state))
            .nest("/v1/customers", gatebox_rust::modules::customers::routes(customers_state))
            .nest("/v1/account_rules", account_rules::routes(account_rules_svc))
            .nest("/v1/accounts", accounts::routes(accounts_svc))
            .nest("/v1/account_types", account_types::routes(account_types_svc))
            .nest("/v1/account_status_types", account_status_types::routes(account_status_types_svc))
            .nest("/v1/address_types", address_types::routes(address_types_svc))
            .nest("/v1/addresses", address::routes(address_svc))
            .nest("/v1/authentication", authentication::routes(authentication_svc))
            .nest("/v1/companies", company::routes(company_svc))
            .nest("/v1/control_med", control_med::routes(control_med_svc))
            .nest("/v1/management", management::routes(management_svc))
            .nest("/v1/customer_status_types", customer_status_types::routes(customer_status_types_svc))
            .nest("/v1/customer", customer::routes(customer_svc))
            .nest("/v1/fees", fees::routes(fees_svc))
            .nest("/v1/history_med", history_med::routes(history_med_svc))
            .nest("/v1/invoice", invoice::routes(invoice_svc))
            .nest("/v1/invoice_status_types", invoice_status_types::routes(invoice_status_types_svc))
            .nest("/v1/invoice_types", invoice_types::routes(invoice_types_svc))
            .nest("/v1/kyc_risk_types", kyc_risk_types::routes(kyc_risk_types_svc))
            .nest("/v1/partners", partners::routes(partners_svc))
            .nest("/v1/partners_list", partners_list::routes(partners_list_svc))
            .nest("/v1/key_pix", key_pix::routes(key_pix_svc))
            .nest("/v1/key_pix_cache", key_pix_cache::routes(key_pix_cache_svc))
            .nest("/v1/pix_key_types", pix_key_types::routes(pix_key_types_svc))
            .nest("/v1/sec_med", sec_med::routes(sec_med_svc))
            .nest("/v1/shared_key", shared_key::routes(shared_key_svc))
            .nest("/v1/status_controle_med_types", status_controle_med_types::routes(status_controle_med_types_svc))
            .nest("/v1/status_sec_med_types", status_sec_med_types::routes(status_sec_med_types_svc))
            .nest("/v1/status_transaction_types", status_transaction_types::routes(status_transaction_types_svc))
            .nest("/v1/styled", styled::routes(styled_svc))
            .nest("/v1/styled_types", styled_types::routes(styled_types_svc))
            .nest("/v1/transaction", transaction::routes(transaction_svc))
            .nest("/v1/token_service", token_service::routes(token_service_svc))
            .nest("/v1/sub_type_transaction_types", sub_type_transaction_types::routes(sub_type_transaction_types_svc))
            .nest("/v1/type_auth_types", type_auth_types::routes(type_auth_types_svc))
            .nest("/v1/type_authorize_types", type_authorize_types::routes(type_authorize_types_svc))
            .nest("/v1/type_external_types", type_external_types::routes(type_external_types_svc))
            .nest("/v1/type_person_types", type_person_types::routes(type_person_types_svc))
            .nest("/v1/type_transaction_types", type_transaction_types::routes(type_transaction_types_svc))
            .nest("/v1/webhook_manager", webhook_manager::routes(webhook_manager_svc))
            .nest("/v1/webhook_types", webhook_types::routes(webhook_types_svc))
            .nest("/v1/with_list_accounts", with_list_accounts::routes(with_list_accounts_svc));

        // Servidor Prometheus dedicado na porta 2112 (como no Go)
        let metrics_port = std::env::var("METRICS_PORT").unwrap_or_else(|_| "2112".to_string());
        let enable_metrics = std::env::var("ENABLE_METRICS").unwrap_or_else(|_| "true".to_string());
        let _metrics_handle = if enable_metrics.to_lowercase() != "false" {
            if let Ok(Some(h)) = metrics_server::start_metrics_server(&metrics_port).await {
                info!("Métricas disponíveis em: http://localhost:{}/metrics", metrics_port);
                Some(h)
            } else {
                None
            }
        } else {
            tracing::warn!("Servidor de métricas desabilitado via ENABLE_METRICS=false");
            None
        };

        // Swagger com Basic Auth (endpoint/segredo, como no Go)
        let swagger = SwaggerUi::new("/swagger").url("/api-docs/openapi.json", ApiDoc::openapi());
        let swagger_with_auth =
            Router::new().merge(swagger).layer(middleware::from_fn(swagger_basic_auth));

        // CORS: restrict to configured origins in production; allow any in dev.
        let cors_layer = {
            let allowed = std::env::var("GATEBOX_CORS_ORIGINS").unwrap_or_default();
            if allowed.is_empty() || allowed == "*" {
                CorsLayer::new()
                    .allow_origin(Any)
                    .allow_methods(Any)
                    .allow_headers(Any)
            } else {
                let origins: Vec<axum::http::HeaderValue> = allowed
                    .split(',')
                    .filter_map(|s| s.trim().parse().ok())
                    .collect();
                CorsLayer::new()
                    .allow_origin(origins)
                    .allow_methods(Any)
                    .allow_headers(Any)
            }
        };

        let app = Router::new()
            .route("/health", get(health))
            .route("/internal/metrics", get(internal_metrics_json))
            .nest("/api", api)
            .merge(swagger_with_auth)
            .layer(cors_layer)
            .layer(
                ServiceBuilder::new()
                    .layer(TimeoutLayer::new(Duration::from_secs(30)))
                    .layer(middleware::from_fn(enhanced_http_metrics_wrapper)),
            );

        let addr = format!("0.0.0.0:{}", port);
        let listener = tokio::net::TcpListener::bind(&addr).await?;

        info!("Listening on {}", addr);
        info!("Health: http://localhost:{}/health", port);
        info!("Swagger: http://localhost:{}/swagger/", port);
        info!("Métricas: http://localhost:{}/metrics", metrics_port);
        info!("API: http://localhost:{}/api/v1/accounts", port);
        info!(
            "Banco Saczuck bridge: http://localhost:{}/api/public/charges/validate (Bearer = GATEBOX_BANK_BRIDGE_API_KEY ou GATEBOX_API_KEY)",
            port
        );

        axum::serve(listener, app)
            .with_graceful_shutdown(shutdown_signal())
            .await?;

        if let Some(pool) = rabbitmq_worker {
            let _ = pool.stop().await;
            info!("RabbitMQ WorkerPool stopped");
        }

        if let Some(tx) = pulsar_cancel_tx {
            let _ = tx.send(());
            info!("Pulsar ResilientConsumer stop signal sent");
        }

        if let Some(tx) = hospital_cancel_tx {
            let _ = tx.send(());
            info!("Hospital consumer stop signal sent");
        }

        Ok(())
    }

    pub async fn stop(&mut self) {
        self.write_pool = None;
        self.read_pool = None;
        info!("Server stopped");
    }
}

async fn health() -> Json<serde_json::Value> {
    Json(json!({
        "status": "healthy",
        "service": "gatebox-rust",
        "version": "1.0.0",
        "timestamp": chrono::Utc::now().to_rfc3339()
    }))
}

/// Retorna JSON informativo sobre métricas (como no Go); métricas Prometheus em GET /metrics na porta 2112.
async fn internal_metrics_json() -> Json<serde_json::Value> {
    let metrics_port = std::env::var("METRICS_PORT").unwrap_or_else(|_| "2112".to_string());
    Json(json!({
        "metrics_endpoint": format!("http://localhost:{}/metrics", metrics_port),
        "prometheus_format": true,
        "service": "gatebox-rust"
    }))
}

/// Basic Auth para Swagger (endpoint/segredo, como no Go).
async fn swagger_basic_auth(
    req: axum::extract::Request,
    next: middleware::Next,
) -> axum::response::Response {
    let auth = req
        .headers()
        .get("Authorization")
        .and_then(|v| v.to_str().ok());
    let valid = auth
        .and_then(|a| {
            a.strip_prefix("Basic ").and_then(|b64| {
                BASE64.decode(b64).ok().and_then(|bytes| {
                    String::from_utf8(bytes).ok().map(|s| {
                        let parts: Vec<&str> = s.splitn(2, ':').collect();
                        parts.len() == 2
                            && parts[0] == "endpoint"
                            && parts[1] == "segredo"
                    })
                })
            })
        })
        .unwrap_or(false);
    if valid {
        next.run(req).await
    } else {
        (
            axum::http::StatusCode::UNAUTHORIZED,
            [(
                axum::http::header::WWW_AUTHENTICATE,
                axum::http::HeaderValue::from_static("Basic realm=\"Swagger\""),
            )],
            Json(json!({"error": "Unauthorized"})),
        )
            .into_response()
    }
}

/// Wrapper que chama enhanced_http_metrics_layer com usecase "global".
async fn enhanced_http_metrics_wrapper(
    req: axum::extract::Request,
    next: middleware::Next,
) -> axum::response::Response {
    observabilidade::enhanced_http_metrics_layer("global".to_string(), req, next).await
}

async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c().await.expect("failed to install Ctrl+C handler");
    };
    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };
    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }
}
