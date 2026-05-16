package document_typesRepo

var(
   SQL_DOCUMENT_TYPES_LIST = `
       SELECT 
              id, 
              type_code, 
              name, 
              description, 
              is_required, 
              max_file_size_mb, 
              allowed_extensions, 
              is_active, 
              created_at 
         FROM document_types
         ORDER BY id LIMIT $1 OFFSET $2
       ;`
 
   SQL_DOCUMENT_TYPES_INSERT = `
       INSERT INTO document_types(
              type_code, 
              name, 
              description, 
              is_required, 
              max_file_size_mb, 
              allowed_extensions, 
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

   SQL_DOCUMENT_TYPES_UPDATE = `
       UPDATE document_types SET 
              type_code = $1, 
              name = $2, 
              description = $3, 
              is_required = $4, 
              max_file_size_mb = $5, 
              allowed_extensions = $6, 
              is_active = $7, 
              created_at = $8 
        WHERE id = $9; `

   SQL_DOCUMENT_TYPES_DELETE_BY_ID = `
       DELETE FROM document_types
        WHERE id = $1; `

   SQL_GET_DOCUMENT_TYPES_BY_ID = `
       SELECT 
              id, 
              type_code, 
              name, 
              description, 
              is_required, 
              max_file_size_mb, 
              allowed_extensions, 
              is_active, 
              created_at 
         FROM document_types
        WHERE id = $1; `
 
   SQL_GET_DOCUMENT_TYPES_BY_TYPE_CODE = `
       SELECT 
              id, 
              type_code, 
              name, 
              description, 
              is_required, 
              max_file_size_mb, 
              allowed_extensions, 
              is_active, 
              created_at 
         FROM document_types
        WHERE Type_code  LIKE '%' || $1 || '%' ; `
 
)