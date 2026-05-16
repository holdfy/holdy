package change_actorsRepo

var(
   SQL_CHANGE_ACTORS_LIST = `
       SELECT 
              id, 
              actor_code, 
              name, 
              description, 
              can_auto_approve, 
              priority_level, 
              is_active, 
              created_at 
         FROM change_actors
         ORDER BY id LIMIT $1 OFFSET $2
       ;`
 
   SQL_CHANGE_ACTORS_INSERT = `
       INSERT INTO change_actors(
              actor_code, 
              name, 
              description, 
              can_auto_approve, 
              priority_level, 
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

   SQL_CHANGE_ACTORS_UPDATE = `
       UPDATE change_actors SET 
              actor_code = $1, 
              name = $2, 
              description = $3, 
              can_auto_approve = $4, 
              priority_level = $5, 
              is_active = $6, 
              created_at = $7 
        WHERE id = $8; `

   SQL_CHANGE_ACTORS_DELETE_BY_ID = `
       DELETE FROM change_actors
        WHERE id = $1; `

   SQL_GET_CHANGE_ACTORS_BY_ID = `
       SELECT 
              id, 
              actor_code, 
              name, 
              description, 
              can_auto_approve, 
              priority_level, 
              is_active, 
              created_at 
         FROM change_actors
        WHERE id = $1; `
 
   SQL_GET_CHANGE_ACTORS_BY_ACTOR_CODE = `
       SELECT 
              id, 
              actor_code, 
              name, 
              description, 
              can_auto_approve, 
              priority_level, 
              is_active, 
              created_at 
         FROM change_actors
        WHERE Actor_code  LIKE '%' || $1 || '%' ; `
 
)