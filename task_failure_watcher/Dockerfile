FROM task-scheduler-base as builder

RUN cargo build --release --bin task_failure_watcher

FROM debian:bullseye-slim
RUN apt-get update && apt-get install -y libssl-dev && rm -rf /var/lib/apt/lists/*
COPY --from=builder /usr/src/app/target/release/task_failure_watcher /usr/local/bin/

CMD ["task_failure_watcher"]