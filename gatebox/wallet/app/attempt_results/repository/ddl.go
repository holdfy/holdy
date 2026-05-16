package attempt_resultsRepo

var(
   SQL_ATTEMPT_RESULTS_LIST = `
       SELECT 
              id, 
              result_code, 
              name, 
              description, 
              is_success, 
              is_active, 
              created_at 
         FROM attempt_results
         ORDER BY id LIMIT $1 OFFSET $2
       ;`
 
   SQL_ATTEMPT_RESULTS_INSERT = `
       INSERT INTO attempt_results(
              result_code, 
              name, 
              description, 
              is_success, 
              is_active, 
              created_at) 
       VALUES( 
              $1, 
              $2, 
              $3, 
              $4, 
              $5, 
              $6) RETURNING id;` 

   SQL_ATTEMPT_RESULTS_UPDATE = `
       UPDATE attempt_results SET 
              result_code = $1, 
              name = $2, 
              description = $3, 
              is_success = $4, 
              is_active = $5, 
              created_at = $6 
        WHERE id = $7; `

   SQL_ATTEMPT_RESULTS_DELETE_BY_ID = `
       DELETE FROM attempt_results
        WHERE id = $1; `

   SQL_GET_ATTEMPT_RESULTS_BY_ID = `
       SELECT 
              id, 
              result_code, 
              name, 
              description, 
              is_success, 
              is_active, 
              created_at 
         FROM attempt_results
        WHERE id = $1; `
 
   SQL_GET_ATTEMPT_RESULTS_BY_RESULT_CODE = `
       SELECT 
              id, 
              result_code, 
              name, 
              description, 
              is_success, 
              is_active, 
              created_at 
         FROM attempt_results
        WHERE Result_code  LIKE '%' || $1 || '%' ; `
 
)