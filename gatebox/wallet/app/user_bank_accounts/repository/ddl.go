package user_bank_accountsRepo

var(
   SQL_USER_BANK_ACCOUNTS_LIST = `
       SELECT 
              id, 
              bank_account_code, 
              user_id, 
              wallet_id, 
              id_bank, 
              agency, 
              account_number, 
              id_account_type, 
              holder_name, 
              holder_document, 
              consent_id, 
              consent_expires_at, 
              is_verified, 
              is_active, 
              last_used, 
              created_at, 
              updated_at 
         FROM user_bank_accounts
         ORDER BY id LIMIT $1 OFFSET $2
       ;`
 
   SQL_USER_BANK_ACCOUNTS_INSERT = `
       INSERT INTO user_bank_accounts(
              bank_account_code, 
              user_id, 
              wallet_id, 
              id_bank, 
              agency, 
              account_number, 
              id_account_type, 
              holder_name, 
              holder_document, 
              consent_id, 
              consent_expires_at, 
              is_verified, 
              is_active, 
              last_used, 
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

   SQL_USER_BANK_ACCOUNTS_UPDATE = `
       UPDATE user_bank_accounts SET 
              bank_account_code = $1, 
              user_id = $2, 
              wallet_id = $3, 
              id_bank = $4, 
              agency = $5, 
              account_number = $6, 
              id_account_type = $7, 
              holder_name = $8, 
              holder_document = $9, 
              consent_id = $10, 
              consent_expires_at = $11, 
              is_verified = $12, 
              is_active = $13, 
              last_used = $14, 
              created_at = $15, 
              updated_at = $16 
        WHERE id = $17; `

   SQL_USER_BANK_ACCOUNTS_DELETE_BY_ID = `
       DELETE FROM user_bank_accounts
        WHERE id = $1; `

   SQL_GET_USER_BANK_ACCOUNTS_BY_ID = `
       SELECT 
              id, 
              bank_account_code, 
              user_id, 
              wallet_id, 
              id_bank, 
              agency, 
              account_number, 
              id_account_type, 
              holder_name, 
              holder_document, 
              consent_id, 
              consent_expires_at, 
              is_verified, 
              is_active, 
              last_used, 
              created_at, 
              updated_at 
         FROM user_bank_accounts
        WHERE id = $1; `
 
   SQL_GET_USER_BANK_ACCOUNTS_BY_BANK_ACCOUNT_CODE = `
       SELECT 
              id, 
              bank_account_code, 
              user_id, 
              wallet_id, 
              id_bank, 
              agency, 
              account_number, 
              id_account_type, 
              holder_name, 
              holder_document, 
              consent_id, 
              consent_expires_at, 
              is_verified, 
              is_active, 
              last_used, 
              created_at, 
              updated_at 
         FROM user_bank_accounts
        WHERE Bank_account_code  LIKE '%' || $1 || '%' ; `
 
)