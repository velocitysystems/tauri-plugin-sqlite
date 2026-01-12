-- Dummy migration for doc test compilation
-- This file exists solely to satisfy sqlx::migrate!() in documentation examples
CREATE TABLE IF NOT EXISTS _doc_test_dummy (
    id INTEGER PRIMARY KEY
);
