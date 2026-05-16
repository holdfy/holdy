package biometric_attemptsRepo

var(
   SQL_BIOMETRIC_ATTEMPTS_LIST = `
       SELECT 
              id, 
              attempt_code, 
              user_id, 
              palm_hash, 
              accuracy_score, 
              device_id, 
              id_attempt_result, 
              id_failure_reason, 
              ip_address, 
              user_agent, 
              location_data, 
              created_at 
         FROM biometric_attempts
         ORDER BY id LIMIT $1 OFFSET $2
       ;`
 
   SQL_BIOMETRIC_ATTEMPTS_INSERT = `
       INSERT INTO biometric_attempts(
              attempt_code, 
              user_id, 
              palm_hash, 
              accuracy_score, 
              device_id, 
              id_attempt_result, 
              id_failure_reason, 
              ip_address, 
              user_agent, 
              location_data, 
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

   SQL_BIOMETRIC_ATTEMPTS_UPDATE = `
       UPDATE biometric_attempts SET 
              attempt_code = $1, 
              user_id = $2, 
              palm_hash = $3, 
              accuracy_score = $4, 
              device_id = $5, 
              id_attempt_result = $6, 
              id_failure_reason = $7, 
              ip_address = $8, 
              user_agent = $9, 
              location_data = $10, 
              created_at = $11 
        WHERE id = $12; `

   SQL_BIOMETRIC_ATTEMPTS_DELETE_BY_ID = `
       DELETE FROM biometric_attempts
        WHERE id = $1; `

   SQL_GET_BIOMETRIC_ATTEMPTS_BY_ID = `
       SELECT 
              id, 
              attempt_code, 
              user_id, 
              palm_hash, 
              accuracy_score, 
              device_id, 
              id_attempt_result, 
              id_failure_reason, 
              ip_address, 
              user_agent, 
              location_data, 
              created_at 
         FROM biometric_attempts
        WHERE id = $1; `
 
   SQL_GET_BIOMETRIC_ATTEMPTS_BY_ATTEMPT_CODE = `
       SELECT 
              id, 
              attempt_code, 
              user_id, 
              palm_hash, 
              accuracy_score, 
              device_id, 
              id_attempt_result, 
              id_failure_reason, 
              ip_address, 
              user_agent, 
              location_data, 
              created_at 
         FROM biometric_attempts
        WHERE Attempt_code  LIKE '%' || $1 || '%' ; `
 
)