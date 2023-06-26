FROM alpine:3.18.2 AS build
ENV RUSTUP_HOME="/usr/local/rustup" CARGO_HOME="/usr/local/cargo" PATH="/usr/local/cargo/bin:$PATH" RUSTFLAGS="-C target-feature=+crt-static"
RUN apk add git curl cmake make g++ pango-dev fontconfig-dev libxinerama-dev libxfixes-dev libxcursor-dev

RUN ln -s /usr/bin/x86_64-alpine-linux-musl-gcc /usr/bin/musl-gcc
RUN ln -s /usr/bin/x86_64-alpine-linux-musl-g++ /usr/bin/musl-g++

RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --profile minimal --default-toolchain stable-x86_64-unknown-linux-musl

WORKDIR /rust

COPY src/ ./src/
COPY Cargo.toml Cargo.lock ./
RUN cargo build --release --target x86_64-unknown-linux-musl

FROM scratch AS runtime
COPY --from=build /rust/target/x86_64-unknown-linux-musl/release/haspa-camt052-to-csv /usr/local/bin/
ENTRYPOINT ["/usr/local/bin/haspa-camt052-to-csv"]
