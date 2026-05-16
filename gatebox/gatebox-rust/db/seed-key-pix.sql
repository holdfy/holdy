-- Seed mínimo de PIX keys para testes do simulador (PIX IN)
-- Cria a chave "test@simulator.com" apontando para account_id=2 (authentication_id/userId=3).

INSERT INTO key_pix (key, pix_key_type_id, document_number, description, account_id, partners_id)
SELECT
  'test@simulator.com',
  4, -- EMAIL
  '12345678900',
  'Seed simulator key (rust)',
  2,
  11
WHERE NOT EXISTS (
  SELECT 1 FROM key_pix WHERE key = 'test@simulator.com'
);

-- Caso a key já exista mas esteja com partners_id NULL (quebra decode no Rust),
-- atualiza para um parceiro válido.
UPDATE key_pix
SET partners_id = 11
WHERE key = 'test@simulator.com'
  AND partners_id IS NULL;

