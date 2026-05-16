package account_typesRepo

var(
   SQL_ACCOUNT_TYPES_LIST = `
       SELECT 
              id, 
              type_code, 
              name, 
              description, 
              requires_verification, 
              is_active, 
              created_at 
         FROM account_types
         ORDER BY id LIMIT $1 OFFSET $2
       ;`
 
   SQL_ACCOUNT_TYPES_INSERT = `
       INSERT INTO account_types(
              type_code, 
              name, 
              description, 
              requires_verification, 
              is_active, 
              created_at) 
       VALUES( 
              $1, 
              $2, 
              $3, 
              $4, 
              $5, 
              $6) RETURNING id;` 

   SQL_ACCOUNT_TYPES_UPDATE = `
       UPDATE account_types SET 
              type_code = $1, 
              name = $2, 
              description = $3, 
              requires_verification = $4, 
              is_active = $5, 
              created_at = $6 
        WHERE id = $7; `

   SQL_ACCOUNT_TYPES_DELETE_BY_ID = `
       DELETE FROM account_types
        WHERE id = $1; `

   SQL_GET_ACCOUNT_TYPES_BY_ID = `
       SELECT 
              id, 
              type_code, 
              name, 
              description, 
              requires_verification, 
              is_active, 
              created_at 
         FROM account_types
        WHERE id = $1; `
 
   SQL_GET_ACCOUNT_TYPES_BY_TYPE_CODE = `
       SELECT 
              id, 
              type_code, 
              name, 
              description, 
              requires_verification, 
              is_active, 
              created_at 
         FROM account_types
        WHERE Type_code  LIKE '%' || $1 || '%' ; `
 
)