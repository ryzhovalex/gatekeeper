FROM rust:1.81-bullseye AS build
RUN cargo new --bin app
WORKDIR /app
COPY . .
RUN cargo build --release

FROM postgres:15
COPY --from=build /app/corund.cfg.yml /app/
COPY --from=build /app/target/release/corund_app /app/main
ENV CORUND_MODE prod
CMD "/app/main"
