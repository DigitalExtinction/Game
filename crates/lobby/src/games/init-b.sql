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
