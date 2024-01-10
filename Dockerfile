FROM rust

COPY ./ ./

RUN cargo build --release

CMD ["./target/release/lima-city-ddns"]