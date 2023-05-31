+++
title = "DE Logging"
weight = 5
sort_by = "weight"
+++

Game logs are both sent to standard output and persistently stored in text
files located at `{user_cache_dir}/DigitalExtinction/logs/`. After each game
startup, a new log file named according to the pattern
`%Y-%m-%d_%H-%M-%S.jsonl` is created. The log files consist of [JSON
lines](https://jsonlines.org/).

One might use a variation of the following command for pretty printing the logs
`cat 2023-05-31_17-44-25.jsonl | jq -cr '.timestamp + ": " + .fields.message'`.

`{user_cache_dir}` is obtained with
[dirs::cache_dir()](https://docs.rs/dirs/latest/dirs/fn.cache_dir.html) and
conforms to the following table:

|Platform | Value                               | Example                      |
| ------- | ----------------------------------- | ---------------------------- |
| Linux   | `$XDG_CACHE_HOME` or `$HOME`/.cache | /home/alice/.cache           |
| macOS   | `$HOME`/Library/Caches              | /Users/Alice/Library/Caches  |
| Windows | `{FOLDERID_LocalAppData}`           | C:\Users\Alice\AppData\Local |
