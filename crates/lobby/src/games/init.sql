CREATE TABLE IF NOT EXISTS games (
    name CHARACTER({game_name_len}) NOT NULL PRIMARY KEY,
    max_players TINYINT NOT NULL,
    map_hash CHARACTER({map_hash_len}) NOT NULL,
    map_name CHARACTER({map_name_len}) NOT NULL,
    server CHARACTER({server_len}) NOT NULL
);

CREATE TABLE IF NOT EXISTS players (
    ordinal TINYINT NOT NULL,
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


CREATE TRIGGER IF NOT EXISTS check_ordinal
BEFORE INSERT ON players
FOR EACH ROW
BEGIN
    SELECT CASE
        WHEN (SELECT max_players FROM games WHERE name = NEW.game) IS NOT NULL AND NEW.ordinal > (SELECT max_players FROM games WHERE name = NEW.game)
        THEN RAISE(FAIL, 'TOO-LARGE-ORDINAL')
    END;
END;
