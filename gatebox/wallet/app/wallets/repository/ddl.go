package walletsRepo

var(
   SQL_WALLETS_LIST = `
       SELECT 
              id, 
              wallet_code, 
              user_id, 
              application_id, 
              name, 
              id_wallet_type, 
              id_currency, 
              id_status, 
              balance, 
              available_balance, 
              daily_limit, 
              monthly_limit, 
              signature_required_above, 
              is_primary, 
              metadata, 
              created_at, 
              updated_at 
         FROM wallets
         ORDER BY id LIMIT $1 OFFSET $2
       ;`
 
   SQL_WALLETS_INSERT = `
       INSERT INTO wallets(
              wallet_code, 
              user_id, 
              application_id, 
              name, 
              id_wallet_type, 
              id_currency, 
              id_status, 
              balance, 
              available_balance, 
              daily_limit, 
              monthly_limit, 
              signature_required_above, 
              is_primary, 
              metadata, 
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
              $12, 
              $13, 
              $14, 
              $15, 
              $16) RETURNING id;` 

   SQL_WALLETS_UPDATE = `
       UPDATE wallets SET 
              wallet_code = $1, 
              user_id = $2, 
              application_id = $3, 
              name = $4, 
              id_wallet_type = $5, 
              id_currency = $6, 
              id_status = $7, 
              balance = $8, 
              available_balance = $9, 
              daily_limit = $10, 
              monthly_limit = $11, 
              signature_required_above = $12, 
              is_primary = $13, 
              metadata = $14, 
              created_at = $15, 
              updated_at = $16 
        WHERE id = $17; `

   SQL_WALLETS_DELETE_BY_ID = `
       DELETE FROM wallets
        WHERE id = $1; `

   SQL_GET_WALLETS_BY_ID = `
       SELECT 
              id, 
              wallet_code, 
              user_id, 
              application_id, 
              name, 
              id_wallet_type, 
              id_currency, 
              id_status, 
              balance, 
              available_balance, 
              daily_limit, 
              monthly_limit, 
              signature_required_above, 
              is_primary, 
              metadata, 
              created_at, 
              updated_at 
         FROM wallets
        WHERE id = $1; `
 
   SQL_GET_WALLETS_BY_WALLET_CODE = `
       SELECT 
              id, 
              wallet_code, 
              user_id, 
              application_id, 
              name, 
              id_wallet_type, 
              id_currency, 
              id_status, 
              balance, 
              available_balance, 
              daily_limit, 
              monthly_limit, 
              signature_required_above, 
              is_primary, 
              metadata, 
              created_at, 
              updated_at 
         FROM wallets
        WHERE Wallet_code  LIKE '%' || $1 || '%' ; `
 
)