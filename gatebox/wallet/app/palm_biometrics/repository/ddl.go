package palm_biometricsRepo

var(
   SQL_PALM_BIOMETRICS_LIST = `
       SELECT 
              id, 
              biometric_code, 
              user_id, 
              palm_hash, 
              accuracy_score, 
              id_hand_type, 
              enrollment_device_id, 
              bitmap_signature, 
              is_primary, 
              is_active, 
              registered_at, 
              last_used, 
              usage_count, 
              created_at, 
              updated_at 
         FROM palm_biometrics
         ORDER BY id LIMIT $1 OFFSET $2
       ;`
 
   SQL_PALM_BIOMETRICS_INSERT = `
       INSERT INTO palm_biometrics(
              biometric_code, 
              user_id, 
              palm_hash, 
              accuracy_score, 
              id_hand_type, 
              enrollment_device_id, 
              bitmap_signature, 
              is_primary, 
              is_active, 
              registered_at, 
              last_used, 
              usage_count, 
              created_at, 
              updated_at) 
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
              $11, 
              $12, 
              $13, 
              $14) RETURNING id;` 

   SQL_PALM_BIOMETRICS_UPDATE = `
       UPDATE palm_biometrics SET 
              biometric_code = $1, 
              user_id = $2, 
              palm_hash = $3, 
              accuracy_score = $4, 
              id_hand_type = $5, 
              enrollment_device_id = $6, 
              bitmap_signature = $7, 
              is_primary = $8, 
              is_active = $9, 
              registered_at = $10, 
              last_used = $11, 
              usage_count = $12, 
              created_at = $13, 
              updated_at = $14 
        WHERE id = $15; `

   SQL_PALM_BIOMETRICS_DELETE_BY_ID = `
       DELETE FROM palm_biometrics
        WHERE id = $1; `

   SQL_GET_PALM_BIOMETRICS_BY_ID = `
       SELECT 
              id, 
              biometric_code, 
              user_id, 
              palm_hash, 
              accuracy_score, 
              id_hand_type, 
              enrollment_device_id, 
              bitmap_signature, 
              is_primary, 
              is_active, 
              registered_at, 
              last_used, 
              usage_count, 
              created_at, 
              updated_at 
         FROM palm_biometrics
        WHERE id = $1; `
 
   SQL_GET_PALM_BIOMETRICS_BY_BIOMETRIC_CODE = `
       SELECT 
              id, 
              biometric_code, 
              user_id, 
              palm_hash, 
              accuracy_score, 
              id_hand_type, 
              enrollment_device_id, 
              bitmap_signature, 
              is_primary, 
              is_active, 
              registered_at, 
              last_used, 
              usage_count, 
              created_at, 
              updated_at 
         FROM palm_biometrics
        WHERE Biometric_code  LIKE '%' || $1 || '%' ; `
 
)