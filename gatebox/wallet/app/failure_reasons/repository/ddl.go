package failure_reasonsRepo

var(
   SQL_FAILURE_REASONS_LIST = `
       SELECT 
              id, 
              reason_code, 
              name, 
              description, 
              is_critical, 
              is_active, 
              created_at 
         FROM failure_reasons
         ORDER BY id LIMIT $1 OFFSET $2
       ;`
 
   SQL_FAILURE_REASONS_INSERT = `
       INSERT INTO failure_reasons(
              reason_code, 
              name, 
              description, 
              is_critical, 
              is_active, 
              created_at) 
       VALUES( 
              $1, 
              $2, 
              $3, 
              $4, 
              $5, 
              $6) RETURNING id;` 

   SQL_FAILURE_REASONS_UPDATE = `
       UPDATE failure_reasons SET 
              reason_code = $1, 
              name = $2, 
              description = $3, 
              is_critical = $4, 
              is_active = $5, 
              created_at = $6 
        WHERE id = $7; `

   SQL_FAILURE_REASONS_DELETE_BY_ID = `
       DELETE FROM failure_reasons
        WHERE id = $1; `

   SQL_GET_FAILURE_REASONS_BY_ID = `
       SELECT 
              id, 
              reason_code, 
              name, 
              description, 
              is_critical, 
              is_active, 
              created_at 
         FROM failure_reasons
        WHERE id = $1; `
 
   SQL_GET_FAILURE_REASONS_BY_REASON_CODE = `
       SELECT 
              id, 
              reason_code, 
              name, 
              description, 
              is_critical, 
              is_active, 
              created_at 
         FROM failure_reasons
        WHERE Reason_code  LIKE '%' || $1 || '%' ; `
 
)