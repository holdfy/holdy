package wallet_providersRepo

var(
   SQL_WALLET_PROVIDERS_LIST = `
       SELECT 
              id, 
              provider_code, 
              name, 
              description, 
              api_endpoint, 
              requires_token, 
              token_duration_hours, 
              is_active, 
              created_at 
         FROM wallet_providers
         ORDER BY id LIMIT $1 OFFSET $2
       ;`
 
   SQL_WALLET_PROVIDERS_INSERT = `
       INSERT INTO wallet_providers(
              provider_code, 
              name, 
              description, 
              api_endpoint, 
              requires_token, 
              token_duration_hours, 
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

   SQL_WALLET_PROVIDERS_UPDATE = `
       UPDATE wallet_providers SET 
              provider_code = $1, 
              name = $2, 
              description = $3, 
              api_endpoint = $4, 
              requires_token = $5, 
              token_duration_hours = $6, 
              is_active = $7, 
              created_at = $8 
        WHERE id = $9; `

   SQL_WALLET_PROVIDERS_DELETE_BY_ID = `
       DELETE FROM wallet_providers
        WHERE id = $1; `

   SQL_GET_WALLET_PROVIDERS_BY_ID = `
       SELECT 
              id, 
              provider_code, 
              name, 
              description, 
              api_endpoint, 
              requires_token, 
              token_duration_hours, 
              is_active, 
              created_at 
         FROM wallet_providers
        WHERE id = $1; `
 
   SQL_GET_WALLET_PROVIDERS_BY_PROVIDER_CODE = `
       SELECT 
              id, 
              provider_code, 
              name, 
              description, 
              api_endpoint, 
              requires_token, 
              token_duration_hours, 
              is_active, 
              created_at 
         FROM wallet_providers
        WHERE Provider_code  LIKE '%' || $1 || '%' ; `
 
)