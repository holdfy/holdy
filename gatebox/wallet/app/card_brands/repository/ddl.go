package card_brandsRepo

var(
   SQL_CARD_BRANDS_LIST = `
       SELECT 
              id, 
              brand_code, 
              name, 
              description, 
              logo_url, 
              is_active, 
              created_at 
         FROM card_brands
         ORDER BY id LIMIT $1 OFFSET $2
       ;`
 
   SQL_CARD_BRANDS_INSERT = `
       INSERT INTO card_brands(
              brand_code, 
              name, 
              description, 
              logo_url, 
              is_active, 
              created_at) 
       VALUES( 
              $1, 
              $2, 
              $3, 
              $4, 
              $5, 
              $6) RETURNING id;` 

   SQL_CARD_BRANDS_UPDATE = `
       UPDATE card_brands SET 
              brand_code = $1, 
              name = $2, 
              description = $3, 
              logo_url = $4, 
              is_active = $5, 
              created_at = $6 
        WHERE id = $7; `

   SQL_CARD_BRANDS_DELETE_BY_ID = `
       DELETE FROM card_brands
        WHERE id = $1; `

   SQL_GET_CARD_BRANDS_BY_ID = `
       SELECT 
              id, 
              brand_code, 
              name, 
              description, 
              logo_url, 
              is_active, 
              created_at 
         FROM card_brands
        WHERE id = $1; `
 
   SQL_GET_CARD_BRANDS_BY_BRAND_CODE = `
       SELECT 
              id, 
              brand_code, 
              name, 
              description, 
              logo_url, 
              is_active, 
              created_at 
         FROM card_brands
        WHERE Brand_code  LIKE '%' || $1 || '%' ; `
 
)