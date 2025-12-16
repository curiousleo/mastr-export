FROM rust:1 as build-env
WORKDIR /app
COPY . /app
RUN cargo build --release
CMD ["./target/release/mastr-export"]

FROM gcr.io/distroless/cc-debian13
COPY --from=build-env /app/target/release/mastr-export /
ENTRYPOINT ["/mastr-export"]
# Run with `podman run --transient-store --network=none mastr-export:latest` for
# fast startup
