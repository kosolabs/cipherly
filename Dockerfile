FROM rust:1.99.1@sha256:09027f873d45b8ecd1dc86f1fa13b7a62b875c961f017a0e331267b350430137 AS backend

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

FROM node:26.13.0@sha256:6d2c7d5d074bf72f81ddc252b4fbb5d625bc028eeda49cb4b8c2ff8eda1ee617 AS frontend

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

FROM gcr.io/distroless/cc-debian13@sha256:d74d144baa124dd9defd01d71872bb56c49d4441dba6153ccf790364f97e64b1 AS runtime

WORKDIR /app

COPY --from=backend /lib/*/libzstd.so.1 /lib/
COPY --from=backend /app/target/release/cipherly ./
COPY --from=frontend /app/build ./static

ENV PORT=8000
ENV RUST_BACKTRACE=1
ENV RUST_LIB_BACKTRACE=0
ENV RUST_LOG=cipherly=debug,tower_http=trace,axum=trace,info

CMD ["./cipherly"]
