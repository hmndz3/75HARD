-- 001_init.sql — esquema completo (§3 del spec)
-- Los PRAGMA de conexión (WAL, synchronous, foreign_keys) se aplican en db/mod.rs.

CREATE TABLE challenge (
  id           TEXT PRIMARY KEY,
  name         TEXT NOT NULL,
  start_date   TEXT NOT NULL,
  target_days  INTEGER NOT NULL DEFAULT 75,
  rules_json   TEXT NOT NULL DEFAULT '{}',
  ended_at     TEXT,
  ended_reason TEXT CHECK (ended_reason IN ('completed','failed','abandoned')),
  created_at   TEXT NOT NULL,
  updated_at   TEXT NOT NULL
);

CREATE TABLE day (
  date               TEXT PRIMARY KEY,
  challenge_id       TEXT REFERENCES challenge(id) ON DELETE SET NULL,
  day_number         INTEGER,
  status             TEXT NOT NULL DEFAULT 'pending'
                       CHECK (status IN ('pending','complete','failed','skipped')),
  morning_checkin_at TEXT,
  evening_checkin_at TEXT,
  notes              TEXT,
  updated_at         TEXT NOT NULL
);
CREATE INDEX idx_day_challenge ON day(challenge_id, day_number);
CREATE INDEX idx_day_status    ON day(status);

CREATE TABLE sleep_log (
  date                 TEXT PRIMARY KEY REFERENCES day(date) ON DELETE CASCADE,
  bedtime              TEXT NOT NULL,
  wake_time            TEXT NOT NULL,
  minutes              INTEGER NOT NULL,
  quality              INTEGER CHECK (quality BETWEEN 1 AND 5),
  woke_up_during_night INTEGER,
  updated_at           TEXT NOT NULL
);

CREATE TABLE meal (
  id          TEXT PRIMARY KEY,
  date        TEXT NOT NULL REFERENCES day(date) ON DELETE CASCADE,
  eaten_at    TEXT NOT NULL,
  kind        TEXT NOT NULL CHECK (kind IN ('meal','snack')),
  description TEXT NOT NULL,
  calories    INTEGER,
  photo_path  TEXT,
  updated_at  TEXT NOT NULL
);
CREATE INDEX idx_meal_date ON meal(date, eaten_at);

CREATE TABLE workout (
  id              TEXT PRIMARY KEY,
  date            TEXT NOT NULL REFERENCES day(date) ON DELETE CASCADE,
  started_at      TEXT NOT NULL,
  kind            TEXT NOT NULL CHECK (kind IN ('gym','cardio','outdoor','sport','other')),
  description     TEXT,
  duration_min    INTEGER NOT NULL,
  is_outdoor      INTEGER NOT NULL DEFAULT 0,
  calories_burned INTEGER,
  updated_at      TEXT NOT NULL
);
CREATE INDEX idx_workout_date ON workout(date, started_at);

CREATE TABLE glucose_reading (
  id             TEXT PRIMARY KEY,
  date           TEXT NOT NULL REFERENCES day(date) ON DELETE CASCADE,
  measured_at    TEXT NOT NULL,
  value_mgdl     INTEGER NOT NULL,
  context        TEXT NOT NULL CHECK (context IN
                   ('fasting','pre_meal','post_meal_2h','random','pre_workout')),
  notes          TEXT,
  linked_meal_id TEXT REFERENCES meal(id) ON DELETE SET NULL,
  updated_at     TEXT NOT NULL
);
CREATE INDEX idx_glucose_date ON glucose_reading(date, measured_at);

CREATE TABLE work_session (
  id          TEXT PRIMARY KEY,
  date        TEXT NOT NULL REFERENCES day(date) ON DELETE CASCADE,
  started_at  TEXT NOT NULL,
  ended_at    TEXT,
  minutes     INTEGER NOT NULL,
  category    TEXT NOT NULL,
  description TEXT,
  updated_at  TEXT NOT NULL
);
CREATE INDEX idx_work_date ON work_session(date, started_at);

CREATE TABLE water_log (
  date       TEXT PRIMARY KEY REFERENCES day(date) ON DELETE CASCADE,
  ml         INTEGER NOT NULL DEFAULT 0,
  updated_at TEXT NOT NULL
);

CREATE TABLE reading_log (
  id         TEXT PRIMARY KEY,
  date       TEXT NOT NULL REFERENCES day(date) ON DELETE CASCADE,
  pages      INTEGER NOT NULL,
  book       TEXT,
  updated_at TEXT NOT NULL
);
CREATE INDEX idx_reading_date ON reading_log(date);

CREATE TABLE weight_log (
  date         TEXT PRIMARY KEY REFERENCES day(date) ON DELETE CASCADE,
  kg           REAL NOT NULL,
  body_fat_pct REAL,
  updated_at   TEXT NOT NULL
);

CREATE TABLE mood_log (
  date       TEXT PRIMARY KEY REFERENCES day(date) ON DELETE CASCADE,
  mood       INTEGER NOT NULL CHECK (mood   BETWEEN 1 AND 5),
  energy     INTEGER NOT NULL CHECK (energy BETWEEN 1 AND 5),
  stress     INTEGER CHECK (stress BETWEEN 1 AND 5),
  notes      TEXT,
  updated_at TEXT NOT NULL
);

CREATE TABLE progress_photo (
  id         TEXT PRIMARY KEY,
  date       TEXT NOT NULL REFERENCES day(date) ON DELETE CASCADE,
  path       TEXT NOT NULL,
  updated_at TEXT NOT NULL
);

CREATE TABLE reminder (
  id            TEXT PRIMARY KEY,
  kind          TEXT NOT NULL,
  time_of_day   TEXT NOT NULL,
  days_mask     INTEGER NOT NULL DEFAULT 127,
  enabled       INTEGER NOT NULL DEFAULT 1,
  last_fired_at TEXT
);

CREATE TABLE settings (key TEXT PRIMARY KEY, value TEXT NOT NULL);

INSERT INTO settings (key, value) VALUES
  ('coach_tone',          'directo'),
  ('sleep_goal_min',      '420'),
  ('water_goal_ml',       '3000'),
  ('workout_goal_min',    '45'),
  ('reading_goal_pages',  '10'),
  ('day_cutoff_hour',     '4'),
  ('autostart',           '0');
