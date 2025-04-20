FROM task-scheduler-base as builder

RUN cargo build --release --bin queue_populator

FROM debian:bullseye-slim
RUN apt-get update && apt-get install -y libssl-dev && rm -rf /var/lib/apt/lists/*
COPY --from=builder /usr/src/app/target/release/queue_populator /usr/local/bin/

CMD ["queue_populator"]