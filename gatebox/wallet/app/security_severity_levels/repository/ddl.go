package security_severity_levelsRepo

var(
   SQL_SECURITY_SEVERITY_LEVELS_LIST = `
       SELECT 
              id, 
              severity_code, 
              name, 
              description, 
              level_number, 
              notification_required, 
              escalation_required, 
              is_active, 
              created_at 
         FROM security_severity_levels
         ORDER BY id LIMIT $1 OFFSET $2
       ;`
 
   SQL_SECURITY_SEVERITY_LEVELS_INSERT = `
       INSERT INTO security_severity_levels(
              severity_code, 
              name, 
              description, 
              level_number, 
              notification_required, 
              escalation_required, 
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

   SQL_SECURITY_SEVERITY_LEVELS_UPDATE = `
       UPDATE security_severity_levels SET 
              severity_code = $1, 
              name = $2, 
              description = $3, 
              level_number = $4, 
              notification_required = $5, 
              escalation_required = $6, 
              is_active = $7, 
              created_at = $8 
        WHERE id = $9; `

   SQL_SECURITY_SEVERITY_LEVELS_DELETE_BY_ID = `
       DELETE FROM security_severity_levels
        WHERE id = $1; `

   SQL_GET_SECURITY_SEVERITY_LEVELS_BY_ID = `
       SELECT 
              id, 
              severity_code, 
              name, 
              description, 
              level_number, 
              notification_required, 
              escalation_required, 
              is_active, 
              created_at 
         FROM security_severity_levels
        WHERE id = $1; `
 
   SQL_GET_SECURITY_SEVERITY_LEVELS_BY_SEVERITY_CODE = `
       SELECT 
              id, 
              severity_code, 
              name, 
              description, 
              level_number, 
              notification_required, 
              escalation_required, 
              is_active, 
              created_at 
         FROM security_severity_levels
        WHERE Severity_code  LIKE '%' || $1 || '%' ; `
 
)