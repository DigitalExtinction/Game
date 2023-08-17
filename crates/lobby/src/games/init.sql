CREATE TABLE IF NOT EXISTS games (
    name CHARACTER({game_name_len}) NOT NULL PRIMARY KEY,
    max_players SMALLINT NOT NULL,
    map_hash CHARACTER({map_hash_len}) NOT NULL,
    map_name CHARACTER({map_name_len}) NOT NULL,
    server CHARACTER({server_len}) NOT NULL
);

CREATE TABLE IF NOT EXISTS players (
    ordinal SMALLINT NOT NULL,
    author BOOLEAN NOT NULL,
    username CHARACTER({username_len}) NOT NULL,
    game CHARACTER({game_name_len}) NOT NULL,

    CONSTRAINT username UNIQUE (username),
    CONSTRAINT ordinal UNIQUE (game, ordinal),

    FOREIGN KEY(username) REFERENCES users(username)
        ON UPDATE CASCADE
        ON DELETE CASCADE,
    FOREIGN KEY(game) REFERENCES games(name)
        ON UPDATE CASCADE
        ON DELETE CASCADE
);


DO $$
BEGIN
    IF NOT EXISTS (SELECT 1 FROM pg_triggers WHERE tgname = 'check_ordinal') THEN
        CREATE TRIGGER check_ordinal
        BEFORE INSERT OR UPDATE ON players
        FOR EACH ROW
        EXECUTE FUNCTION check_max_players();
    END IF;
END $$;

CREATE OR REPLACE FUNCTION check_max_players()
RETURNS TRIGGER
LANGUAGE plpgsql
AS
$$
BEGIN
    IF (SELECT max_players FROM games WHERE name = NEW.game) IS NOT NULL AND NEW.ordinal > (SELECT max_players FROM games WHERE name = NEW.game) THEN
        RAISE EXCEPTION 'TOO-LARGE-ORDINAL';
    END IF;
    RETURN NEW;
END;
$$
