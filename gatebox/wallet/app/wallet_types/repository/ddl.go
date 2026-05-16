package wallet_typesRepo

var(
   SQL_WALLET_TYPES_LIST = `
       SELECT 
              id, 
              type_code, 
              name, 
              description, 
              default_daily_limit, 
              default_monthly_limit, 
              is_active, 
              created_at 
         FROM wallet_types
         ORDER BY id LIMIT $1 OFFSET $2
       ;`
 
   SQL_WALLET_TYPES_INSERT = `
       INSERT INTO wallet_types(
              type_code, 
              name, 
              description, 
              default_daily_limit, 
              default_monthly_limit, 
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

   SQL_WALLET_TYPES_UPDATE = `
       UPDATE wallet_types SET 
              type_code = $1, 
              name = $2, 
              description = $3, 
              default_daily_limit = $4, 
              default_monthly_limit = $5, 
              is_active = $6, 
              created_at = $7 
        WHERE id = $8; `

   SQL_WALLET_TYPES_DELETE_BY_ID = `
       DELETE FROM wallet_types
        WHERE id = $1; `

   SQL_GET_WALLET_TYPES_BY_ID = `
       SELECT 
              id, 
              type_code, 
              name, 
              description, 
              default_daily_limit, 
              default_monthly_limit, 
              is_active, 
              created_at 
         FROM wallet_types
        WHERE id = $1; `
 
   SQL_GET_WALLET_TYPES_BY_TYPE_CODE = `
       SELECT 
              id, 
              type_code, 
              name, 
              description, 
              default_daily_limit, 
              default_monthly_limit, 
              is_active, 
              created_at 
         FROM wallet_types
        WHERE Type_code  LIKE '%' || $1 || '%' ; `
 
)