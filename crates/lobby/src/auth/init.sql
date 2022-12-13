CREATE TABLE IF NOT EXISTS users (
    username CHARACTER({username_len}) NOT NULL PRIMARY KEY,
    pass_hash CHARACTER({pass_hash_len}) NOT NULL,
    pass_salt CHARACTER({pass_salt_len}) NOT NULL
);
