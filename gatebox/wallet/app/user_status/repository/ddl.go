package user_statusRepo

var(
   SQL_USER_STATUS_LIST = `
       SELECT 
              id, 
              status_code, 
              name, 
              description, 
              is_active, 
              created_at 
         FROM user_status
         ORDER BY id LIMIT $1 OFFSET $2
       ;`
 
   SQL_USER_STATUS_INSERT = `
       INSERT INTO user_status(
              status_code, 
              name, 
              description, 
              is_active, 
              created_at) 
       VALUES( 
              $1, 
              $2, 
              $3, 
              $4, 
              $5) RETURNING id;` 

   SQL_USER_STATUS_UPDATE = `
       UPDATE user_status SET 
              status_code = $1, 
              name = $2, 
              description = $3, 
              is_active = $4, 
              created_at = $5 
        WHERE id = $6; `

   SQL_USER_STATUS_DELETE_BY_ID = `
       DELETE FROM user_status
        WHERE id = $1; `

   SQL_GET_USER_STATUS_BY_ID = `
       SELECT 
              id, 
              status_code, 
              name, 
              description, 
              is_active, 
              created_at 
         FROM user_status
        WHERE id = $1; `
 
   SQL_GET_USER_STATUS_BY_STATUS_CODE = `
       SELECT 
              id, 
              status_code, 
              name, 
              description, 
              is_active, 
              created_at 
         FROM user_status
        WHERE Status_code  LIKE '%' || $1 || '%' ; `
 
)