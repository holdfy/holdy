package session_statusRepo

var(
   SQL_SESSION_STATUS_LIST = `
       SELECT 
              id, 
              status_code, 
              name, 
              description, 
              allows_activity, 
              is_active, 
              created_at 
         FROM session_status
         ORDER BY id LIMIT $1 OFFSET $2
       ;`
 
   SQL_SESSION_STATUS_INSERT = `
       INSERT INTO session_status(
              status_code, 
              name, 
              description, 
              allows_activity, 
              is_active, 
              created_at) 
       VALUES( 
              $1, 
              $2, 
              $3, 
              $4, 
              $5, 
              $6) RETURNING id;` 

   SQL_SESSION_STATUS_UPDATE = `
       UPDATE session_status SET 
              status_code = $1, 
              name = $2, 
              description = $3, 
              allows_activity = $4, 
              is_active = $5, 
              created_at = $6 
        WHERE id = $7; `

   SQL_SESSION_STATUS_DELETE_BY_ID = `
       DELETE FROM session_status
        WHERE id = $1; `

   SQL_GET_SESSION_STATUS_BY_ID = `
       SELECT 
              id, 
              status_code, 
              name, 
              description, 
              allows_activity, 
              is_active, 
              created_at 
         FROM session_status
        WHERE id = $1; `
 
   SQL_GET_SESSION_STATUS_BY_STATUS_CODE = `
       SELECT 
              id, 
              status_code, 
              name, 
              description, 
              allows_activity, 
              is_active, 
              created_at 
         FROM session_status
        WHERE Status_code  LIKE '%' || $1 || '%' ; `
 
)