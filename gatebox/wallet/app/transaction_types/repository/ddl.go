package transaction_typesRepo

var(
   SQL_TRANSACTION_TYPES_LIST = `
       SELECT 
              id, 
              type_code, 
              name, 
              description, 
              affects_balance, 
              requires_recipient, 
              is_active, 
              created_at 
         FROM transaction_types
         ORDER BY id LIMIT $1 OFFSET $2
       ;`
 
   SQL_TRANSACTION_TYPES_INSERT = `
       INSERT INTO transaction_types(
              type_code, 
              name, 
              description, 
              affects_balance, 
              requires_recipient, 
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

   SQL_TRANSACTION_TYPES_UPDATE = `
       UPDATE transaction_types SET 
              type_code = $1, 
              name = $2, 
              description = $3, 
              affects_balance = $4, 
              requires_recipient = $5, 
              is_active = $6, 
              created_at = $7 
        WHERE id = $8; `

   SQL_TRANSACTION_TYPES_DELETE_BY_ID = `
       DELETE FROM transaction_types
        WHERE id = $1; `

   SQL_GET_TRANSACTION_TYPES_BY_ID = `
       SELECT 
              id, 
              type_code, 
              name, 
              description, 
              affects_balance, 
              requires_recipient, 
              is_active, 
              created_at 
         FROM transaction_types
        WHERE id = $1; `
 
   SQL_GET_TRANSACTION_TYPES_BY_TYPE_CODE = `
       SELECT 
              id, 
              type_code, 
              name, 
              description, 
              affects_balance, 
              requires_recipient, 
              is_active, 
              created_at 
         FROM transaction_types
        WHERE Type_code  LIKE '%' || $1 || '%' ; `
 
)