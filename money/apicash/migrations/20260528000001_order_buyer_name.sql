-- Nome completo do comprador (obtido via NFS-e na criação do pedido via WhatsApp).
ALTER TABLE orders ADD COLUMN IF NOT EXISTS buyer_name TEXT;
