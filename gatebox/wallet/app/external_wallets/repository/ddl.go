package external_walletsRepo

var(
   SQL_EXTERNAL_WALLETS_LIST = `
       SELECT 
              id, 
              external_wallet_code, 
              user_id, 
              id_provider, 
              external_account_id, 
              account_info, 
              access_token_encrypted, 
              refresh_token_encrypted, 
              token_expires_at, 
              is_active, 
              last_sync, 
              created_at, 
              updated_at 
         FROM external_wallets
         ORDER BY id LIMIT $1 OFFSET $2
       ;`
 
   SQL_EXTERNAL_WALLETS_INSERT = `
       INSERT INTO external_wallets(
              external_wallet_code, 
              user_id, 
              id_provider, 
              external_account_id, 
              account_info, 
              access_token_encrypted, 
              refresh_token_encrypted, 
              token_expires_at, 
              is_active, 
              last_sync, 
              created_at, 
              updated_at) 
       VALUES( 
              $1, 
              $2, 
              $3, 
              $4, 
              $5, 
              $6, 
              $7, 
              $8, 
              $9, 
              $10, 
              $11, 
              $12) RETURNING id;` 

   SQL_EXTERNAL_WALLETS_UPDATE = `
       UPDATE external_wallets SET 
              external_wallet_code = $1, 
              user_id = $2, 
              id_provider = $3, 
              external_account_id = $4, 
              account_info = $5, 
              access_token_encrypted = $6, 
              refresh_token_encrypted = $7, 
              token_expires_at = $8, 
              is_active = $9, 
              last_sync = $10, 
              created_at = $11, 
              updated_at = $12 
        WHERE id = $13; `

   SQL_EXTERNAL_WALLETS_DELETE_BY_ID = `
       DELETE FROM external_wallets
        WHERE id = $1; `

   SQL_GET_EXTERNAL_WALLETS_BY_ID = `
       SELECT 
              id, 
              external_wallet_code, 
              user_id, 
              id_provider, 
              external_account_id, 
              account_info, 
              access_token_encrypted, 
              refresh_token_encrypted, 
              token_expires_at, 
              is_active, 
              last_sync, 
              created_at, 
              updated_at 
         FROM external_wallets
        WHERE id = $1; `
 
   SQL_GET_EXTERNAL_WALLETS_BY_EXTERNAL_WALLET_CODE = `
       SELECT 
              id, 
              external_wallet_code, 
              user_id, 
              id_provider, 
              external_account_id, 
              account_info, 
              access_token_encrypted, 
              refresh_token_encrypted, 
              token_expires_at, 
              is_active, 
              last_sync, 
              created_at, 
              updated_at 
         FROM external_wallets
        WHERE External_wallet_code  LIKE '%' || $1 || '%' ; `
 
)