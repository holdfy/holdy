package payment_methodsRepo

var(
   SQL_PAYMENT_METHODS_LIST = `
       SELECT 
              id, 
              method_code, 
              name, 
              description, 
              requires_external_auth, 
              processing_time_minutes, 
              is_active, 
              created_at 
         FROM payment_methods
         ORDER BY id LIMIT $1 OFFSET $2
       ;`
 
   SQL_PAYMENT_METHODS_INSERT = `
       INSERT INTO payment_methods(
              method_code, 
              name, 
              description, 
              requires_external_auth, 
              processing_time_minutes, 
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

   SQL_PAYMENT_METHODS_UPDATE = `
       UPDATE payment_methods SET 
              method_code = $1, 
              name = $2, 
              description = $3, 
              requires_external_auth = $4, 
              processing_time_minutes = $5, 
              is_active = $6, 
              created_at = $7 
        WHERE id = $8; `

   SQL_PAYMENT_METHODS_DELETE_BY_ID = `
       DELETE FROM payment_methods
        WHERE id = $1; `

   SQL_GET_PAYMENT_METHODS_BY_ID = `
       SELECT 
              id, 
              method_code, 
              name, 
              description, 
              requires_external_auth, 
              processing_time_minutes, 
              is_active, 
              created_at 
         FROM payment_methods
        WHERE id = $1; `
 
   SQL_GET_PAYMENT_METHODS_BY_METHOD_CODE = `
       SELECT 
              id, 
              method_code, 
              name, 
              description, 
              requires_external_auth, 
              processing_time_minutes, 
              is_active, 
              created_at 
         FROM payment_methods
        WHERE Method_code  LIKE '%' || $1 || '%' ; `
 
)