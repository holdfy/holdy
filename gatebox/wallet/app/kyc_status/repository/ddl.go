package kyc_statusRepo

var(
   SQL_KYC_STATUS_LIST = `
       SELECT 
              id, 
              status_code, 
              name, 
              description, 
              allows_transactions, 
              max_transaction_amount, 
              created_at 
         FROM kyc_status
         ORDER BY id LIMIT $1 OFFSET $2
       ;`
 
   SQL_KYC_STATUS_INSERT = `
       INSERT INTO kyc_status(
              status_code, 
              name, 
              description, 
              allows_transactions, 
              max_transaction_amount, 
              created_at) 
       VALUES( 
              $1, 
              $2, 
              $3, 
              $4, 
              $5, 
              $6) RETURNING id;` 

   SQL_KYC_STATUS_UPDATE = `
       UPDATE kyc_status SET 
              status_code = $1, 
              name = $2, 
              description = $3, 
              allows_transactions = $4, 
              max_transaction_amount = $5, 
              created_at = $6 
        WHERE id = $7; `

   SQL_KYC_STATUS_DELETE_BY_ID = `
       DELETE FROM kyc_status
        WHERE id = $1; `

   SQL_GET_KYC_STATUS_BY_ID = `
       SELECT 
              id, 
              status_code, 
              name, 
              description, 
              allows_transactions, 
              max_transaction_amount, 
              created_at 
         FROM kyc_status
        WHERE id = $1; `
 
   SQL_GET_KYC_STATUS_BY_STATUS_CODE = `
       SELECT 
              id, 
              status_code, 
              name, 
              description, 
              allows_transactions, 
              max_transaction_amount, 
              created_at 
         FROM kyc_status
        WHERE Status_code  LIKE '%' || $1 || '%' ; `
 
)