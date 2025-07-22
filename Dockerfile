FROM rust AS backend

WORKDIR /app

COPY backend/Cargo.toml backend/Cargo.lock ./
COPY backend/src ./src
RUN cargo build --release
RUN mv target/release/cipherly .

FROM node AS frontend

WORKDIR /app

COPY frontend/*.json frontend/*.js frontend/*.cjs frontend/*.ts ./
RUN pnpm install

COPY frontend/src ./src
COPY frontend/static ./static
RUN pnpm build

FROM gcr.io/distroless/cc-debian12 AS runtime

WORKDIR /app

COPY --from=backend /app/cipherly ./
COPY --from=frontend /app/build ./static

ENV PORT=8000
ENV ROCKET_ADDRESS=0.0.0.0

CMD ["./cipherly"]
