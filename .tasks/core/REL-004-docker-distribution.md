---
id: REL-004
title: One-file Docker Compose install for the server, checked in CI
status: To Do
assignee: bernardopg
priority: Medium
milestone: M5
sprint: S07
tags: [release, server, docker, ci, docs]
related_tasks: [REL-003, CORE-026]
last_updated: 2026-11-26
---

## Description

Evaluated on 2026-11-26 while moving the author's homelab back to the published image. The image
itself is in good shape: `server.yml` builds `apps/server/Dockerfile` natively for amd64 and arm64
on every `v*` tag, publishes one manifest to `ghcr.io/bernardopg/wingdrive/server` (`vX.Y.Z`,
`sha-…` and `latest` for stable tags) and the root `.dockerignore` keeps `target/`, `node_modules/`
and `.git/` out of the context. What a new user meets around it is broken:

- **`apps/server/docker-compose.yml` is not valid YAML.** Lines 6, 15 and 58 are indented with
  tabs, so `docker compose up` fails with `found character '\t' that cannot start any token`. The
  self-hosting docs send production users to this file.
- **It builds from source.** `build: context: ../..` needs the whole repository and a prebuilt
  `apps/web/dist` (the Dockerfile says so), while the docs show it with
  `image: ghcr.io/bernardopg/wingdrive/server:latest`. Someone who downloads only the compose file
  cannot use it.
- **The health check always fails.** It runs `curl -f http://localhost:8080/health`; the runtime
  image installs no `curl`, so the container reports unhealthy forever. The image has no
  `HEALTHCHECK` of its own.
- **Networking does not match how pairing works.** It publishes `8080` and `7373`, but nothing
  binds `7373`: Iroh picks its own UDP port, and pairing on the local network needs mDNS. The
  author's homelab runs `network_mode: host` for this reason.
- **Unsafe default:** `WING_AUTH=${WING_AUTH:-admin:changeme}` starts a reachable server with a
  known password when the variable is unset.
- **Library lock after a restart:** the server is PID 1 in every container run, so the lock the
  previous run left looked held and the library did not load. Fixed in CORE-026; the homelab
  entrypoint that deletes `*.winglibrary.lock` can go once that ships.
- `apps/server/.dockerignore` is never read: every build uses the repository root as context.
- The Dockerfile is only built on tags, so a change that breaks it is found during a release.

### What to add, and what not

- **One `docker-compose.yml` that pulls the published image** (`ghcr.io/…/server:latest`), with
  `network_mode: host`, a named volume for `/data`, a commented bind mount for files to index,
  `WING_AUTH` required (`${WING_AUTH:?set WING_AUTH}`), a health check the image can run and
  memory limits. This is the file new users download; it is worth having.
- **`docker-compose.build.yml`** as an override for contributors who build the image from the
  checkout (`docker compose -f docker-compose.yml -f docker-compose.build.yml up --build`).
- A separate `docker-compose.ghcr.yml` and `docker-compose.prod.yml` add nothing over these two:
  the published image is the production setup. Not worth it.
- **No `docker.yml` workflow.** `server.yml` already is the image workflow. Instead, add to it (or
  to `ci.yml`) a pull request job that runs `docker compose config` on both files and, when
  `apps/server/Dockerfile` or the server crate changes, `docker build` without pushing.
- **Healthcheck in the image:** either a `wing-server --health` subcommand or a tiny static
  probe, so the compose file and `docker run` users get the same check without `curl`.

## Implementation Steps

- [ ] Rewrite `apps/server/docker-compose.yml` for the published image (spaces only, host network,
      required `WING_AUTH`, working health check, `/data` volume, commented file mounts).
- [ ] Add `apps/server/docker-compose.build.yml` for local builds and move the web build into a
      Dockerfile stage so `docker build` works from a clean checkout.
- [ ] Add a `HEALTHCHECK` the runtime image can execute.
- [ ] Remove `apps/server/.dockerignore` or move what it adds into the root `.dockerignore`.
- [ ] CI: `docker compose config` for both files on every pull request; image build without push
      when the Dockerfile or `apps/server` changes.
- [ ] Docs: self-hosting (EN/PT site and `docs/overview/self-hosting.mdx`) show the one-file
      install, `WING_AUTH`, host networking for pairing and the update command.

## Acceptance Criteria

- [ ] `curl -O …/docker-compose.yml && WING_AUTH=me:secret docker compose up -d` starts a healthy
      server on a machine without the repository.
- [ ] A phone on the same network pairs with that server.
- [ ] Restarting the container keeps the library loaded (with CORE-026 shipped).
- [ ] A pull request that breaks either compose file or the Dockerfile fails CI.
