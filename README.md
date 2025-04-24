# Task Scheduler

## Development Setup

### Prerequisites
- Docker
- Docker Compose

### Starting the Development Environment

1. Start the PostgreSQL and Redis services:
```bash
docker compose -f docker-compose.dev.yml up -d postges redis
```

2. The services will be available at:
   - PostgreSQL: localhost:5432
     - User: postgres
     - Password: postgres
     - Database: scheduler
   - Redis: localhost:6379

3. To stop the services:
```bash
docker compose -f docker-compose.yml down postgres redis
```

4. To view logs:
```bash
docker compose -f docker-compose.yml logs -f
```

5. To remove volumes and start fresh:
```bash
docker compose -f docker-compose.yml down -v
```

### Database Migrations

The migrations will run automatically when the application starts. The initial migration creates:
- `jobs` table for managing scheduled tasks
- `templates` table for job templates
- Required ENUM types for job status and schedule types

### Environment Variables

The following environment variables are configured in `.env`:
```
DATABASE_URL=postgres://postgres:postgres@localhost:5432/scheduler
REDIS_URL=redis://localhost:6379
```

### Development vs Production

- `docker-compose.dev.yml`: Used for local development, spins up only PostgreSQL and Redis
- `docker-compose.yml`: Full production setup with all microservices

## Project Structure Overview

Your final directory structure should look something like this:

```
task_scheduler_system/
├── Cargo.toml              # Workspace definition
├── scheduler_core/         # Shared library crate
│   ├── Cargo.toml
│   └── src/
│       └── lib.rs
│       └── models.rs      # Define Job/Template structs here
│       └── db.rs          # DB Pool setup, common queries
│       └── redis.rs       # Redis connection setup, queue ops
│       └── config.rs      # Config loading structs/logic
│       └── error.rs       # Common error enum/types
├── task_scheduler_api/     # API service binary crate
│   ├── Cargo.toml
│   └── src/
│       └── main.rs        # API server setup (e.g., Axum/Actix)
├── task_executor/          # Worker service binary crate
│   ├── Cargo.toml
│   └── src/
│       └── main.rs        # Redis BLPOP loop, job execution logic
├── task_failure_watcher/   # Watchdog service binary crate
│   ├── Cargo.toml
│   └── src/
│       └── main.rs        # Periodic DB scan logic
├── task_recurrence_manager/ # Recurrence manager binary crate
│   ├── Cargo.toml
│   └── src/
│       └── main.rs        # Periodic template scanning/generation logic
├── queue_populator/        # Queue populator binary crate
│   ├── Cargo.toml
│   └── src/
│       └── main.rs        # Periodic DB polling/update/Redis push logic
├── .gitignore              # Standard Rust gitignore (target/, Cargo.lock)
└── target/                 # Build artifacts (created by Cargo)
# Optional UI backend crate would be here too
# Optional /frontend directory for non-Rust UI code
```

**5. Building and Running**

- **Build all crates:** Run `cargo build` from the root (`task_scheduler_system/`) directory.
- **Build a specific crate:** `cargo build -p task_scheduler_api`
- **Run a specific service:** `cargo run -p task_scheduler_api`
- **Run tests for all crates:** `cargo test`
- **Run tests for a specific crate:** `cargo test -p scheduler_core`

This workspace setup provides a clean, maintainable structure for your multi-component Rust application, promoting code reuse through the `scheduler_core` crate while keeping each service distinct and independently buildable/runnable. Remember to populate the `src/` directories with the actual implementation logic based on the responsibilities you've defined.
