FROM task-scheduler-base:latest AS builder

# Use the pre-built binary from the base image
# Just build this specific service to ensure it is up to date
RUN cargo build --release --bin task_scheduler_api

FROM task-scheduler-base:runtime AS runtime

# Copy only the needed binary
COPY --from=builder /usr/src/app/target/release/task_scheduler_api /usr/local/bin/

# Set an explicit user for better security
USER nobody

EXPOSE 8080
CMD ["task_scheduler_api"] 