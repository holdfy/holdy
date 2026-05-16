package balance_change_typesRepo

var(
   SQL_BALANCE_CHANGE_TYPES_LIST = `
       SELECT 
              id, 
              type_code, 
              name, 
              description, 
              is_positive, 
              is_active, 
              created_at 
         FROM balance_change_types
         ORDER BY id LIMIT $1 OFFSET $2
       ;`
 
   SQL_BALANCE_CHANGE_TYPES_INSERT = `
       INSERT INTO balance_change_types(
              type_code, 
              name, 
              description, 
              is_positive, 
              is_active, 
              created_at) 
       VALUES( 
              $1, 
              $2, 
              $3, 
              $4, 
              $5, 
              $6) RETURNING id;` 

   SQL_BALANCE_CHANGE_TYPES_UPDATE = `
       UPDATE balance_change_types SET 
              type_code = $1, 
              name = $2, 
              description = $3, 
              is_positive = $4, 
              is_active = $5, 
              created_at = $6 
        WHERE id = $7; `

   SQL_BALANCE_CHANGE_TYPES_DELETE_BY_ID = `
       DELETE FROM balance_change_types
        WHERE id = $1; `

   SQL_GET_BALANCE_CHANGE_TYPES_BY_ID = `
       SELECT 
              id, 
              type_code, 
              name, 
              description, 
              is_positive, 
              is_active, 
              created_at 
         FROM balance_change_types
        WHERE id = $1; `
 
   SQL_GET_BALANCE_CHANGE_TYPES_BY_TYPE_CODE = `
       SELECT 
              id, 
              type_code, 
              name, 
              description, 
              is_positive, 
              is_active, 
              created_at 
         FROM balance_change_types
        WHERE Type_code  LIKE '%' || $1 || '%' ; `
 
)