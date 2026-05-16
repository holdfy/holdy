package user_addressesRepo

var(
   SQL_USER_ADDRESSES_LIST = `
       SELECT 
              id, 
              address_code, 
              user_id, 
              id_address_type, 
              street, 
              number, 
              complement, 
              neighborhood, 
              city, 
              state, 
              zip_code, 
              country, 
              is_primary, 
              is_active, 
              created_at, 
              updated_at 
         FROM user_addresses
         ORDER BY id LIMIT $1 OFFSET $2
       ;`
 
   SQL_USER_ADDRESSES_INSERT = `
       INSERT INTO user_addresses(
              address_code, 
              user_id, 
              id_address_type, 
              street, 
              number, 
              complement, 
              neighborhood, 
              city, 
              state, 
              zip_code, 
              country, 
              is_primary, 
              is_active, 
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
              $15) RETURNING id;` 

   SQL_USER_ADDRESSES_UPDATE = `
       UPDATE user_addresses SET 
              address_code = $1, 
              user_id = $2, 
              id_address_type = $3, 
              street = $4, 
              number = $5, 
              complement = $6, 
              neighborhood = $7, 
              city = $8, 
              state = $9, 
              zip_code = $10, 
              country = $11, 
              is_primary = $12, 
              is_active = $13, 
              created_at = $14, 
              updated_at = $15 
        WHERE id = $16; `

   SQL_USER_ADDRESSES_DELETE_BY_ID = `
       DELETE FROM user_addresses
        WHERE id = $1; `

   SQL_GET_USER_ADDRESSES_BY_ID = `
       SELECT 
              id, 
              address_code, 
              user_id, 
              id_address_type, 
              street, 
              number, 
              complement, 
              neighborhood, 
              city, 
              state, 
              zip_code, 
              country, 
              is_primary, 
              is_active, 
              created_at, 
              updated_at 
         FROM user_addresses
        WHERE id = $1; `
 
   SQL_GET_USER_ADDRESSES_BY_ADDRESS_CODE = `
       SELECT 
              id, 
              address_code, 
              user_id, 
              id_address_type, 
              street, 
              number, 
              complement, 
              neighborhood, 
              city, 
              state, 
              zip_code, 
              country, 
              is_primary, 
              is_active, 
              created_at, 
              updated_at 
         FROM user_addresses
        WHERE Address_code  LIKE '%' || $1 || '%' ; `
 
)