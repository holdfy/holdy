package system_configurationsRepo

var(
   SQL_SYSTEM_CONFIGURATIONS_LIST = `
       SELECT 
              id, 
              config_code, 
              application_id, 
              config_key, 
              config_value, 
              config_type, 
              description, 
              is_active, 
              created_at, 
              updated_at 
         FROM system_configurations
         ORDER BY id LIMIT $1 OFFSET $2
       ;`
 
   SQL_SYSTEM_CONFIGURATIONS_INSERT = `
       INSERT INTO system_configurations(
              config_code, 
              application_id, 
              config_key, 
              config_value, 
              config_type, 
              description, 
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
              $9) RETURNING id;` 

   SQL_SYSTEM_CONFIGURATIONS_UPDATE = `
       UPDATE system_configurations SET 
              config_code = $1, 
              application_id = $2, 
              config_key = $3, 
              config_value = $4, 
              config_type = $5, 
              description = $6, 
              is_active = $7, 
              created_at = $8, 
              updated_at = $9 
        WHERE id = $10; `

   SQL_SYSTEM_CONFIGURATIONS_DELETE_BY_ID = `
       DELETE FROM system_configurations
        WHERE id = $1; `

   SQL_GET_SYSTEM_CONFIGURATIONS_BY_ID = `
       SELECT 
              id, 
              config_code, 
              application_id, 
              config_key, 
              config_value, 
              config_type, 
              description, 
              is_active, 
              created_at, 
              updated_at 
         FROM system_configurations
        WHERE id = $1; `
 
   SQL_GET_SYSTEM_CONFIGURATIONS_BY_CONFIG_CODE = `
       SELECT 
              id, 
              config_code, 
              application_id, 
              config_key, 
              config_value, 
              config_type, 
              description, 
              is_active, 
              created_at, 
              updated_at 
         FROM system_configurations
        WHERE Config_code  LIKE '%' || $1 || '%' ; `
 
)