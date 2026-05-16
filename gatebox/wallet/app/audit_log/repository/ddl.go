package audit_logRepo

var(
   SQL_AUDIT_LOG_LIST = `
       SELECT 
              id, 
              audit_code, 
              id_table, 
              record_id, 
              id_action, 
              old_values, 
              new_values, 
              changed_fields, 
              user_id, 
              application_id, 
              ip_address, 
              user_agent, 
              session_id, 
              created_at 
         FROM audit_log
         ORDER BY id LIMIT $1 OFFSET $2
       ;`
 
   SQL_AUDIT_LOG_INSERT = `
       INSERT INTO audit_log(
              audit_code, 
              id_table, 
              record_id, 
              id_action, 
              old_values, 
              new_values, 
              changed_fields, 
              user_id, 
              application_id, 
              ip_address, 
              user_agent, 
              session_id, 
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

   SQL_AUDIT_LOG_UPDATE = `
       UPDATE audit_log SET 
              audit_code = $1, 
              id_table = $2, 
              record_id = $3, 
              id_action = $4, 
              old_values = $5, 
              new_values = $6, 
              changed_fields = $7, 
              user_id = $8, 
              application_id = $9, 
              ip_address = $10, 
              user_agent = $11, 
              session_id = $12, 
              created_at = $13 
        WHERE id = $14; `

   SQL_AUDIT_LOG_DELETE_BY_ID = `
       DELETE FROM audit_log
        WHERE id = $1; `

   SQL_GET_AUDIT_LOG_BY_ID = `
       SELECT 
              id, 
              audit_code, 
              id_table, 
              record_id, 
              id_action, 
              old_values, 
              new_values, 
              changed_fields, 
              user_id, 
              application_id, 
              ip_address, 
              user_agent, 
              session_id, 
              created_at 
         FROM audit_log
        WHERE id = $1; `
 
   SQL_GET_AUDIT_LOG_BY_AUDIT_CODE = `
       SELECT 
              id, 
              audit_code, 
              id_table, 
              record_id, 
              id_action, 
              old_values, 
              new_values, 
              changed_fields, 
              user_id, 
              application_id, 
              ip_address, 
              user_agent, 
              session_id, 
              created_at 
         FROM audit_log
        WHERE Audit_code  LIKE '%' || $1 || '%' ; `
 
)