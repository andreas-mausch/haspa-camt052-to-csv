# https://levelup.gitconnected.com/create-an-optimized-rust-alpine-docker-image-1940db638a6c
# https://fltk-rs.github.io/fltk-book/Cross-Compiling.html#using-docker
# https://stackoverflow.com/questions/72424759/rust-in-docker-image-exec-no-such-file-or-directory

FROM alpine:3.18.2 AS build
ENV RUSTUP_HOME="/usr/local/rustup" CARGO_HOME="/usr/local/cargo" PATH="/usr/local/cargo/bin:$PATH" RUSTFLAGS="-C target-feature=-crt-static"
RUN apk add git curl cmake make g++ pango-dev fontconfig-dev libxinerama-dev libxfixes-dev libxcursor-dev

RUN ln -s /usr/bin/x86_64-alpine-linux-musl-gcc /usr/bin/musl-gcc
RUN ln -s /usr/bin/x86_64-alpine-linux-musl-g++ /usr/bin/musl-g++

RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --profile minimal --default-toolchain stable-x86_64-unknown-linux-musl

COPY . .
RUN cargo build --release

FROM alpine:3.18.2 AS runtime
COPY --from=build target/release/hello-world /usr/local/bin/hello-world
CMD ["/usr/local/bin/hello-world"]

# This still fails with missing dependencies:
# Error loading shared library libX11.so.6: No such file or directory (needed by /usr/local/bin/hello-world)
# Error loading shared library libXinerama.so.1: No such file or directory (needed by /usr/local/bin/hello-world)
# Error loading shared library libXcursor.so.1: No such file or directory (needed by /usr/local/bin/hello-world)
# Error loading shared library libXfixes.so.3: No such file or directory (needed by /usr/local/bin/hello-world)
# Error loading shared library libpango-1.0.so.0: No such file or directory (needed by /usr/local/bin/hello-world)
# Error loading shared library libgobject-2.0.so.0: No such file or directory (needed by /usr/local/bin/hello-world)
# Error loading shared library libcairo.so.2: No such file or directory (needed by /usr/local/bin/hello-world)
# Error loading shared library libpangocairo-1.0.so.0: No such file or directory (needed by /usr/local/bin/hello-world)
# Error loading shared library libgcc_s.so.1: No such file or directory (needed by /usr/local/bin/hello-world)
# Error relocating /usr/local/bin/hello-world: cairo_matrix_init_identity: symbol not found
# Error relocating /usr/local/bin/hello-world: pango_layout_get_extents: symbol not found
# ...
