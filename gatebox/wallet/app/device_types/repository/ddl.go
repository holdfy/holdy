package device_typesRepo

var(
   SQL_DEVICE_TYPES_LIST = `
       SELECT 
              id, 
              type_code, 
              name, 
              description, 
              is_mobile, 
              security_level, 
              is_active, 
              created_at 
         FROM device_types
         ORDER BY id LIMIT $1 OFFSET $2
       ;`
 
   SQL_DEVICE_TYPES_INSERT = `
       INSERT INTO device_types(
              type_code, 
              name, 
              description, 
              is_mobile, 
              security_level, 
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

   SQL_DEVICE_TYPES_UPDATE = `
       UPDATE device_types SET 
              type_code = $1, 
              name = $2, 
              description = $3, 
              is_mobile = $4, 
              security_level = $5, 
              is_active = $6, 
              created_at = $7 
        WHERE id = $8; `

   SQL_DEVICE_TYPES_DELETE_BY_ID = `
       DELETE FROM device_types
        WHERE id = $1; `

   SQL_GET_DEVICE_TYPES_BY_ID = `
       SELECT 
              id, 
              type_code, 
              name, 
              description, 
              is_mobile, 
              security_level, 
              is_active, 
              created_at 
         FROM device_types
        WHERE id = $1; `
 
   SQL_GET_DEVICE_TYPES_BY_TYPE_CODE = `
       SELECT 
              id, 
              type_code, 
              name, 
              description, 
              is_mobile, 
              security_level, 
              is_active, 
              created_at 
         FROM device_types
        WHERE Type_code  LIKE '%' || $1 || '%' ; `
 
)