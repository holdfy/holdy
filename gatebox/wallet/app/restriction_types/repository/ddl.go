package restriction_typesRepo

var(
   SQL_RESTRICTION_TYPES_LIST = `
       SELECT 
              id, 
              type_code, 
              name, 
              description, 
              affects_transactions, 
              affects_login, 
              affects_biometric, 
              can_auto_expire, 
              default_duration_hours, 
              severity_level, 
              is_active, 
              created_at 
         FROM restriction_types
         ORDER BY id LIMIT $1 OFFSET $2
       ;`
 
   SQL_RESTRICTION_TYPES_INSERT = `
       INSERT INTO restriction_types(
              type_code, 
              name, 
              description, 
              affects_transactions, 
              affects_login, 
              affects_biometric, 
              can_auto_expire, 
              default_duration_hours, 
              severity_level, 
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
              $8, 
              $9, 
              $10, 
              $11) RETURNING id;` 

   SQL_RESTRICTION_TYPES_UPDATE = `
       UPDATE restriction_types SET 
              type_code = $1, 
              name = $2, 
              description = $3, 
              affects_transactions = $4, 
              affects_login = $5, 
              affects_biometric = $6, 
              can_auto_expire = $7, 
              default_duration_hours = $8, 
              severity_level = $9, 
              is_active = $10, 
              created_at = $11 
        WHERE id = $12; `

   SQL_RESTRICTION_TYPES_DELETE_BY_ID = `
       DELETE FROM restriction_types
        WHERE id = $1; `

   SQL_GET_RESTRICTION_TYPES_BY_ID = `
       SELECT 
              id, 
              type_code, 
              name, 
              description, 
              affects_transactions, 
              affects_login, 
              affects_biometric, 
              can_auto_expire, 
              default_duration_hours, 
              severity_level, 
              is_active, 
              created_at 
         FROM restriction_types
        WHERE id = $1; `
 
   SQL_GET_RESTRICTION_TYPES_BY_TYPE_CODE = `
       SELECT 
              id, 
              type_code, 
              name, 
              description, 
              affects_transactions, 
              affects_login, 
              affects_biometric, 
              can_auto_expire, 
              default_duration_hours, 
              severity_level, 
              is_active, 
              created_at 
         FROM restriction_types
        WHERE Type_code  LIKE '%' || $1 || '%' ; `
 
)