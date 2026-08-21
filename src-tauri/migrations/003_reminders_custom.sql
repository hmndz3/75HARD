-- 003_reminders_custom.sql
-- Recordatorios propios y control de en qué días suena cada uno.
--
-- `days_mask` ya existía: un bit por día de la semana, bit 0 = lunes.
-- `interval_days` cubre lo que una máscara no puede expresar: "día de por
-- medio" y en general "cada N días". 0 significa "usa solo la máscara".

ALTER TABLE reminder ADD COLUMN title TEXT;
ALTER TABLE reminder ADD COLUMN message TEXT;
ALTER TABLE reminder ADD COLUMN interval_days INTEGER NOT NULL DEFAULT 0;
ALTER TABLE reminder ADD COLUMN custom INTEGER NOT NULL DEFAULT 0;

CREATE INDEX idx_reminder_enabled ON reminder(enabled);
