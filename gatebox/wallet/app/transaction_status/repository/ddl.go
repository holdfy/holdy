package transaction_statusRepo

var(
   SQL_TRANSACTION_STATUS_LIST = `
       SELECT 
              id, 
              status_code, 
              name, 
              description, 
              is_final, 
              is_success, 
              allows_refund, 
              is_active, 
              created_at 
         FROM transaction_status
         ORDER BY id LIMIT $1 OFFSET $2
       ;`
 
   SQL_TRANSACTION_STATUS_INSERT = `
       INSERT INTO transaction_status(
              status_code, 
              name, 
              description, 
              is_final, 
              is_success, 
              allows_refund, 
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

   SQL_TRANSACTION_STATUS_UPDATE = `
       UPDATE transaction_status SET 
              status_code = $1, 
              name = $2, 
              description = $3, 
              is_final = $4, 
              is_success = $5, 
              allows_refund = $6, 
              is_active = $7, 
              created_at = $8 
        WHERE id = $9; `

   SQL_TRANSACTION_STATUS_DELETE_BY_ID = `
       DELETE FROM transaction_status
        WHERE id = $1; `

   SQL_GET_TRANSACTION_STATUS_BY_ID = `
       SELECT 
              id, 
              status_code, 
              name, 
              description, 
              is_final, 
              is_success, 
              allows_refund, 
              is_active, 
              created_at 
         FROM transaction_status
        WHERE id = $1; `
 
   SQL_GET_TRANSACTION_STATUS_BY_STATUS_CODE = `
       SELECT 
              id, 
              status_code, 
              name, 
              description, 
              is_final, 
              is_success, 
              allows_refund, 
              is_active, 
              created_at 
         FROM transaction_status
        WHERE Status_code  LIKE '%' || $1 || '%' ; `
 
)