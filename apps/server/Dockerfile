# WingDrive Server Docker Image
# Builds wing-server with the apps/web bundle embedded via rust-embed.
#
# Prerequisite: `apps/web/dist/` must exist in the build context — run
# `bun install && bun run build` in `apps/web/` before `docker build`.
# CI workflows and the unified wingdrive+spacebot image handle this
# automatically; this Dockerfile assumes the dist directory is ready.

FROM ubuntu:24.04 AS builder
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

# Build server with media processing features.
# WING_SKIP_WEB_BUILD tells build.rs to use the pre-built apps/web/dist instead
# of trying to run `bun run build` (bun isn't installed in this stage).
RUN --mount=type=cache,target=/root/.cargo/registry \
	--mount=type=cache,target=/root/.cargo/git \
	--mount=type=cache,id=wingdrive-server-target-${TARGETARCH},target=/build/target \
	WING_SKIP_WEB_BUILD=1 cargo build --locked --release -p wing-server --features wing-core/heif,wing-core/ffmpeg && \
	cp target/release/wing-server /usr/local/bin/wing-server

#--
# Runtime image
#--
FROM ubuntu:24.04

# Install runtime dependencies only
RUN --mount=type=cache,target=/var/cache/apt,sharing=locked --mount=type=cache,target=/var/lib/apt,sharing=locked \
	apt-get update && apt-get install -y \
	libssl3t64 \
	ca-certificates \
	libavcodec60 \
	libavdevice60 \
	libavfilter9 \
	libavformat60 \
	libavutil58 \
	libswresample4 \
	libswscale7 \
	libheif1 \
	&& rm -rf /var/lib/apt/lists/*

# Create non-root user
RUN useradd -m -u 1000 wingdrive

# Copy binary
COPY --from=builder /usr/local/bin/wing-server /usr/bin/wing-server

# Environment
ENV DATA_DIR=/data \
	PORT=8080 \
	RUST_LOG=info,wing_core=debug

# Expose HTTP port
EXPOSE 8080

# Expose P2P port
EXPOSE 7373

# Volume for persistent data
VOLUME ["/data"]

# Run as non-root user
USER wingdrive:wingdrive

# Start server
ENTRYPOINT ["/usr/bin/wing-server"]
CMD ["--data-dir", "/data"]
