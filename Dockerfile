FROM rust:1.90.0@sha256:f98e87a9cc69f782ddbf3e7020b7438c8d7fa113bb54556757042134c7d1dfe7 AS backend

# Setup dependencies and run a dummy build ahead
# of copying in our code. This speeds up re-builds
# triggered by changes to src/ by keeping dependencies
# in a separate layer.
WORKDIR /app
COPY backend/Cargo.toml backend/Cargo.lock backend/rust-toolchain.toml ./
COPY backend/build/dummy.rs build/dummy.rs
RUN cargo build --release --lib

# Build the backend.
COPY backend/src ./src
RUN cargo build --release

FROM node:24.9.0@sha256:89ec1788bd39c23adafd9f7ee14a023fcd01d08c5920745d71fb571bc7d30d1e AS frontend

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

FROM gcr.io/distroless/cc-debian12@sha256:620d8b11ae800f0dbd7995f89ddc5344ad603269ea98770588b1b07a4a0a6872 AS runtime

WORKDIR /app

COPY --from=backend /app/target/release/cipherly ./
COPY --from=frontend /app/build ./static

ENV PORT=8000
ENV RUST_BACKTRACE=1
ENV RUST_LIB_BACKTRACE=0
ENV RUST_LOG=cipherly=debug,tower_http=trace,axum=trace,info

CMD ["./cipherly"]
