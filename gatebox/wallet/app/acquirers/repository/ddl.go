package acquirersRepo

var(
   SQL_ACQUIRERS_LIST = `
       SELECT 
              id, 
              acquirer_code, 
              name, 
              description, 
              api_endpoint, 
              is_active, 
              created_at 
         FROM acquirers
         ORDER BY id LIMIT $1 OFFSET $2
       ;`
 
   SQL_ACQUIRERS_INSERT = `
       INSERT INTO acquirers(
              acquirer_code, 
              name, 
              description, 
              api_endpoint, 
              is_active, 
              created_at) 
       VALUES( 
              $1, 
              $2, 
              $3, 
              $4, 
              $5, 
              $6) RETURNING id;` 

   SQL_ACQUIRERS_UPDATE = `
       UPDATE acquirers SET 
              acquirer_code = $1, 
              name = $2, 
              description = $3, 
              api_endpoint = $4, 
              is_active = $5, 
              created_at = $6 
        WHERE id = $7; `

   SQL_ACQUIRERS_DELETE_BY_ID = `
       DELETE FROM acquirers
        WHERE id = $1; `

   SQL_GET_ACQUIRERS_BY_ID = `
       SELECT 
              id, 
              acquirer_code, 
              name, 
              description, 
              api_endpoint, 
              is_active, 
              created_at 
         FROM acquirers
        WHERE id = $1; `
 
   SQL_GET_ACQUIRERS_BY_ACQUIRER_CODE = `
       SELECT 
              id, 
              acquirer_code, 
              name, 
              description, 
              api_endpoint, 
              is_active, 
              created_at 
         FROM acquirers
        WHERE Acquirer_code  LIKE '%' || $1 || '%' ; `
 
)