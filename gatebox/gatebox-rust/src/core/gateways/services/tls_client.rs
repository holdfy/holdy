// Optional TLS client certificate for gateways (Sulcred/SevenTrust use PFX in Go).
// Env vars for future implementation: SULCRED_CLIENT_CERT_PEM, SULCRED_CLIENT_KEY_PEM (PEM file paths);
// SEVENOUT_CLIENT_CERT_PEM, SEVENOUT_CLIENT_KEY_PEM.
// When set, build reqwest::Client with identity; otherwise use default Client.
use reqwest::Client;

/// Builds a reqwest Client for the given gateway. If env vars for client cert are set,
/// a custom client could be built (e.g. with rustls and identity); for now returns default.
#[allow(dead_code)]
pub fn client_for_gateway(_gateway_name: &str) -> Client {
    // TODO: load SULCRED_CLIENT_CERT_PEM + SULCRED_CLIENT_KEY_PEM (or SEVENOUT_*) and build
    // Client with .identity() when using a TLS backend that supports it.
    Client::new()
}
