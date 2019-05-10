FROM rust:1.34.1

WORKDIR /app
RUN apt-get update && apt-get -y upgrade && apt-get -y install libseccomp-dev nmap
RUN rustup update nightly
RUN rustup target add wasm32-unknown-wasi --toolchain nightly
ADD src /app/src/
ADD Cargo.toml test.sh /app/
RUN cargo +nightly build --release --target wasm32-unknown-wasi
