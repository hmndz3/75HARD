-- 002_reminders.sql — recordatorios por defecto (Fase 1).
-- El id es estable y legible: el planificador y la UI se refieren a él.

INSERT INTO reminder (id, kind, time_of_day, days_mask, enabled) VALUES
  ('morning', 'morning', '07:00', 127, 1),
  ('meal',    'meal',    '12:30', 127, 1),
  ('workout', 'workout', '17:00', 127, 1),
  ('water',   'water',   '08:00', 127, 1),
  ('evening', 'evening', '21:30', 127, 1);

INSERT INTO settings (key, value) VALUES
  ('hotkey_quick',      'Ctrl+Alt+H'),
  ('notifications',     '1'),
  ('water_every_hours', '2'),
  ('quiet_start',       '22'),
  ('quiet_end',         '7');
