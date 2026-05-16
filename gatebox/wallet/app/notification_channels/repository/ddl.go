package notification_channelsRepo

var(
   SQL_NOTIFICATION_CHANNELS_LIST = `
       SELECT 
              id, 
              channel_code, 
              name, 
              description, 
              requires_subject, 
              max_body_length, 
              delivery_time_seconds, 
              is_active, 
              created_at 
         FROM notification_channels
         ORDER BY id LIMIT $1 OFFSET $2
       ;`
 
   SQL_NOTIFICATION_CHANNELS_INSERT = `
       INSERT INTO notification_channels(
              channel_code, 
              name, 
              description, 
              requires_subject, 
              max_body_length, 
              delivery_time_seconds, 
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

   SQL_NOTIFICATION_CHANNELS_UPDATE = `
       UPDATE notification_channels SET 
              channel_code = $1, 
              name = $2, 
              description = $3, 
              requires_subject = $4, 
              max_body_length = $5, 
              delivery_time_seconds = $6, 
              is_active = $7, 
              created_at = $8 
        WHERE id = $9; `

   SQL_NOTIFICATION_CHANNELS_DELETE_BY_ID = `
       DELETE FROM notification_channels
        WHERE id = $1; `

   SQL_GET_NOTIFICATION_CHANNELS_BY_ID = `
       SELECT 
              id, 
              channel_code, 
              name, 
              description, 
              requires_subject, 
              max_body_length, 
              delivery_time_seconds, 
              is_active, 
              created_at 
         FROM notification_channels
        WHERE id = $1; `
 
   SQL_GET_NOTIFICATION_CHANNELS_BY_CHANNEL_CODE = `
       SELECT 
              id, 
              channel_code, 
              name, 
              description, 
              requires_subject, 
              max_body_length, 
              delivery_time_seconds, 
              is_active, 
              created_at 
         FROM notification_channels
        WHERE Channel_code  LIKE '%' || $1 || '%' ; `
 
)