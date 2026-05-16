package security_eventsRepo

var(
   SQL_SECURITY_EVENTS_LIST = `
       SELECT 
              id, 
              security_event_code, 
              user_id, 
              id_event_type, 
              id_severity, 
              description, 
              source_ip, 
              device_info, 
              metadata, 
              is_resolved, 
              resolved_at, 
              resolved_by, 
              created_at 
         FROM security_events
         ORDER BY id LIMIT $1 OFFSET $2
       ;`
 
   SQL_SECURITY_EVENTS_INSERT = `
       INSERT INTO security_events(
              security_event_code, 
              user_id, 
              id_event_type, 
              id_severity, 
              description, 
              source_ip, 
              device_info, 
              metadata, 
              is_resolved, 
              resolved_at, 
              resolved_by, 
              created_at) 
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
              $12) RETURNING id;` 

   SQL_SECURITY_EVENTS_UPDATE = `
       UPDATE security_events SET 
              security_event_code = $1, 
              user_id = $2, 
              id_event_type = $3, 
              id_severity = $4, 
              description = $5, 
              source_ip = $6, 
              device_info = $7, 
              metadata = $8, 
              is_resolved = $9, 
              resolved_at = $10, 
              resolved_by = $11, 
              created_at = $12 
        WHERE id = $13; `

   SQL_SECURITY_EVENTS_DELETE_BY_ID = `
       DELETE FROM security_events
        WHERE id = $1; `

   SQL_GET_SECURITY_EVENTS_BY_ID = `
       SELECT 
              id, 
              security_event_code, 
              user_id, 
              id_event_type, 
              id_severity, 
              description, 
              source_ip, 
              device_info, 
              metadata, 
              is_resolved, 
              resolved_at, 
              resolved_by, 
              created_at 
         FROM security_events
        WHERE id = $1; `
 
   SQL_GET_SECURITY_EVENTS_BY_SECURITY_EVENT_CODE = `
       SELECT 
              id, 
              security_event_code, 
              user_id, 
              id_event_type, 
              id_severity, 
              description, 
              source_ip, 
              device_info, 
              metadata, 
              is_resolved, 
              resolved_at, 
              resolved_by, 
              created_at 
         FROM security_events
        WHERE Security_event_code  LIKE '%' || $1 || '%' ; `
 
)