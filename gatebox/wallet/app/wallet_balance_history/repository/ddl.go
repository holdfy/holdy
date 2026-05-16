package wallet_balance_historyRepo

var(
   SQL_WALLET_BALANCE_HISTORY_LIST = `
       SELECT 
              id, 
              balance_history_code, 
              wallet_id, 
              previous_balance, 
              new_balance, 
              change_amount, 
              id_change_type, 
              reference_id, 
              description, 
              created_at 
         FROM wallet_balance_history
         ORDER BY id LIMIT $1 OFFSET $2
       ;`
 
   SQL_WALLET_BALANCE_HISTORY_INSERT = `
       INSERT INTO wallet_balance_history(
              balance_history_code, 
              wallet_id, 
              previous_balance, 
              new_balance, 
              change_amount, 
              id_change_type, 
              reference_id, 
              description, 
              created_at) 
       VALUES( 
              $1, 
              $2, 
              $3, 
              $4, 
              $5, 
              $6, 
              $7, 
              $8, 
              $9) RETURNING id;` 

   SQL_WALLET_BALANCE_HISTORY_UPDATE = `
       UPDATE wallet_balance_history SET 
              balance_history_code = $1, 
              wallet_id = $2, 
              previous_balance = $3, 
              new_balance = $4, 
              change_amount = $5, 
              id_change_type = $6, 
              reference_id = $7, 
              description = $8, 
              created_at = $9 
        WHERE id = $10; `

   SQL_WALLET_BALANCE_HISTORY_DELETE_BY_ID = `
       DELETE FROM wallet_balance_history
        WHERE id = $1; `

   SQL_GET_WALLET_BALANCE_HISTORY_BY_ID = `
       SELECT 
              id, 
              balance_history_code, 
              wallet_id, 
              previous_balance, 
              new_balance, 
              change_amount, 
              id_change_type, 
              reference_id, 
              description, 
              created_at 
         FROM wallet_balance_history
        WHERE id = $1; `
 
   SQL_GET_WALLET_BALANCE_HISTORY_BY_BALANCE_HISTORY_CODE = `
       SELECT 
              id, 
              balance_history_code, 
              wallet_id, 
              previous_balance, 
              new_balance, 
              change_amount, 
              id_change_type, 
              reference_id, 
              description, 
              created_at 
         FROM wallet_balance_history
        WHERE Balance_history_code  LIKE '%' || $1 || '%' ; `
 
)