FROM rust

COPY ./ ./

RUN cargo build --release

EXPOSE 3000

CMD ["./target/release/lima-city-ddns"]