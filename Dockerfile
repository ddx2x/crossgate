FROM rustlang/rust:nightly as cargo-build

RUN rustup target add x86_64-unknown-linux-musl && \
    rustup component add rust-src --toolchain nightly-x86_64-unknown-linux-gnu
RUN apt update && apt install -y musl-tools musl-dev
RUN update-ca-certificates

WORKDIR /app

ADD . .

# add vendor libraries
RUN cargo vendor

RUN cargo build -Zbuild-std --release --target=x86_64-unknown-linux-musl --bin api

FROM scratch

WORKDIR /app

COPY --from=cargo-build /app/target/x86_64-unknown-linux-musl/release/api .

COPY --from=cargo-build /app/.env .

ENTRYPOINT ["/app/api"]