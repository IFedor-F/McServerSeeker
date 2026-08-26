CREATE SCHEMA IF NOT EXISTS data;
CREATE SCHEMA IF NOT EXISTS analytics;

CREATE TYPE data.difficulty AS ENUM (
    'peaceful',
    'easy',
    'normal',
    'hard'
    );

CREATE TYPE data.gamemode AS ENUM (
    'survival',
    'creative',
    'adventure',
    'spectator'
    );

CREATE TYPE data.connection_result AS ENUM (
    'login_disconnect',
    'configuration_disconnect',
    'play_disconnect',
    'successful'
    );

CREATE TABLE IF NOT EXISTS data.servers
(
    id                     INTEGER GENERATED ALWAYS AS IDENTITY,
    ip                     INET                   NOT NULL,
    port                   INTEGER                NOT NULL,
    domain                 TEXT,
    last_connection_result data.connection_result NOT NULL,
    description            TEXT                   NOT NULL,
    protocol               INTEGER                NOT NULL,
    version_name           TEXT                   NOT NULL,
    brand                  TEXT,
    first_seen             TIMESTAMPTZ            NOT NULL,
    last_seen              TIMESTAMPTZ            NOT NULL,
    last_checked           TIMESTAMPTZ            NOT NULL,
    online_players         INTEGER                NOT NULL,
    max_players            INTEGER                NOT NULL,
    enforces_secure_chat   BOOLEAN                NOT NULL,
    no_chat_reports        BOOLEAN                NOT NULL,
    is_whitelist           BOOLEAN,
    is_online_mode         BOOLEAN,
    offline_auth           BOOLEAN,
    code_of_conduct        TEXT,
    last_used_nick         VARCHAR(16),
    PRIMARY KEY (id),
    CONSTRAINT servers_ip_and_port UNIQUE (ip, port)
);


CREATE TABLE IF NOT EXISTS data.players
(
    id             INTEGER GENERATED ALWAYS AS IDENTITY,
    uuid           UUID        NOT NULL UNIQUE,
    name           VARCHAR(16) NOT NULL,
    is_online_mode BOOLEAN     NOT NULL,
    last_seen      TIMESTAMPTZ NOT NULL,
    first_seen     TIMESTAMPTZ NOT NULL,
    PRIMARY KEY (id)
);
CREATE INDEX IF NOT EXISTS idx_players_name ON data.players (name);


CREATE TABLE IF NOT EXISTS data.player_records
(
    server_id     INTEGER     NOT NULL,
    player_id     INTEGER     NOT NULL,
    first_seen    TIMESTAMPTZ NOT NULL,
    last_seen     TIMESTAMPTZ NOT NULL,
    last_gamemode data.gamemode,
    last_ping     INTEGER,
    PRIMARY KEY (server_id, player_id)
);

CREATE INDEX IF NOT EXISTS idx_player_records_player_id
    ON data.player_records (player_id);

--- Index for fast player tracking scan
CREATE INDEX IF NOT EXISTS idx_player_records_player_id_and_last_seen ON data.player_records (player_id, last_seen DESC)
    INCLUDE (server_id);

CREATE TABLE IF NOT EXISTS data.resource_packs
(
    server_id INTEGER,
    url       TEXT        NOT NULL,
    hash      VARCHAR(40) NOT NULL,
    forced    BOOLEAN     NOT NULL,
    PRIMARY KEY (server_id)
);


CREATE TABLE IF NOT EXISTS data.links
(
    id        INTEGER GENERATED ALWAYS AS IDENTITY,
    server_id INTEGER NOT NULL,
    url       TEXT    NOT NULL,
    PRIMARY KEY (id),
    CONSTRAINT servers_ip_and_url UNIQUE (server_id, url)
);


CREATE INDEX IF NOT EXISTS idx_links_server_id
    ON data.links (server_id);

CREATE TABLE IF NOT EXISTS data.world_data
(
    server_id           INTEGER NOT NULL,
    hashed_seed         BIGINT,
    gamemode            data.gamemode,
    difficulty          data.difficulty,
    dimension           TEXT,
    view_distance       SMALLINT,
    simulation_distance SMALLINT,
    is_hardcore         BOOLEAN,
    reduced_debug_info  BOOLEAN,
    do_limited_crafting BOOLEAN,
    is_flat             BOOLEAN,
    PRIMARY KEY (server_id)
);



CREATE TABLE IF NOT EXISTS data.commands
(
    id   INTEGER GENERATED ALWAYS AS IDENTITY,
    name VARCHAR(32767) NOT NULL UNIQUE,
    PRIMARY KEY (id)
);



CREATE TABLE IF NOT EXISTS data.server_commands
(
    server_id  INTEGER NOT NULL,
    command_id INTEGER NOT NULL,
    PRIMARY KEY (server_id, command_id)
);


CREATE INDEX IF NOT EXISTS idx_server_commands_command_id
    ON data.server_commands (command_id);


CREATE TABLE IF NOT EXISTS data.server_features
(
    server_id  INTEGER NOT NULL,
    feature_id INTEGER NOT NULL,
    PRIMARY KEY (server_id, feature_id)
);



CREATE TABLE IF NOT EXISTS data.features
(
    id         INTEGER GENERATED ALWAYS AS IDENTITY,
    identifier VARCHAR(32767) NOT NULL UNIQUE,
    PRIMARY KEY (id)
);



CREATE TABLE IF NOT EXISTS data.channels
(
    id         INTEGER GENERATED ALWAYS AS IDENTITY,
    identifier VARCHAR(32767) NOT NULL UNIQUE,
    PRIMARY KEY (id)
);



CREATE TABLE IF NOT EXISTS data.server_channels
(
    server_id  INTEGER NOT NULL,
    channel_id INTEGER NOT NULL,
    PRIMARY KEY (server_id, channel_id)
);



CREATE TABLE IF NOT EXISTS data.server_mods
(
    server_id INTEGER NOT NULL,
    mod_id    INTEGER NOT NULL,
    PRIMARY KEY (server_id, mod_id)
);


CREATE TABLE IF NOT EXISTS data.mods
(
    id   INTEGER GENERATED ALWAYS AS IDENTITY,
    name TEXT NOT NULL UNIQUE,
    PRIMARY KEY (id)
);

ALTER TABLE data.player_records
    ADD FOREIGN KEY (server_id) REFERENCES data.servers (id)
        ON UPDATE NO ACTION ON DELETE CASCADE;
ALTER TABLE data.resource_packs
    ADD FOREIGN KEY (server_id) REFERENCES data.servers (id)
        ON UPDATE NO ACTION ON DELETE CASCADE;
ALTER TABLE data.links
    ADD FOREIGN KEY (server_id) REFERENCES data.servers (id)
        ON UPDATE NO ACTION ON DELETE CASCADE;
ALTER TABLE data.world_data
    ADD FOREIGN KEY (server_id) REFERENCES data.servers (id)
        ON UPDATE NO ACTION ON DELETE CASCADE;
ALTER TABLE data.player_records
    ADD FOREIGN KEY (player_id) REFERENCES data.players (id)
        ON UPDATE NO ACTION ON DELETE CASCADE;
ALTER TABLE data.server_commands
    ADD FOREIGN KEY (server_id) REFERENCES data.servers (id)
        ON UPDATE NO ACTION ON DELETE CASCADE;
ALTER TABLE data.server_commands
    ADD FOREIGN KEY (command_id) REFERENCES data.commands (id)
        ON UPDATE NO ACTION ON DELETE CASCADE;
ALTER TABLE data.server_features
    ADD FOREIGN KEY (server_id) REFERENCES data.servers (id)
        ON UPDATE NO ACTION ON DELETE CASCADE;
ALTER TABLE data.server_features
    ADD FOREIGN KEY (feature_id) REFERENCES data.features (id)
        ON UPDATE NO ACTION ON DELETE CASCADE;
ALTER TABLE data.server_channels
    ADD FOREIGN KEY (server_id) REFERENCES data.servers (id)
        ON UPDATE NO ACTION ON DELETE CASCADE;
ALTER TABLE data.server_channels
    ADD FOREIGN KEY (channel_id) REFERENCES data.channels (id)
        ON UPDATE NO ACTION ON DELETE CASCADE;
ALTER TABLE data.server_mods
    ADD FOREIGN KEY (server_id) REFERENCES data.servers (id)
        ON UPDATE NO ACTION ON DELETE CASCADE;
ALTER TABLE data.server_mods
    ADD FOREIGN KEY (mod_id) REFERENCES data.mods (id)
        ON UPDATE NO ACTION ON DELETE CASCADE;

--- ANALYTICS

CREATE TABLE IF NOT EXISTS analytics.player_tracks
(
    id             INTEGER GENERATED ALWAYS AS IDENTITY,
    name           VARCHAR(16),
    uuid           UUID,
    webhook_id     INTEGER NOT NULL,
    last_send      TIMESTAMPTZ,
    last_server_id INTEGER,
    PRIMARY KEY (id),
    CONSTRAINT webhook_with_uuid_and_name
        UNIQUE NULLS NOT DISTINCT (name, uuid, webhook_id)
);

CREATE INDEX IF NOT EXISTS idx_player_tracks_uuid ON analytics.player_tracks (uuid);
CREATE INDEX idx_player_tracks_name ON analytics.player_tracks (name);


CREATE TABLE IF NOT EXISTS analytics.webhooks
(
    id   INTEGER NOT NULL GENERATED ALWAYS AS IDENTITY,
    name TEXT    NOT NULL UNIQUE,
    url  TEXT    NOT NULL,
    PRIMARY KEY (id)
);

ALTER TABLE analytics.player_tracks
    ADD CONSTRAINT require_name_or_uuid
        CHECK (name IS NOT NULL OR uuid IS NOT NULL);
ALTER TABLE analytics.player_tracks
    ADD FOREIGN KEY (webhook_id) REFERENCES analytics.webhooks (id)
        ON UPDATE NO ACTION ON DELETE CASCADE;
ALTER TABLE analytics.player_tracks
    ADD FOREIGN KEY (last_server_id) REFERENCES data.servers (id)
        ON UPDATE NO ACTION ON DELETE SET NULL;
