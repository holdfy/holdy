package user_documentsRepo

var(
   SQL_USER_DOCUMENTS_LIST = `
       SELECT 
              id, 
              document_code, 
              user_id, 
              id_document_type, 
              document_number, 
              file_path, 
              file_hash, 
              id_status_documents, 
              verified_at, 
              verified_by, 
              rejection_reason, 
              metadata, 
              created_at, 
              updated_at 
         FROM user_documents
         ORDER BY id LIMIT $1 OFFSET $2
       ;`
 
   SQL_USER_DOCUMENTS_INSERT = `
       INSERT INTO user_documents(
              document_code, 
              user_id, 
              id_document_type, 
              document_number, 
              file_path, 
              file_hash, 
              id_status_documents, 
              verified_at, 
              verified_by, 
              rejection_reason, 
              metadata, 
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
              $13) RETURNING id;` 

   SQL_USER_DOCUMENTS_UPDATE = `
       UPDATE user_documents SET 
              document_code = $1, 
              user_id = $2, 
              id_document_type = $3, 
              document_number = $4, 
              file_path = $5, 
              file_hash = $6, 
              id_status_documents = $7, 
              verified_at = $8, 
              verified_by = $9, 
              rejection_reason = $10, 
              metadata = $11, 
              created_at = $12, 
              updated_at = $13 
        WHERE id = $14; `

   SQL_USER_DOCUMENTS_DELETE_BY_ID = `
       DELETE FROM user_documents
        WHERE id = $1; `

   SQL_GET_USER_DOCUMENTS_BY_ID = `
       SELECT 
              id, 
              document_code, 
              user_id, 
              id_document_type, 
              document_number, 
              file_path, 
              file_hash, 
              id_status_documents, 
              verified_at, 
              verified_by, 
              rejection_reason, 
              metadata, 
              created_at, 
              updated_at 
         FROM user_documents
        WHERE id = $1; `
 
   SQL_GET_USER_DOCUMENTS_BY_DOCUMENT_CODE = `
       SELECT 
              id, 
              document_code, 
              user_id, 
              id_document_type, 
              document_number, 
              file_path, 
              file_hash, 
              id_status_documents, 
              verified_at, 
              verified_by, 
              rejection_reason, 
              metadata, 
              created_at, 
              updated_at 
         FROM user_documents
        WHERE Document_code  LIKE '%' || $1 || '%' ; `
 
)