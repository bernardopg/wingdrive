# WingDrive Server Docker Image
# Builds wing-server with the apps/web bundle embedded via rust-embed.
#
# Build from the repository root on a clean checkout; the `web` stage builds
# the bundle, so no `apps/web/dist` is needed beforehand:
#
#   docker build -f apps/server/Dockerfile -t wingdrive-server .
#
# Most users want the published image instead: see apps/server/docker-compose.yml.

#--
# Web UI bundle
#--
FROM oven/bun:1.4.4 AS web
WORKDIR /src
# The workspace needs every package.json it lists, so the stage takes the
# whole context; the root .dockerignore keeps node_modules and target out.
# Only the web workspace's dependencies are installed, without install
# scripts: other workspaces pull native modules (node-gyp) the bundle never uses.
COPY . .
RUN --mount=type=cache,target=/root/.bun/install/cache \
	bun install --frozen-lockfile --filter @wingdrive/web --ignore-scripts && \
	cd apps/web && bun run build

#--
# Rust build
#--
FROM ubuntu:26.04 AS builder
ARG TARGETARCH

# Install build dependencies
RUN --mount=type=cache,target=/var/cache/apt,sharing=locked --mount=type=cache,target=/var/lib/apt,sharing=locked \
	apt-get update && apt-get install -y \
	build-essential \
	curl \
	git \
	pkg-config \
	libssl-dev \
	ca-certificates \
	cmake \
	nasm \
	zlib1g-dev \
	libclang-dev \
	libavcodec-dev \
	libavdevice-dev \
	libavfilter-dev \
	libavformat-dev \
	libavutil-dev \
	libswresample-dev \
	libswscale-dev \
	libheif-dev

# Install Rust
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --profile minimal
ENV PATH="/root/.cargo/bin:$PATH"

WORKDIR /build

# Copy workspace configuration
COPY Cargo.toml Cargo.lock ./
COPY .cargo ./.cargo

# Copy source code
COPY core ./core
COPY crates ./crates
COPY apps ./apps
COPY xtask ./xtask
COPY patches ./patches
COPY --from=web /src/apps/web/dist ./apps/web/dist

# Build server with media processing features.
# WING_SKIP_WEB_BUILD tells build.rs to use the apps/web/dist built by the web
# stage instead of trying to run `bun run build` (bun isn't installed here).
RUN --mount=type=cache,target=/root/.cargo/registry \
	--mount=type=cache,target=/root/.cargo/git \
	--mount=type=cache,id=wingdrive-server-target-${TARGETARCH},target=/build/target \
	WING_SKIP_WEB_BUILD=1 cargo build --locked --release -p wing-server --features wing-core/heif,wing-core/ffmpeg && \
	cp target/release/wing-server /usr/local/bin/wing-server

#--
# Runtime image
#--
FROM ubuntu:26.04

# Install runtime dependencies only
RUN --mount=type=cache,target=/var/cache/apt,sharing=locked --mount=type=cache,target=/var/lib/apt,sharing=locked \
	apt-get update && apt-get install -y \
	libssl3t64 \
	ca-certificates \
	libavcodec62 \
	libavdevice62 \
	libavfilter11 \
	libavformat62 \
	libavutil60 \
	libswresample6 \
	libswscale9 \
	libheif1 \
	python3 \
	bubblewrap \
	&& rm -rf /var/lib/apt/lists/*

# Create non-root user
RUN usermod -l wingdrive -d /home/wingdrive -m ubuntu && \
	groupmod -n wingdrive ubuntu && \
	install -d -o wingdrive -g wingdrive /data

# Copy binary
COPY --from=builder /usr/local/bin/wing-server /usr/bin/wing-server

# Environment
ENV DATA_DIR=/data \
	PORT=8080 \
	RUST_LOG=info,wing_core=debug

# HTTP port. Peer-to-peer traffic uses a UDP port Iroh picks at start and
# pairing on the local network uses mDNS, so run the container with host
# networking (`network_mode: host`) rather than publishing a P2P port.
EXPOSE 8080

# Volume for persistent data
VOLUME ["/data"]

# Run as non-root user
USER wingdrive:wingdrive

# Probes /health on the configured WING_HOST and PORT; no curl in the image.
HEALTHCHECK --interval=30s --timeout=10s --start-period=60s --retries=3 \
	CMD ["/usr/bin/wing-server", "--health-check"]

# Start server
ENTRYPOINT ["/usr/bin/wing-server"]
CMD ["--data-dir", "/data"]
