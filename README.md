# Cipherly

A web app for sharing secrets.

Development of this app is being presented as a series of videos on [The Friendly TL's YouTube channel](https://www.youtube.com/@FriendlyTL).

## Build

```sh
# Build prod container
DOCKER_DEFAULT_PLATFORM=linux/amd64 docker build -t gcr.io/cipherly/cipherly .

# Run prod container at http://127.0.0.1:8000
docker run -p 8000:8000 gcr.io/cipherly/cipherly
```

## Deploy

Staging is automatically deployed when a PR is merged to main. Prod is deployed by [cutting a new release tag](https://github.com/shadanan/cipherly/releases/new).

### Manual Deployment

```sh
# Upload to Google Container Registry
gcloud builds submit --tag gcr.io/cipherly/cipherly

# Staging Deployment
gcloud run deploy cipherly-staging \
  --image gcr.io/cipherly/cipherly \
  --platform managed \
  --region us-west1 \
  --allow-unauthenticated

# Prod Deployment
gcloud run deploy cipherly \
  --image gcr.io/cipherly/cipherly \
  --platform managed \
  --region us-west1 \
  --allow-unauthenticated
```

## Setting up the Development Environment

1. Install prerequisites.

   - [Node.js](https://nodejs.org/)
   - [Rust](https://www.rust-lang.org/)
     ```sh
     curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
     ```

1. Clone the repository.

   ```sh
   git clone git@github.com:shadanan/cipherly.git
   cd cipherly
   ```

1. Start a frontend dev server.

   ```sh
   cd frontend
   pnpm install
   pnpm build
   pnpm dev
   ```

1. Start a backend server using a test [KEK](https://cloud.google.com/kms/docs/envelope-encryption#key_encryption_keys)
   First, install [cargo-watch](https://crates.io/crates/cargo-watch) then run:

   ```sh
   KEKS='{"v1":"jRg36ErQ6FLcc7nZgngOpjJnJLGwA3xaMy0yx1pxJrI"}' cargo watch -x run
   ```

## Message Format

### URL Form

The basic URL form for a cipherly message is:

```
/schema/<schema>/version/<version>/<header>.<envelope>#<header>.<secret>
```

The schema is a name for the secret type. Currently only `password` is supported.

The version represents what version the secret type is on. Should be 1.

The header is a URL-safe base64 encoded value that is specific to the schema.

#### Password

For the password schema encrypted message:

```
/schema/password/version/1/<header>.<envelope>#<secret>
```
