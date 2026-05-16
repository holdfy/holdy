package gateway_status_typesRepo

var(
   SQL_GATEWAY_STATUS_TYPES_LIST = `
       SELECT 
              id, 
              status_code, 
              name, 
              description, 
              is_success, 
              is_final, 
              is_active, 
              created_at 
         FROM gateway_status_types
         ORDER BY id LIMIT $1 OFFSET $2
       ;`
 
   SQL_GATEWAY_STATUS_TYPES_INSERT = `
       INSERT INTO gateway_status_types(
              status_code, 
              name, 
              description, 
              is_success, 
              is_final, 
              is_active, 
              created_at) 
       VALUES( 
              $1, 
              $2, 
              $3, 
              $4, 
              $5, 
              $6, 
              $7) RETURNING id;` 

   SQL_GATEWAY_STATUS_TYPES_UPDATE = `
       UPDATE gateway_status_types SET 
              status_code = $1, 
              name = $2, 
              description = $3, 
              is_success = $4, 
              is_final = $5, 
              is_active = $6, 
              created_at = $7 
        WHERE id = $8; `

   SQL_GATEWAY_STATUS_TYPES_DELETE_BY_ID = `
       DELETE FROM gateway_status_types
        WHERE id = $1; `

   SQL_GET_GATEWAY_STATUS_TYPES_BY_ID = `
       SELECT 
              id, 
              status_code, 
              name, 
              description, 
              is_success, 
              is_final, 
              is_active, 
              created_at 
         FROM gateway_status_types
        WHERE id = $1; `
 
   SQL_GET_GATEWAY_STATUS_TYPES_BY_STATUS_CODE = `
       SELECT 
              id, 
              status_code, 
              name, 
              description, 
              is_success, 
              is_final, 
              is_active, 
              created_at 
         FROM gateway_status_types
        WHERE Status_code  LIKE '%' || $1 || '%' ; `
 
)