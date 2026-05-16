package currenciesRepo

var(
   SQL_CURRENCIES_LIST = `
       SELECT 
              id, 
              currency_code, 
              iso_code, 
              name, 
              symbol, 
              decimal_places, 
              is_active, 
              created_at 
         FROM currencies
         ORDER BY id LIMIT $1 OFFSET $2
       ;`
 
   SQL_CURRENCIES_INSERT = `
       INSERT INTO currencies(
              currency_code, 
              iso_code, 
              name, 
              symbol, 
              decimal_places, 
              is_active, 
              created_at) 
       VALUES( 
              $1, 
              $2, 
              $3, 
              $4, 
              $5, 
              $6, 
              $7) RETURNING id;` 

   SQL_CURRENCIES_UPDATE = `
       UPDATE currencies SET 
              currency_code = $1, 
              iso_code = $2, 
              name = $3, 
              symbol = $4, 
              decimal_places = $5, 
              is_active = $6, 
              created_at = $7 
        WHERE id = $8; `

   SQL_CURRENCIES_DELETE_BY_ID = `
       DELETE FROM currencies
        WHERE id = $1; `

   SQL_GET_CURRENCIES_BY_ID = `
       SELECT 
              id, 
              currency_code, 
              iso_code, 
              name, 
              symbol, 
              decimal_places, 
              is_active, 
              created_at 
         FROM currencies
        WHERE id = $1; `
 
   SQL_GET_CURRENCIES_BY_CURRENCY_CODE = `
       SELECT 
              id, 
              currency_code, 
              iso_code, 
              name, 
              symbol, 
              decimal_places, 
              is_active, 
              created_at 
         FROM currencies
        WHERE Currency_code  LIKE '%' || $1 || '%' ; `
 
)