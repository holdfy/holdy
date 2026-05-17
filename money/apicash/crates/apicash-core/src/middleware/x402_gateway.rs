//! Middleware x402 (HTTP 402) nas rotas protegidas; bypass com JWT válido (WhatsApp / app).

use std::sync::Arc;

use alloy_primitives::Address;
use apicash_auth::AuthService;
use apicash_shared::{
    assert_x402_config, facilitator_url, pay_to_address, price_usdc, public_base_url, require_x402,
};
use axum::http::{header, HeaderMap};
use url::Url;
use x402_axum::facilitator_client::FacilitatorClient;
use x402_axum::paygate::DynamicPriceTags;
use x402_axum::{X402LayerBuilder, X402Middleware};
use x402_chain_eip155::{KnownNetworkEip155, V2Eip155Exact};
use x402_types::networks::USDC;
use x402_types::proto::v2::PriceTag;

use crate::state::AppState;

/// Layer tower aplicado ao sub-router `protected`.
pub type X402PaymentLayer =
    X402LayerBuilder<DynamicPriceTags<PriceTag>, Arc<FacilitatorClient>>;

fn extract_bearer(headers: &HeaderMap) -> Option<&str> {
    headers
        .get(header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .and_then(|h| h.strip_prefix("Bearer ").or_else(|| h.strip_prefix("bearer ")))
}

/// JWT HS256 válido → sem cobrança x402 (integradores sem conta pagam USDC).
pub fn jwt_bypasses_x402(auth: &AuthService, headers: &HeaderMap) -> bool {
    jwt_bypasses_x402_token(auth, extract_bearer(headers))
}

fn jwt_bypasses_x402_token(auth: &AuthService, bearer: Option<&str>) -> bool {
    if auth.config().jwt_secret.is_empty() {
        return false;
    }
    let Some(token) = bearer else {
        return false;
    };
    auth.validate_access_token(token).is_ok()
}

/// Constrói o layer x402 quando `APICASH_X402_REQUIRED=1`; caso contrário `None`.
pub fn build_x402_layer(state: Arc<AppState>) -> Option<X402PaymentLayer> {
    if !require_x402() {
        return None;
    }
    assert_x402_config().expect("x402 config validated at startup");

    let facilitator = facilitator_url().expect("X402_FACILITATOR_URL");
    let pay_to_str = pay_to_address().expect("X402_PAY_TO");
    let pay_to: Address = pay_to_str
        .parse()
        .unwrap_or_else(|_| panic!("X402_PAY_TO is not a valid EVM address: {pay_to_str}"));
    let price = price_usdc();

    let base_url = Url::parse(&public_base_url())
        .unwrap_or_else(|e| panic!("APICASH_PUBLIC_BASE_URL invalid: {e}"));

    let auth = state.auth.clone();

    Some(
        X402Middleware::new(facilitator.as_str())
            .with_base_url(base_url)
            .with_dynamic_price(move |headers, _uri, _base_url| {
                let auth = auth.clone();
                let bearer = extract_bearer(headers).map(str::to_string);
                let price = price.clone();
                async move {
                    if jwt_bypasses_x402_token(&auth, bearer.as_deref()) {
                        return vec![];
                    }
                    let amount = USDC::base_sepolia().parse(price.as_str()).unwrap_or_else(|e| {
                        panic!("X402_PRICE_USDC invalid for USDC::base_sepolia(): {e}");
                    });
                    vec![V2Eip155Exact::price_tag(pay_to, amount)]
                }
            }),
    )
}
