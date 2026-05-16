package banksRepo

var(
   SQL_BANKS_LIST = `
       SELECT 
              id, 
              bank_code_internal, 
              bank_code, 
              name, 
              full_name, 
              website, 
              is_open_finance, 
              is_active, 
              created_at 
         FROM banks
         ORDER BY id LIMIT $1 OFFSET $2
       ;`
 
   SQL_BANKS_INSERT = `
       INSERT INTO banks(
              bank_code_internal, 
              bank_code, 
              name, 
              full_name, 
              website, 
              is_open_finance, 
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

   SQL_BANKS_UPDATE = `
       UPDATE banks SET 
              bank_code_internal = $1, 
              bank_code = $2, 
              name = $3, 
              full_name = $4, 
              website = $5, 
              is_open_finance = $6, 
              is_active = $7, 
              created_at = $8 
        WHERE id = $9; `

   SQL_BANKS_DELETE_BY_ID = `
       DELETE FROM banks
        WHERE id = $1; `

   SQL_GET_BANKS_BY_ID = `
       SELECT 
              id, 
              bank_code_internal, 
              bank_code, 
              name, 
              full_name, 
              website, 
              is_open_finance, 
              is_active, 
              created_at 
         FROM banks
        WHERE id = $1; `
 
   SQL_GET_BANKS_BY_BANK_CODE_INTERNAL = `
       SELECT 
              id, 
              bank_code_internal, 
              bank_code, 
              name, 
              full_name, 
              website, 
              is_open_finance, 
              is_active, 
              created_at 
         FROM banks
        WHERE Bank_code_internal  LIKE '%' || $1 || '%' ; `
 
)