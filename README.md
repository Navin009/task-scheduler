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

## Architecture Overview (Updated)

The Task Scheduler system is now designed for multi-tenant SaaS use, with clear separation between API authentication and user management. The Task Scheduler API supports only API key authentication for producer services. User management (login, onboarding, roles) is handled by a separate backend (not included here), which manages users, merchants, and roles.

### Key Points
- **Task Scheduler API**: Only supports API key authentication for service-to-service (producer) access.
- **Jobs**: Each job is linked to a merchant (multi-tenant support).
- **Templates**: Can be enabled/disabled via the `active` boolean field.
- **User Management**: Handled by a separate backend/UI, not by the Task Scheduler API.

### Updated Database Schema
- **merchants**: Represents each tenant/merchant.
- **users**: Stores user credentials and roles (super admin, merchant admin, merchant member). Managed by the user management backend.
- **api_keys**: Stores API keys for producer services, with expiry and permissions. Used for authenticating API requests to the Task Scheduler API.
- **jobs**: Now includes a `merchant_id` column to link jobs to merchants.
- **templates**: Has an `active` boolean to enable/disable templates.

### API Authentication
- All API requests to the Task Scheduler API must include a valid API key in the `x-api-key` header.
- The API key is checked for validity, expiry, and permissions.
- No user login or session management is handled by the Task Scheduler API.

### Example Architecture Diagram

```mermaid
graph TD
    subgraph UI & User Management
        A[Scheduler Monitoring UI]
        B[User Management Backend]
        A -- login, user CRUD --> B
    end
    subgraph Task Scheduler System
        C[Task Scheduler API]
        D[Job Executor, Watcher, etc.]
        E[(Database)]
        F[Producer Service]
        F -- API Key Auth --> C
        C -- jobs, templates, merchants --> E
        D -- jobs --> E
    end
    B -- manages users, merchants, api_keys --> E
    A -- monitoring, job queries (via B) --> C
```

### Example: API Key Auth for Getting Jobs

```http
GET /jobs
x-api-key: <producer-api-key>
```
- Only requests with a valid, non-expired API key are authorized.
- Jobs returned are filtered by the merchant associated with the API key.

### Example: Enabling/Disabling a Template

```http
PATCH /templates/<template_id>
{
  "active": false
}
```
- Only available to authorized producer services via API key.

### Migration Changes
- See `task_scheduler_migrations/migrations/001_consolidated_schema.sql` for new tables: `merchants`, `users`, `api_keys`.
- `jobs` table now includes `merchant_id`.
- `templates` table already includes `active` for enable/disable.
