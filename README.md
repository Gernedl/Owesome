# Owesome

## Requirements

- Docker with the Compose plugin (`docker compose version`)

## Starting the app

1. Build and start everything:

   ```bash
   docker compose up -d --build
   ```

   The first build takes a while (it compiles the server and the WebAssembly bundle). The app waits until the database is ready before starting.

2. Open <http://localhost:8080>.

## Registering a user

Registration needs a code. With the app running, generate one with:

```bash
docker compose exec app /app/regcode
```

Enter it on the register page within 15 minutes; after that it expires.

## Secrets

You don't need to set any secrets. On the first start, the `secrets-init` container generates the Postgres password and the HMAC key into the `secrets` volume. Later starts reuse them.

The database only reads its password when it is first created, so the `secrets` and `db-data` volumes belong together. To reset, remove both:

```bash
docker compose down -v
```

Removing only one of them leaves the app unable to log in to the database.

## Configuration

Optional settings can go in a `.env` file next to `compose.yaml` (it is git-ignored):

| Variable        | Default   | Purpose                  |
|-----------------|-----------|--------------------------|
| `APP_PORT`      | `8080`    | Port the app is published on |
| `RUST_LOG`      | `info`    | Log level                |
| `POSTGRES_USER` | `owesome` | Database user            |
| `POSTGRES_DB`   | `owesome` | Database name            |

Changing `POSTGRES_USER` or `POSTGRES_DB` only takes effect on a fresh database (`docker compose down -v`).
