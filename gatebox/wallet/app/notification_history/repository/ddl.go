package notification_historyRepo

var(
   SQL_NOTIFICATION_HISTORY_LIST = `
       SELECT 
              id, 
              notification_code, 
              user_id, 
              transaction_id, 
              template_id, 
              id_channel, 
              recipient, 
              subject, 
              message_body, 
              id_status, 
              provider_response, 
              sent_at, 
              delivered_at, 
              failed_at, 
              failure_reason, 
              created_at 
         FROM notification_history
         ORDER BY id LIMIT $1 OFFSET $2
       ;`
 
   SQL_NOTIFICATION_HISTORY_INSERT = `
       INSERT INTO notification_history(
              notification_code, 
              user_id, 
              transaction_id, 
              template_id, 
              id_channel, 
              recipient, 
              subject, 
              message_body, 
              id_status, 
              provider_response, 
              sent_at, 
              delivered_at, 
              failed_at, 
              failure_reason, 
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
              $9, 
              $10, 
              $11, 
              $12, 
              $13, 
              $14, 
              $15) RETURNING id;` 

   SQL_NOTIFICATION_HISTORY_UPDATE = `
       UPDATE notification_history SET 
              notification_code = $1, 
              user_id = $2, 
              transaction_id = $3, 
              template_id = $4, 
              id_channel = $5, 
              recipient = $6, 
              subject = $7, 
              message_body = $8, 
              id_status = $9, 
              provider_response = $10, 
              sent_at = $11, 
              delivered_at = $12, 
              failed_at = $13, 
              failure_reason = $14, 
              created_at = $15 
        WHERE id = $16; `

   SQL_NOTIFICATION_HISTORY_DELETE_BY_ID = `
       DELETE FROM notification_history
        WHERE id = $1; `

   SQL_GET_NOTIFICATION_HISTORY_BY_ID = `
       SELECT 
              id, 
              notification_code, 
              user_id, 
              transaction_id, 
              template_id, 
              id_channel, 
              recipient, 
              subject, 
              message_body, 
              id_status, 
              provider_response, 
              sent_at, 
              delivered_at, 
              failed_at, 
              failure_reason, 
              created_at 
         FROM notification_history
        WHERE id = $1; `
 
   SQL_GET_NOTIFICATION_HISTORY_BY_NOTIFICATION_CODE = `
       SELECT 
              id, 
              notification_code, 
              user_id, 
              transaction_id, 
              template_id, 
              id_channel, 
              recipient, 
              subject, 
              message_body, 
              id_status, 
              provider_response, 
              sent_at, 
              delivered_at, 
              failed_at, 
              failure_reason, 
              created_at 
         FROM notification_history
        WHERE Notification_code  LIKE '%' || $1 || '%' ; `
 
)