CREATE TABLE IF NOT EXISTS data.favicons
(
    id         integer PRIMARY KEY GENERATED ALWAYS AS IDENTITY,
    hash       VARCHAR(64) UNIQUE NOT NULL,
    image_data BYTEA              NOT NULL
);

ALTER TABLE data.servers
    ADD COLUMN IF NOT EXISTS favicon_id integer REFERENCES data.favicons (id) ON DELETE SET NULL;

