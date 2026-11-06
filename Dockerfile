FROM rust:1.99.1@sha256:8a4e90c7946b74626488b1446f712c0fe0ed1a596cbd3d871684bc9c5433ae07 AS backend

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

FROM node:26.13.0@sha256:438603bbb5cda404b9416f0b5c8026f61501ab911760d861ec28c36b5d675725 AS frontend

COPY frontend/.npmrc ./
ENV PNPM_HOME="/pnpm"
ENV PATH="$PNPM_HOME:$PATH"
RUN npm install -g --force corepack
RUN corepack enable
WORKDIR /app

# Setup dependencies
COPY frontend/package.json frontend/pnpm-lock.yaml ./
RUN pnpm install --frozen-lockfile

# Build the frontend.
COPY frontend/*.json frontend/*.js frontend/*.cjs frontend/*.ts ./
COPY frontend/src ./src
COPY frontend/static ./static
RUN pnpm build

FROM gcr.io/distroless/cc-debian13@sha256:58e12be899275afaa05f1b03051ab19037ed7f252528e57a3a9ad7e220a62578 AS runtime

WORKDIR /app

COPY --from=backend /lib/*/libzstd.so.1 /lib/
COPY --from=backend /app/target/release/cipherly ./
COPY --from=frontend /app/build ./static

ENV PORT=8000
ENV RUST_BACKTRACE=1
ENV RUST_LIB_BACKTRACE=0
ENV RUST_LOG=cipherly=debug,tower_http=trace,axum=trace,info

CMD ["./cipherly"]
