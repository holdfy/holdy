package card_typesRepo

var(
   SQL_CARD_TYPES_LIST = `
       SELECT 
              id, 
              type_code, 
              name, 
              description, 
              default_daily_limit, 
              default_monthly_limit, 
              is_active, 
              created_at 
         FROM card_types
         ORDER BY id LIMIT $1 OFFSET $2
       ;`
 
   SQL_CARD_TYPES_INSERT = `
       INSERT INTO card_types(
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

   SQL_CARD_TYPES_UPDATE = `
       UPDATE card_types SET 
              type_code = $1, 
              name = $2, 
              description = $3, 
              default_daily_limit = $4, 
              default_monthly_limit = $5, 
              is_active = $6, 
              created_at = $7 
        WHERE id = $8; `

   SQL_CARD_TYPES_DELETE_BY_ID = `
       DELETE FROM card_types
        WHERE id = $1; `

   SQL_GET_CARD_TYPES_BY_ID = `
       SELECT 
              id, 
              type_code, 
              name, 
              description, 
              default_daily_limit, 
              default_monthly_limit, 
              is_active, 
              created_at 
         FROM card_types
        WHERE id = $1; `
 
   SQL_GET_CARD_TYPES_BY_TYPE_CODE = `
       SELECT 
              id, 
              type_code, 
              name, 
              description, 
              default_daily_limit, 
              default_monthly_limit, 
              is_active, 
              created_at 
         FROM card_types
        WHERE Type_code  LIKE '%' || $1 || '%' ; `
 
)