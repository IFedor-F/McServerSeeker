ALTER TABLE data.resource_packs
    ALTER COLUMN hash DROP NOT NULL; -- column hash can now be NULL