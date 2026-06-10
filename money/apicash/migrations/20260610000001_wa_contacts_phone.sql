-- Adiciona coluna phone para usuários do site web registrarem número WhatsApp.
ALTER TABLE wa_contacts ADD COLUMN IF NOT EXISTS phone TEXT;
