package audit_actionsRepo

var(
   SQL_AUDIT_ACTIONS_LIST = `
       SELECT 
              id, 
              action_code, 
              name, 
              description, 
              severity_level, 
              requires_user, 
              is_active, 
              created_at 
         FROM audit_actions
         ORDER BY id LIMIT $1 OFFSET $2
       ;`
 
   SQL_AUDIT_ACTIONS_INSERT = `
       INSERT INTO audit_actions(
              action_code, 
              name, 
              description, 
              severity_level, 
              requires_user, 
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

   SQL_AUDIT_ACTIONS_UPDATE = `
       UPDATE audit_actions SET 
              action_code = $1, 
              name = $2, 
              description = $3, 
              severity_level = $4, 
              requires_user = $5, 
              is_active = $6, 
              created_at = $7 
        WHERE id = $8; `

   SQL_AUDIT_ACTIONS_DELETE_BY_ID = `
       DELETE FROM audit_actions
        WHERE id = $1; `

   SQL_GET_AUDIT_ACTIONS_BY_ID = `
       SELECT 
              id, 
              action_code, 
              name, 
              description, 
              severity_level, 
              requires_user, 
              is_active, 
              created_at 
         FROM audit_actions
        WHERE id = $1; `
 
   SQL_GET_AUDIT_ACTIONS_BY_ACTION_CODE = `
       SELECT 
              id, 
              action_code, 
              name, 
              description, 
              severity_level, 
              requires_user, 
              is_active, 
              created_at 
         FROM audit_actions
        WHERE Action_code  LIKE '%' || $1 || '%' ; `
 
)