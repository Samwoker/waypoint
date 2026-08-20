-- ============================================================
-- ADD DEAD_LETTER AND DISCARDED VALUES TO DELIVERY_STATUS ENUM
-- ============================================================

ALTER TYPE delivery_status ADD VALUE IF NOT EXISTS 'dead_letter';
ALTER TYPE delivery_status ADD VALUE IF NOT EXISTS 'discarded';
