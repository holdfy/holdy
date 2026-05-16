package notification_templatesRepo

var(
   SQL_NOTIFICATION_TEMPLATES_LIST = `
       SELECT 
              id, 
              template_code, 
              application_id, 
              template_key, 
              id_channel, 
              subject, 
              template_body, 
              template_variables, 
              is_active, 
              created_at, 
              updated_at 
         FROM notification_templates
         ORDER BY id LIMIT $1 OFFSET $2
       ;`
 
   SQL_NOTIFICATION_TEMPLATES_INSERT = `
       INSERT INTO notification_templates(
              template_code, 
              application_id, 
              template_key, 
              id_channel, 
              subject, 
              template_body, 
              template_variables, 
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
              $10) RETURNING id;` 

   SQL_NOTIFICATION_TEMPLATES_UPDATE = `
       UPDATE notification_templates SET 
              template_code = $1, 
              application_id = $2, 
              template_key = $3, 
              id_channel = $4, 
              subject = $5, 
              template_body = $6, 
              template_variables = $7, 
              is_active = $8, 
              created_at = $9, 
              updated_at = $10 
        WHERE id = $11; `

   SQL_NOTIFICATION_TEMPLATES_DELETE_BY_ID = `
       DELETE FROM notification_templates
        WHERE id = $1; `

   SQL_GET_NOTIFICATION_TEMPLATES_BY_ID = `
       SELECT 
              id, 
              template_code, 
              application_id, 
              template_key, 
              id_channel, 
              subject, 
              template_body, 
              template_variables, 
              is_active, 
              created_at, 
              updated_at 
         FROM notification_templates
        WHERE id = $1; `
 
   SQL_GET_NOTIFICATION_TEMPLATES_BY_TEMPLATE_CODE = `
       SELECT 
              id, 
              template_code, 
              application_id, 
              template_key, 
              id_channel, 
              subject, 
              template_body, 
              template_variables, 
              is_active, 
              created_at, 
              updated_at 
         FROM notification_templates
        WHERE Template_code  LIKE '%' || $1 || '%' ; `
 
)