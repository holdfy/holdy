package applicationsRepo

var(
   SQL_APPLICATIONS_LIST = `
       SELECT 
              id, 
              app_code, 
              name, 
              code, 
              description, 
              api_key, 
              is_active, 
              settings, 
              created_at, 
              updated_at 
         FROM applications
         ORDER BY id LIMIT $1 OFFSET $2
       ;`
 
   SQL_APPLICATIONS_INSERT = `
       INSERT INTO applications(
              app_code, 
              name, 
              code, 
              description, 
              api_key, 
              is_active, 
              settings, 
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
              $9) RETURNING id;` 

   SQL_APPLICATIONS_UPDATE = `
       UPDATE applications SET 
              app_code = $1, 
              name = $2, 
              code = $3, 
              description = $4, 
              api_key = $5, 
              is_active = $6, 
              settings = $7, 
              created_at = $8, 
              updated_at = $9 
        WHERE id = $10; `

   SQL_APPLICATIONS_DELETE_BY_ID = `
       DELETE FROM applications
        WHERE id = $1; `

   SQL_GET_APPLICATIONS_BY_ID = `
       SELECT 
              id, 
              app_code, 
              name, 
              code, 
              description, 
              api_key, 
              is_active, 
              settings, 
              created_at, 
              updated_at 
         FROM applications
        WHERE id = $1; `
 
   SQL_GET_APPLICATIONS_BY_APP_CODE = `
       SELECT 
              id, 
              app_code, 
              name, 
              code, 
              description, 
              api_key, 
              is_active, 
              settings, 
              created_at, 
              updated_at 
         FROM applications
        WHERE App_code  LIKE '%' || $1 || '%' ; `
 
)