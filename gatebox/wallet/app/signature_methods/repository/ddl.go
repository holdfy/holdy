package signature_methodsRepo

var(
   SQL_SIGNATURE_METHODS_LIST = `
       SELECT 
              id, 
              method_code, 
              name, 
              description, 
              security_level, 
              is_active, 
              created_at 
         FROM signature_methods
         ORDER BY id LIMIT $1 OFFSET $2
       ;`
 
   SQL_SIGNATURE_METHODS_INSERT = `
       INSERT INTO signature_methods(
              method_code, 
              name, 
              description, 
              security_level, 
              is_active, 
              created_at) 
       VALUES( 
              $1, 
              $2, 
              $3, 
              $4, 
              $5, 
              $6) RETURNING id;` 

   SQL_SIGNATURE_METHODS_UPDATE = `
       UPDATE signature_methods SET 
              method_code = $1, 
              name = $2, 
              description = $3, 
              security_level = $4, 
              is_active = $5, 
              created_at = $6 
        WHERE id = $7; `

   SQL_SIGNATURE_METHODS_DELETE_BY_ID = `
       DELETE FROM signature_methods
        WHERE id = $1; `

   SQL_GET_SIGNATURE_METHODS_BY_ID = `
       SELECT 
              id, 
              method_code, 
              name, 
              description, 
              security_level, 
              is_active, 
              created_at 
         FROM signature_methods
        WHERE id = $1; `
 
   SQL_GET_SIGNATURE_METHODS_BY_METHOD_CODE = `
       SELECT 
              id, 
              method_code, 
              name, 
              description, 
              security_level, 
              is_active, 
              created_at 
         FROM signature_methods
        WHERE Method_code  LIKE '%' || $1 || '%' ; `
 
)