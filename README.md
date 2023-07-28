# Wrapscan Registry

## Gateway

Wrapscan Registry is a package registry for `wraps` (composable wasm modules).
The primary function of the registry is to facilitate the publishing of URIs, acting as the go-to source from where these wraps can be downloaded.

### Routes: 
- `GET /` - Home: get version of the registry
- `GET /r/{user}/{package_and_version}` - Get the latest version of the wrap
  - Returns: 
    - Body `{ name: "0.1.0", uri: "wrap://...", ... }`
- `GET /v/{user}/{package}` - Get package info
  - Returns: 
    - Body `{ id: "user_name/package_name", name: "package_name", ... }`
- `GET /r/{user}/{package_and_version}/wrap.info` - Get the published URI for the wrap
  - Returns: 
    - Header `x-wrap-uri: wrap://...`
    - Status: 200
- `POST /r/{user}/{package_and_version}` - Publish a URI for the wrap
  - Header: `Authorization: Bearer {base64 encoded API key}`
  - Body: `{ uri: "wrap://..." }`

### How to run
- nvm use
- `yarn db` to start the db
- `yarn dev` to start the server
- `yarn test` to run the tests

You can also use the cargo commands directly in the gateway_service crate:
- `cargo run -F local` to start the server
- `cargo test` to run the tests

All of the yarn commands run both for the feature flag `local` and without it.

We use `#[cfg(feature = "local")]` to switch between local and production code.
To run locally the feature flag `local` must be set. This will use a local database and server.
To get your editor working with the local flag, temporarily edit Cargo.toml and add the following:
```
[features]
default = ["local"]
```

### Getting started with the codebase

- `rust/gateway_service` contains the gateway service crate

#### Main files and directories
- `src/main.rs` is the entrypoint for the server
- `src/setup_routes` contains the server initialization and route registration
- `src/routes` contains the route handlers
- `src/functions` contains the raw functions the service supports (1:1 mapping to routes)
- `src/constants` contains constants used throughout the service
- `src/models` contains the models used throughout the service

#### Database
- `src/db` contains the database code
- `src/db/local_db.rs` contains setup for the local database, it only runs when the `local` feature flag is set
- `src/db/dynamodb.rs` contains DynamoDbClient and PackageRepository implementations for the production database

### Debugging
- `src/debugging.rs` contains debugging utilities
- To help with debugging locally, two macros are available (note they only work when the `local` feature flag is set):
  - `debug!` - is a wrapper around `dbg!`.
    - It prints the file and line number of the debug statement as well as the value of the expression passed.
  - `debug_println!` - prints to stdout