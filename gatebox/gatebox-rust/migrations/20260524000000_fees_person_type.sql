-- Fase 7.6: PF/PJ fee differentiation
-- Adds person_type_id to fees so per-account rows can be tagged as PF(1) or PJ(2).
-- NULL means the row applies to any person type (existing behaviour preserved).
ALTER TABLE fees ADD COLUMN IF NOT EXISTS person_type_id BIGINT REFERENCES type_person_types(id);
