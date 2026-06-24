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

## Coolify
If your Coolify server is weak, the better setup is to build the image in GitHub Actions and let Coolify deploy a ready image from GHCR.

Recommended approach:
- Let GitHub Actions build and push the production image to GHCR.
- Let Coolify deploy the registry image instead of building from the repository.
- Do not use `docker-compose.yml` for production if PostgreSQL already exists as a separate Coolify service.
- Keep `git` and `scc` in the application image itself. They belong in the `Dockerfile`, not in Coolify runtime settings.

GitHub Actions:
- Workflow file: `.github/workflows/docker-image.yml`
- Registry: `ghcr.io`
- Image name: `ghcr.io/<owner>/<repo>`
- Tags pushed automatically: branch tag, `sha-<commit>`, and `latest` on `main`

Coolify application settings:
- Source type: Docker image / Registry
- Registry: `ghcr.io`
- Image: `ghcr.io/<owner>/<repo>:latest`
- If the GHCR package is private, add GitHub registry credentials in Coolify with a token that can read packages.
- Port: `4000`
- Persistent storage: mount a volume to `/app/cloc_repo`

Required environment variables:
- `DATABASE_HOST`
- `DATABASE_USER`
- `DATABASE_NAME`
- `DATABASE_PASSWORD`

Optional environment variables:
- `RUST_LOG=cloc=info,tower_http=info`

Database initialization:
- The application does not run schema migrations automatically.
- Apply [schema/cloc.sql](/home/i/cloc.info/schema/cloc.sql) once to the PostgreSQL database you created in Coolify before starting the app.

Notes:
- The GitHub Actions workflow uses the repository `Dockerfile`, so `git` metadata is available during the Rust build and the final image already contains both `git` and `scc`.
- If image push fails with a permissions error, enable read/write workflow permissions for GitHub Actions in the repository settings.
