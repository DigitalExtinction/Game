CREATE TABLE IF NOT EXISTS games (
    name CHARACTER({game_name_len}) NOT NULL PRIMARY KEY,
    max_players SMALLINT NOT NULL,
    map_hash CHARACTER({map_hash_len}) NOT NULL,
    map_name CHARACTER({map_name_len}) NOT NULL,
    server CHARACTER({server_len}) NOT NULL
);
