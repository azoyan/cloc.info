# cloc.info
Count lines of code - the service that get information of any git repository. 

Try it online: https://cloc.info

## Details
Backend written in Rust. The tool that count lines of code under hood is scc https://github.com/boyter/scc

## Local Docker Compose
The repository includes a local Docker Compose stack that brings up PostgreSQL and the Rust application with the frontend already baked into the app image.

Run it through the wrapper so containers use your current UID/GID for bind-mounted files:

```bash
./compose.sh up --build
```

The stack exposes the application on http://localhost:4000.
PostgreSQL is published on host port `55432` by default to avoid colliding with an already running local database. Override it with `POSTGRES_PORT` if you need a different host port.

What the services do:
- `db`: PostgreSQL 16 with the schema loaded from `schema/cloc.sql`.
- `app`: builds the Rust backend image, bakes the current frontend into `dist`, waits for the database, and serves the site and API on port 4000.

Useful commands:

```bash
./compose.sh ps
./compose.sh logs -f app db
./compose.sh down
```

Notes:
- `./compose.sh` creates `cloc_repo` as your current user before starting Compose, so bind mounts do not become root-owned.
- The PostgreSQL container still runs with its own internal `postgres` user because that is how the official image is designed; its data lives in a Docker named volume, so it does not create host-side permission issues.
