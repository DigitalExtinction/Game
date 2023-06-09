CREATE TABLE IF NOT EXISTS games (
    name CHARACTER({game_name_len}) NOT NULL PRIMARY KEY,
    max_players TINYINT NOT NULL,
    map_hash CHARACTER({map_hash_lenght}) NOT NULL,
    map_name CHARACTER({map_name_lenght}) NOT NULL
);

CREATE TABLE IF NOT EXISTS players (
    ordinal INTEGER PRIMARY KEY AUTOINCREMENT,
    author BOOLEAN NOT NULL,
    username CHARACTER({username_len}) NOT NULL UNIQUE,
    game CHARACTER({game_name_len}) NOT NULL,

    FOREIGN KEY(username) REFERENCES users(username)
        ON UPDATE CASCADE
        ON DELETE CASCADE,
    FOREIGN KEY(game) REFERENCES games(name)
        ON UPDATE CASCADE
        ON DELETE CASCADE
);
