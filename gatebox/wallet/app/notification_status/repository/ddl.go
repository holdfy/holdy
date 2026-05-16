package notification_statusRepo

var(
   SQL_NOTIFICATION_STATUS_LIST = `
       SELECT 
              id, 
              status_code, 
              name, 
              description, 
              is_final, 
              is_success, 
              requires_retry, 
              is_active, 
              created_at 
         FROM notification_status
         ORDER BY id LIMIT $1 OFFSET $2
       ;`
 
   SQL_NOTIFICATION_STATUS_INSERT = `
       INSERT INTO notification_status(
              status_code, 
              name, 
              description, 
              is_final, 
              is_success, 
              requires_retry, 
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

   SQL_NOTIFICATION_STATUS_UPDATE = `
       UPDATE notification_status SET 
              status_code = $1, 
              name = $2, 
              description = $3, 
              is_final = $4, 
              is_success = $5, 
              requires_retry = $6, 
              is_active = $7, 
              created_at = $8 
        WHERE id = $9; `

   SQL_NOTIFICATION_STATUS_DELETE_BY_ID = `
       DELETE FROM notification_status
        WHERE id = $1; `

   SQL_GET_NOTIFICATION_STATUS_BY_ID = `
       SELECT 
              id, 
              status_code, 
              name, 
              description, 
              is_final, 
              is_success, 
              requires_retry, 
              is_active, 
              created_at 
         FROM notification_status
        WHERE id = $1; `
 
   SQL_GET_NOTIFICATION_STATUS_BY_STATUS_CODE = `
       SELECT 
              id, 
              status_code, 
              name, 
              description, 
              is_final, 
              is_success, 
              requires_retry, 
              is_active, 
              created_at 
         FROM notification_status
        WHERE Status_code  LIKE '%' || $1 || '%' ; `
 
)