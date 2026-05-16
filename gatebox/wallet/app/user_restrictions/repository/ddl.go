package user_restrictionsRepo

var(
   SQL_USER_RESTRICTIONS_LIST = `
       SELECT 
              id, 
              restriction_code, 
              user_id, 
              id_restriction_type, 
              restriction_reason, 
              restrictions, 
              is_active, 
              expires_at, 
              created_by, 
              removed_by, 
              removed_at, 
              created_at 
         FROM user_restrictions
         ORDER BY id LIMIT $1 OFFSET $2
       ;`
 
   SQL_USER_RESTRICTIONS_INSERT = `
       INSERT INTO user_restrictions(
              restriction_code, 
              user_id, 
              id_restriction_type, 
              restriction_reason, 
              restrictions, 
              is_active, 
              expires_at, 
              created_by, 
              removed_by, 
              removed_at, 
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

   SQL_USER_RESTRICTIONS_UPDATE = `
       UPDATE user_restrictions SET 
              restriction_code = $1, 
              user_id = $2, 
              id_restriction_type = $3, 
              restriction_reason = $4, 
              restrictions = $5, 
              is_active = $6, 
              expires_at = $7, 
              created_by = $8, 
              removed_by = $9, 
              removed_at = $10, 
              created_at = $11 
        WHERE id = $12; `

   SQL_USER_RESTRICTIONS_DELETE_BY_ID = `
       DELETE FROM user_restrictions
        WHERE id = $1; `

   SQL_GET_USER_RESTRICTIONS_BY_ID = `
       SELECT 
              id, 
              restriction_code, 
              user_id, 
              id_restriction_type, 
              restriction_reason, 
              restrictions, 
              is_active, 
              expires_at, 
              created_by, 
              removed_by, 
              removed_at, 
              created_at 
         FROM user_restrictions
        WHERE id = $1; `
 
   SQL_GET_USER_RESTRICTIONS_BY_RESTRICTION_CODE = `
       SELECT 
              id, 
              restriction_code, 
              user_id, 
              id_restriction_type, 
              restriction_reason, 
              restrictions, 
              is_active, 
              expires_at, 
              created_by, 
              removed_by, 
              removed_at, 
              created_at 
         FROM user_restrictions
        WHERE Restriction_code  LIKE '%' || $1 || '%' ; `
 
)