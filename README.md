# Owesome

## Requirements

- Docker with the Compose plugin (`docker compose version`)

## Starting the app

1. Create your local config from the template:

   ```bash
   cp .env.example .env
   ```

2. Set `POSTGRES_PASSWORD` in `.env`. Use URL-safe characters only (letters, digits, `- _ . ~`), since the password is embedded in the app's database URL. To generate one:

   ```bash
   openssl rand -hex 32
   ```

3. Build and start everything:

   ```bash
   docker compose up -d --build
   ```

   The first build takes a while (it compiles the server and the WebAssembly bundle). The app waits until the database is ready before starting.

4. Open <http://localhost:8080>. To use another port, set `APP_PORT` in `.env`.
