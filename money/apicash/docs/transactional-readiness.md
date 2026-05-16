# Transactional readiness (pre–go-live checklist)

Use this checklist before moving **real money**. Adapt **testnet vs mainnet** per your rollout policy.

1. **Anchor:** `APICASH_STELLAR_ANCHOR_URL` is HTTPS with a valid certificate; deposit/poll/withdraw JSON matches the provider contract (see `anchor-http-contract.md`); no mocked fiat rails in configuration.
2. **Stellar / Soroban:** network explicitly selected (`APICASH_STELLAR_*`); accounts funded; trustlines where required; escrow contract exercised through terminal states in the target environment.
3. **WhatsApp:** Meta webhook URL uses HTTPS; credentials configured; bot sends PIX **only** after `pix_br_code` is present from the anchor response.
4. **Reconciliation:** persist `transaction_id`, order ids, and ledger hashes; idempotent retries where supported; timeouts surfaced as failures—never silent success.
5. **Operations:** structured logs on settlement failures; alarms or dashboards for anchor/Horizon errors; product limits (max amount per order, fraud controls).

Recommended sequence: validate end-to-end on **public Stellar testnet** + contractual anchor endpoint first; cut over to **mainnet** only after legal/ops sign-off.
