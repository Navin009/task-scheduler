FROM task-scheduler-base:latest AS builder

# Use the pre-built binary from the base image
# Just build this specific service to ensure it is up to date
RUN cargo build --release --bin task_recurrence_manager

FROM task-scheduler-base:runtime AS runtime

# Copy only the needed binary
COPY --from=builder /usr/src/app/target/release/task_recurrence_manager /usr/local/bin/

# Set an explicit user for better security
USER nobody

CMD ["task_recurrence_manager"]