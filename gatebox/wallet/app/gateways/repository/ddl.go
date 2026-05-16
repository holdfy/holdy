package gatewaysRepo

var(
   SQL_GATEWAYS_LIST = `
       SELECT 
              id, 
              gateway_code, 
              name, 
              description, 
              api_endpoint, 
              timeout_seconds, 
              max_retries, 
              is_active, 
              created_at 
         FROM gateways
         ORDER BY id LIMIT $1 OFFSET $2
       ;`
 
   SQL_GATEWAYS_INSERT = `
       INSERT INTO gateways(
              gateway_code, 
              name, 
              description, 
              api_endpoint, 
              timeout_seconds, 
              max_retries, 
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
              $8) RETURNING id;` 

   SQL_GATEWAYS_UPDATE = `
       UPDATE gateways SET 
              gateway_code = $1, 
              name = $2, 
              description = $3, 
              api_endpoint = $4, 
              timeout_seconds = $5, 
              max_retries = $6, 
              is_active = $7, 
              created_at = $8 
        WHERE id = $9; `

   SQL_GATEWAYS_DELETE_BY_ID = `
       DELETE FROM gateways
        WHERE id = $1; `

   SQL_GET_GATEWAYS_BY_ID = `
       SELECT 
              id, 
              gateway_code, 
              name, 
              description, 
              api_endpoint, 
              timeout_seconds, 
              max_retries, 
              is_active, 
              created_at 
         FROM gateways
        WHERE id = $1; `
 
   SQL_GET_GATEWAYS_BY_GATEWAY_CODE = `
       SELECT 
              id, 
              gateway_code, 
              name, 
              description, 
              api_endpoint, 
              timeout_seconds, 
              max_retries, 
              is_active, 
              created_at 
         FROM gateways
        WHERE Gateway_code  LIKE '%' || $1 || '%' ; `
 
)