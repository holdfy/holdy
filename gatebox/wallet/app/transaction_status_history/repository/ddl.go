package transaction_status_historyRepo

var(
   SQL_TRANSACTION_STATUS_HISTORY_LIST = `
       SELECT 
              id, 
              status_history_code, 
              transaction_id, 
              id_previous_status, 
              id_new_status, 
              reason, 
              id_changed_by, 
              gateway_response, 
              created_at 
         FROM transaction_status_history
         ORDER BY id LIMIT $1 OFFSET $2
       ;`
 
   SQL_TRANSACTION_STATUS_HISTORY_INSERT = `
       INSERT INTO transaction_status_history(
              status_history_code, 
              transaction_id, 
              id_previous_status, 
              id_new_status, 
              reason, 
              id_changed_by, 
              gateway_response, 
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

   SQL_TRANSACTION_STATUS_HISTORY_UPDATE = `
       UPDATE transaction_status_history SET 
              status_history_code = $1, 
              transaction_id = $2, 
              id_previous_status = $3, 
              id_new_status = $4, 
              reason = $5, 
              id_changed_by = $6, 
              gateway_response = $7, 
              created_at = $8 
        WHERE id = $9; `

   SQL_TRANSACTION_STATUS_HISTORY_DELETE_BY_ID = `
       DELETE FROM transaction_status_history
        WHERE id = $1; `

   SQL_GET_TRANSACTION_STATUS_HISTORY_BY_ID = `
       SELECT 
              id, 
              status_history_code, 
              transaction_id, 
              id_previous_status, 
              id_new_status, 
              reason, 
              id_changed_by, 
              gateway_response, 
              created_at 
         FROM transaction_status_history
        WHERE id = $1; `
 
   SQL_GET_TRANSACTION_STATUS_HISTORY_BY_STATUS_HISTORY_CODE = `
       SELECT 
              id, 
              status_history_code, 
              transaction_id, 
              id_previous_status, 
              id_new_status, 
              reason, 
              id_changed_by, 
              gateway_response, 
              created_at 
         FROM transaction_status_history
        WHERE Status_history_code  LIKE '%' || $1 || '%' ; `
 
)