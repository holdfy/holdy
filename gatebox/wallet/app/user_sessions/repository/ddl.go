package user_sessionsRepo

var(
   SQL_USER_SESSIONS_LIST = `
       SELECT 
              id, 
              session_code, 
              user_id, 
              application_id, 
              session_token, 
              device_fingerprint, 
              id_device_type, 
              device_info, 
              ip_address, 
              location_data, 
              id_status, 
              expires_at, 
              last_activity, 
              created_at 
         FROM user_sessions
         ORDER BY id LIMIT $1 OFFSET $2
       ;`
 
   SQL_USER_SESSIONS_INSERT = `
       INSERT INTO user_sessions(
              session_code, 
              user_id, 
              application_id, 
              session_token, 
              device_fingerprint, 
              id_device_type, 
              device_info, 
              ip_address, 
              location_data, 
              id_status, 
              expires_at, 
              last_activity, 
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
              $13) RETURNING id;` 

   SQL_USER_SESSIONS_UPDATE = `
       UPDATE user_sessions SET 
              session_code = $1, 
              user_id = $2, 
              application_id = $3, 
              session_token = $4, 
              device_fingerprint = $5, 
              id_device_type = $6, 
              device_info = $7, 
              ip_address = $8, 
              location_data = $9, 
              id_status = $10, 
              expires_at = $11, 
              last_activity = $12, 
              created_at = $13 
        WHERE id = $14; `

   SQL_USER_SESSIONS_DELETE_BY_ID = `
       DELETE FROM user_sessions
        WHERE id = $1; `

   SQL_GET_USER_SESSIONS_BY_ID = `
       SELECT 
              id, 
              session_code, 
              user_id, 
              application_id, 
              session_token, 
              device_fingerprint, 
              id_device_type, 
              device_info, 
              ip_address, 
              location_data, 
              id_status, 
              expires_at, 
              last_activity, 
              created_at 
         FROM user_sessions
        WHERE id = $1; `
 
   SQL_GET_USER_SESSIONS_BY_SESSION_CODE = `
       SELECT 
              id, 
              session_code, 
              user_id, 
              application_id, 
              session_token, 
              device_fingerprint, 
              id_device_type, 
              device_info, 
              ip_address, 
              location_data, 
              id_status, 
              expires_at, 
              last_activity, 
              created_at 
         FROM user_sessions
        WHERE Session_code  LIKE '%' || $1 || '%' ; `
 
)