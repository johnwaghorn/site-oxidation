ALTER TABLE users
ADD COLUMN theme_preference TEXT NOT NULL DEFAULT 'system'
CHECK(theme_preference IN ('system', 'light', 'dark'));
