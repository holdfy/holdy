package wallet_statusRepo

var(
   SQL_WALLET_STATUS_LIST = `
       SELECT 
              id, 
              status_code, 
              name, 
              description, 
              allows_transactions, 
              allows_deposits, 
              allows_withdrawals, 
              is_active, 
              created_at 
         FROM wallet_status
         ORDER BY id LIMIT $1 OFFSET $2
       ;`
 
   SQL_WALLET_STATUS_INSERT = `
       INSERT INTO wallet_status(
              status_code, 
              name, 
              description, 
              allows_transactions, 
              allows_deposits, 
              allows_withdrawals, 
              is_active, 
              created_at) 
       VALUES( 
              $1, 
              $2, 
              $3, 
              $4, 
              $5, 
              $6, 
              $7, 
              $8) RETURNING id;` 

   SQL_WALLET_STATUS_UPDATE = `
       UPDATE wallet_status SET 
              status_code = $1, 
              name = $2, 
              description = $3, 
              allows_transactions = $4, 
              allows_deposits = $5, 
              allows_withdrawals = $6, 
              is_active = $7, 
              created_at = $8 
        WHERE id = $9; `

   SQL_WALLET_STATUS_DELETE_BY_ID = `
       DELETE FROM wallet_status
        WHERE id = $1; `

   SQL_GET_WALLET_STATUS_BY_ID = `
       SELECT 
              id, 
              status_code, 
              name, 
              description, 
              allows_transactions, 
              allows_deposits, 
              allows_withdrawals, 
              is_active, 
              created_at 
         FROM wallet_status
        WHERE id = $1; `
 
   SQL_GET_WALLET_STATUS_BY_STATUS_CODE = `
       SELECT 
              id, 
              status_code, 
              name, 
              description, 
              allows_transactions, 
              allows_deposits, 
              allows_withdrawals, 
              is_active, 
              created_at 
         FROM wallet_status
        WHERE Status_code  LIKE '%' || $1 || '%' ; `
 
)