package usersRepo

var(
   SQL_USERS_LIST = `
       SELECT 
              id, 
              user_code, 
              cpf, 
              full_name, 
              email, 
              phone, 
              birth_date, 
              id_status, 
              id_kyc_status, 
              kyc_level, 
              app_password_hash, 
              biometric_failures, 
              last_login, 
              created_at, 
              updated_at 
         FROM users
         ORDER BY id LIMIT $1 OFFSET $2
       ;`
 
   SQL_USERS_INSERT = `
       INSERT INTO users(
              user_code, 
              cpf, 
              full_name, 
              email, 
              phone, 
              birth_date, 
              id_status, 
              id_kyc_status, 
              kyc_level, 
              app_password_hash, 
              biometric_failures, 
              last_login, 
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

   SQL_USERS_UPDATE = `
       UPDATE users SET 
              user_code = $1, 
              cpf = $2, 
              full_name = $3, 
              email = $4, 
              phone = $5, 
              birth_date = $6, 
              id_status = $7, 
              id_kyc_status = $8, 
              kyc_level = $9, 
              app_password_hash = $10, 
              biometric_failures = $11, 
              last_login = $12, 
              created_at = $13, 
              updated_at = $14 
        WHERE id = $15; `

   SQL_USERS_DELETE_BY_ID = `
       DELETE FROM users
        WHERE id = $1; `

   SQL_GET_USERS_BY_ID = `
       SELECT 
              id, 
              user_code, 
              cpf, 
              full_name, 
              email, 
              phone, 
              birth_date, 
              id_status, 
              id_kyc_status, 
              kyc_level, 
              app_password_hash, 
              biometric_failures, 
              last_login, 
              created_at, 
              updated_at 
         FROM users
        WHERE id = $1; `
 
   SQL_GET_USERS_BY_USER_CODE = `
       SELECT 
              id, 
              user_code, 
              cpf, 
              full_name, 
              email, 
              phone, 
              birth_date, 
              id_status, 
              id_kyc_status, 
              kyc_level, 
              app_password_hash, 
              biometric_failures, 
              last_login, 
              created_at, 
              updated_at 
         FROM users
        WHERE User_code  LIKE '%' || $1 || '%' ; `
 
)