-- Chave PIX do vendedor — usada no off-ramp automático após confirmação de recebimento.
ALTER TABLE wa_contacts ADD COLUMN IF NOT EXISTS pix_key VARCHAR(140);
