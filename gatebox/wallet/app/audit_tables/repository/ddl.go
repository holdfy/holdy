package audit_tablesRepo

var(
   SQL_AUDIT_TABLES_LIST = `
       SELECT 
              id, 
              table_code, 
              name, 
              description, 
              sensitivity_level, 
              retention_days, 
              is_active, 
              created_at 
         FROM audit_tables
         ORDER BY id LIMIT $1 OFFSET $2
       ;`
 
   SQL_AUDIT_TABLES_INSERT = `
       INSERT INTO audit_tables(
              table_code, 
              name, 
              description, 
              sensitivity_level, 
              retention_days, 
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

   SQL_AUDIT_TABLES_UPDATE = `
       UPDATE audit_tables SET 
              table_code = $1, 
              name = $2, 
              description = $3, 
              sensitivity_level = $4, 
              retention_days = $5, 
              is_active = $6, 
              created_at = $7 
        WHERE id = $8; `

   SQL_AUDIT_TABLES_DELETE_BY_ID = `
       DELETE FROM audit_tables
        WHERE id = $1; `

   SQL_GET_AUDIT_TABLES_BY_ID = `
       SELECT 
              id, 
              table_code, 
              name, 
              description, 
              sensitivity_level, 
              retention_days, 
              is_active, 
              created_at 
         FROM audit_tables
        WHERE id = $1; `
 
   SQL_GET_AUDIT_TABLES_BY_TABLE_CODE = `
       SELECT 
              id, 
              table_code, 
              name, 
              description, 
              sensitivity_level, 
              retention_days, 
              is_active, 
              created_at 
         FROM audit_tables
        WHERE Table_code  LIKE '%' || $1 || '%' ; `
 
)