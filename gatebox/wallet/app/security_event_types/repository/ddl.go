package security_event_typesRepo

var(
   SQL_SECURITY_EVENT_TYPES_LIST = `
       SELECT 
              id, 
              type_code, 
              name, 
              description, 
              default_severity, 
              auto_block, 
              requires_investigation, 
              is_active, 
              created_at 
         FROM security_event_types
         ORDER BY id LIMIT $1 OFFSET $2
       ;`
 
   SQL_SECURITY_EVENT_TYPES_INSERT = `
       INSERT INTO security_event_types(
              type_code, 
              name, 
              description, 
              default_severity, 
              auto_block, 
              requires_investigation, 
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

   SQL_SECURITY_EVENT_TYPES_UPDATE = `
       UPDATE security_event_types SET 
              type_code = $1, 
              name = $2, 
              description = $3, 
              default_severity = $4, 
              auto_block = $5, 
              requires_investigation = $6, 
              is_active = $7, 
              created_at = $8 
        WHERE id = $9; `

   SQL_SECURITY_EVENT_TYPES_DELETE_BY_ID = `
       DELETE FROM security_event_types
        WHERE id = $1; `

   SQL_GET_SECURITY_EVENT_TYPES_BY_ID = `
       SELECT 
              id, 
              type_code, 
              name, 
              description, 
              default_severity, 
              auto_block, 
              requires_investigation, 
              is_active, 
              created_at 
         FROM security_event_types
        WHERE id = $1; `
 
   SQL_GET_SECURITY_EVENT_TYPES_BY_TYPE_CODE = `
       SELECT 
              id, 
              type_code, 
              name, 
              description, 
              default_severity, 
              auto_block, 
              requires_investigation, 
              is_active, 
              created_at 
         FROM security_event_types
        WHERE Type_code  LIKE '%' || $1 || '%' ; `
 
)