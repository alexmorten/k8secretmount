FROM rust:1.41.1
WORKDIR /app
COPY . .
RUN cargo build --release

FROM gcr.io/distroless/static
WORKDIR /app
COPY --from=0 /app/target/release/k8secretmount .
ENTRYPOINT [ "/app/k8secretmount" ]
