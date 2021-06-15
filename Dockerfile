FROM rust:latest

WORKDIR /usr/src/shas
COPY . .

WORKDIR /usr/src/shas/browser
RUN curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh
RUN wasm-pack build --target web --release

WORKDIR /usr/src/shas/server
RUN cargo build --release

CMD /usr/src/shas/server/target/release/shas
