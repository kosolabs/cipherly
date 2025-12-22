FROM rust:1.92.0@sha256:48851a839d6a67370c9dbe0e709bedc138e3e404b161c5233aedcf2b717366e4 AS backend

# Setup dependencies and run a dummy build ahead
# of copying in our code. This speeds up re-builds
# triggered by changes to src/ by keeping dependencies
# in a separate layer.
WORKDIR /app

# The debian 13 image doesn't seem to include libzstd1. Install it.
# See https://github.com/GoogleContainerTools/distroless/issues/1887
# Error: ./sqlx: error while loading shared libraries: libzstd.so.1: cannot open shared object file: No such file or directory
RUN apt-get update && apt-get install -y --no-install-recommends libzstd1

COPY backend/Cargo.toml backend/Cargo.lock backend/rust-toolchain.toml ./
COPY backend/build/dummy.rs build/dummy.rs
RUN cargo build --release --lib

# Build the backend.
COPY backend/src ./src
RUN cargo build --release

FROM node:24.12.0@sha256:20988bcdc6dc76690023eb2505dd273bdeefddcd0bde4bfd1efe4ebf8707f747 AS frontend

ENV PNPM_HOME="/pnpm"
ENV PATH="$PNPM_HOME:$PATH"
RUN corepack enable
WORKDIR /app

# Setup dependencies
COPY frontend/package.json frontend/pnpm-lock.yaml frontend/.npmrc ./
RUN pnpm install --frozen-lockfile

# Build the frontend.
COPY frontend/*.json frontend/*.js frontend/*.cjs frontend/*.ts ./
COPY frontend/src ./src
COPY frontend/static ./static
RUN pnpm build

FROM gcr.io/distroless/cc-debian13@sha256:43fc7a7004c4cdb27aac60b3e95c87130cf47823f72d25d42ed0f9b503f1d184 AS runtime

WORKDIR /app

COPY --from=backend /lib/*/libzstd.so.1 /lib/
COPY --from=backend /app/target/release/cipherly ./
COPY --from=frontend /app/build ./static

ENV PORT=8000
ENV RUST_BACKTRACE=1
ENV RUST_LIB_BACKTRACE=0
ENV RUST_LOG=cipherly=debug,tower_http=trace,axum=trace,info

CMD ["./cipherly"]
