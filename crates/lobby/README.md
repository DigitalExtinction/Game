# Digital Extinction Lobby Server

DE Lobby Server implements multiplayer game management functionality. It
exposes a simple HTTP based API and a [hole
punching](https://en.wikipedia.org/wiki/Hole_punching_(networking)) rendezvous.

Full documentation is available here
[docs.de-game.org/lobby/](https://docs.de-game.org/lobby/).


### Postgres

Pull Postgres docker image.

```bash
docker pull postgres
```

`DE_POSTGRES_PASSWORD` and `DE_POSTGRES_MOUNT`

```bash
docker run --name de-lobby-postgres \
  -e POSTGRES_PASSWORD=$DE_POSTGRES_PASSWORD \
  -e PGDATA=/var/lib/postgresql/data/pgdata \
  -v $DE_POSTGRES_MOUNT:/var/lib/postgresql/data \
  -p 127.0.0.1:5432:5432 \
  -d postgres
```

`postgres://postgres:$DE_POSTGRES_PASSWORD@localhost/postgres`
