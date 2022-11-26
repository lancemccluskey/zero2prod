FROM rust:1-slim AS builder

WORKDIR /app

# Build with just deps for caching
RUN apt update && apt install musl-tools lld clang -y
RUN rustup target add x86_64-unknown-linux-musl
COPY Cargo.toml Cargo.lock /app/
RUN mkdir src
RUN touch src/main.rs
RUN echo "fn main() {}" >> src/main.rs
RUN cargo build --target x86_64-unknown-linux-musl --release

# Copy rest of source code over and build
COPY . .
ENV SQLX_OFFLINE true
RUN cargo build --target x86_64-unknown-linux-musl --release --bin zero2prod

FROM alpine:latest AS runtime

RUN apk update && apk add openssl ca-certificates && apk fix
COPY --from=builder /app/target/x86_64-unknown-linux-musl/release/zero2prod zero2prod
COPY configuration configuration
ENV APP_ENVIRONMENT production
ENTRYPOINT [ "./zero2prod" ]
