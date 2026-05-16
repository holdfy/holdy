package user_cardsRepo

var(
   SQL_USER_CARDS_LIST = `
       SELECT 
              id, 
              card_code, 
              user_id, 
              wallet_id, 
              card_token, 
              id_card_brand, 
              masked_number, 
              holder_name, 
              expiry_month, 
              expiry_year, 
              id_card_type, 
              id_acquirer, 
              is_primary, 
              is_active, 
              last_used, 
              created_at, 
              updated_at 
         FROM user_cards
         ORDER BY id LIMIT $1 OFFSET $2
       ;`
 
   SQL_USER_CARDS_INSERT = `
       INSERT INTO user_cards(
              card_code, 
              user_id, 
              wallet_id, 
              card_token, 
              id_card_brand, 
              masked_number, 
              holder_name, 
              expiry_month, 
              expiry_year, 
              id_card_type, 
              id_acquirer, 
              is_primary, 
              is_active, 
              last_used, 
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
              $14, 
              $15, 
              $16) RETURNING id;` 

   SQL_USER_CARDS_UPDATE = `
       UPDATE user_cards SET 
              card_code = $1, 
              user_id = $2, 
              wallet_id = $3, 
              card_token = $4, 
              id_card_brand = $5, 
              masked_number = $6, 
              holder_name = $7, 
              expiry_month = $8, 
              expiry_year = $9, 
              id_card_type = $10, 
              id_acquirer = $11, 
              is_primary = $12, 
              is_active = $13, 
              last_used = $14, 
              created_at = $15, 
              updated_at = $16 
        WHERE id = $17; `

   SQL_USER_CARDS_DELETE_BY_ID = `
       DELETE FROM user_cards
        WHERE id = $1; `

   SQL_GET_USER_CARDS_BY_ID = `
       SELECT 
              id, 
              card_code, 
              user_id, 
              wallet_id, 
              card_token, 
              id_card_brand, 
              masked_number, 
              holder_name, 
              expiry_month, 
              expiry_year, 
              id_card_type, 
              id_acquirer, 
              is_primary, 
              is_active, 
              last_used, 
              created_at, 
              updated_at 
         FROM user_cards
        WHERE id = $1; `
 
   SQL_GET_USER_CARDS_BY_CARD_CODE = `
       SELECT 
              id, 
              card_code, 
              user_id, 
              wallet_id, 
              card_token, 
              id_card_brand, 
              masked_number, 
              holder_name, 
              expiry_month, 
              expiry_year, 
              id_card_type, 
              id_acquirer, 
              is_primary, 
              is_active, 
              last_used, 
              created_at, 
              updated_at 
         FROM user_cards
        WHERE Card_code  LIKE '%' || $1 || '%' ; `
 
)