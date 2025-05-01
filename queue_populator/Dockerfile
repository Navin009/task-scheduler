FROM task-scheduler-base:latest AS builder

# Use the pre-built binary from the base image
# Just build this specific service to ensure it is up to date
RUN cargo build --release --bin queue_populator

FROM task-scheduler-base:runtime AS runtime

# Copy only the needed binary
COPY --from=builder /usr/src/app/target/release/queue_populator /usr/local/bin/

# Set an explicit user for better security
USER nobody

CMD ["queue_populator"]