FROM task-scheduler-base as builder

RUN cargo build --release --bin task_scheduler_api

FROM debian:bullseye-slim
RUN apt-get update && apt-get install -y libssl-dev && rm -rf /var/lib/apt/lists/*
COPY --from=builder /usr/src/app/target/release/task_scheduler_api /usr/local/bin/

EXPOSE 8080
CMD ["task_scheduler_api"]