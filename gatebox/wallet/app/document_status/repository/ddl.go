package document_statusRepo

var(
   SQL_DOCUMENT_STATUS_LIST = `
       SELECT 
              id, 
              status_code, 
              name, 
              description, 
              is_final, 
              requires_action, 
              next_possible_status, 
              is_active, 
              created_at 
         FROM document_status
         ORDER BY id LIMIT $1 OFFSET $2
       ;`
 
   SQL_DOCUMENT_STATUS_INSERT = `
       INSERT INTO document_status(
              status_code, 
              name, 
              description, 
              is_final, 
              requires_action, 
              next_possible_status, 
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

   SQL_DOCUMENT_STATUS_UPDATE = `
       UPDATE document_status SET 
              status_code = $1, 
              name = $2, 
              description = $3, 
              is_final = $4, 
              requires_action = $5, 
              next_possible_status = $6, 
              is_active = $7, 
              created_at = $8 
        WHERE id = $9; `

   SQL_DOCUMENT_STATUS_DELETE_BY_ID = `
       DELETE FROM document_status
        WHERE id = $1; `

   SQL_GET_DOCUMENT_STATUS_BY_ID = `
       SELECT 
              id, 
              status_code, 
              name, 
              description, 
              is_final, 
              requires_action, 
              next_possible_status, 
              is_active, 
              created_at 
         FROM document_status
        WHERE id = $1; `
 
   SQL_GET_DOCUMENT_STATUS_BY_STATUS_CODE = `
       SELECT 
              id, 
              status_code, 
              name, 
              description, 
              is_final, 
              requires_action, 
              next_possible_status, 
              is_active, 
              created_at 
         FROM document_status
        WHERE Status_code  LIKE '%' || $1 || '%' ; `
 
)