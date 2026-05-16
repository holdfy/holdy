package hand_typesRepo

var(
   SQL_HAND_TYPES_LIST = `
       SELECT 
              id, 
              type_code, 
              name, 
              description, 
              is_active, 
              created_at 
         FROM hand_types
         ORDER BY id LIMIT $1 OFFSET $2
       ;`
 
   SQL_HAND_TYPES_INSERT = `
       INSERT INTO hand_types(
              type_code, 
              name, 
              description, 
              is_active, 
              created_at) 
       VALUES( 
              $1, 
              $2, 
              $3, 
              $4, 
              $5) RETURNING id;` 

   SQL_HAND_TYPES_UPDATE = `
       UPDATE hand_types SET 
              type_code = $1, 
              name = $2, 
              description = $3, 
              is_active = $4, 
              created_at = $5 
        WHERE id = $6; `

   SQL_HAND_TYPES_DELETE_BY_ID = `
       DELETE FROM hand_types
        WHERE id = $1; `

   SQL_GET_HAND_TYPES_BY_ID = `
       SELECT 
              id, 
              type_code, 
              name, 
              description, 
              is_active, 
              created_at 
         FROM hand_types
        WHERE id = $1; `
 
   SQL_GET_HAND_TYPES_BY_TYPE_CODE = `
       SELECT 
              id, 
              type_code, 
              name, 
              description, 
              is_active, 
              created_at 
         FROM hand_types
        WHERE Type_code  LIKE '%' || $1 || '%' ; `
 
)