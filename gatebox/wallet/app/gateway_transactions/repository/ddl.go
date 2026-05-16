package gateway_transactionsRepo

var(
   SQL_GATEWAY_TRANSACTIONS_LIST = `
       SELECT 
              id, 
              gateway_transaction_code, 
              transaction_id, 
              id_gateway, 
              gateway_transaction_id, 
              id_gateway_status, 
              gateway_response, 
              gateway_request, 
              processing_time_ms, 
              retry_count, 
              last_retry_at, 
              created_at, 
              updated_at 
         FROM gateway_transactions
         ORDER BY id LIMIT $1 OFFSET $2
       ;`
 
   SQL_GATEWAY_TRANSACTIONS_INSERT = `
       INSERT INTO gateway_transactions(
              gateway_transaction_code, 
              transaction_id, 
              id_gateway, 
              gateway_transaction_id, 
              id_gateway_status, 
              gateway_response, 
              gateway_request, 
              processing_time_ms, 
              retry_count, 
              last_retry_at, 
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

   SQL_GATEWAY_TRANSACTIONS_UPDATE = `
       UPDATE gateway_transactions SET 
              gateway_transaction_code = $1, 
              transaction_id = $2, 
              id_gateway = $3, 
              gateway_transaction_id = $4, 
              id_gateway_status = $5, 
              gateway_response = $6, 
              gateway_request = $7, 
              processing_time_ms = $8, 
              retry_count = $9, 
              last_retry_at = $10, 
              created_at = $11, 
              updated_at = $12 
        WHERE id = $13; `

   SQL_GATEWAY_TRANSACTIONS_DELETE_BY_ID = `
       DELETE FROM gateway_transactions
        WHERE id = $1; `

   SQL_GET_GATEWAY_TRANSACTIONS_BY_ID = `
       SELECT 
              id, 
              gateway_transaction_code, 
              transaction_id, 
              id_gateway, 
              gateway_transaction_id, 
              id_gateway_status, 
              gateway_response, 
              gateway_request, 
              processing_time_ms, 
              retry_count, 
              last_retry_at, 
              created_at, 
              updated_at 
         FROM gateway_transactions
        WHERE id = $1; `
 
   SQL_GET_GATEWAY_TRANSACTIONS_BY_GATEWAY_TRANSACTION_CODE = `
       SELECT 
              id, 
              gateway_transaction_code, 
              transaction_id, 
              id_gateway, 
              gateway_transaction_id, 
              id_gateway_status, 
              gateway_response, 
              gateway_request, 
              processing_time_ms, 
              retry_count, 
              last_retry_at, 
              created_at, 
              updated_at 
         FROM gateway_transactions
        WHERE Gateway_transaction_code  LIKE '%' || $1 || '%' ; `
 
)