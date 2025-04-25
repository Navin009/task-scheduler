#!/bin/bash

# Exit on error
set -e

# Colors for output
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Source environment variables
echo -e "${YELLOW}Loading environment variables...${NC}"
if [ -f .env ]; then
    # Export non-array environment variables
    export $(grep -v '^#' .env | grep -v 'QUEUE_NAMES__' | xargs)

    # Export array environment variables
    export QUEUE_NAMES__0=$(grep '^QUEUE_NAMES__0=' .env | cut -d '=' -f2)
    export QUEUE_NAMES__1=$(grep '^QUEUE_NAMES__1=' .env | cut -d '=' -f2)
    export QUEUE_NAMES__2=$(grep '^QUEUE_NAMES__2=' .env | cut -d '=' -f2)
else
    echo -e "${YELLOW}Warning: .env file not found${NC}"
fi

echo -e "${YELLOW}Starting local development environment...${NC}"

# Start PostgreSQL and Valkey containers
echo -e "${GREEN}Starting PostgreSQL and Valkey containers...${NC}"
docker-compose up -d postgres valkey

# Wait for PostgreSQL to be ready
echo -e "${YELLOW}Waiting for PostgreSQL to be ready...${NC}"
until docker-compose exec postgres pg_isready -U task_scheduler; do
    sleep 1
done

# Start the API service
echo -e "${GREEN}Starting Task Scheduler API...${NC}"
DATABASE_URL=$DATABASE_URL REDIS_URL=$REDIS_URL QUEUE_NAMES__0=$QUEUE_NAMES__0 QUEUE_NAMES__1=$QUEUE_NAMES__1 QUEUE_NAMES__2=$QUEUE_NAMES__2 MAX_RETRIES=$MAX_RETRIES cargo run --bin task_scheduler_api &

# Start Queue Populator
echo -e "${GREEN}Starting Queue Populator...${NC}"
DATABASE_URL=$DATABASE_URL REDIS_URL=$REDIS_URL QUEUE_NAMES__0=$QUEUE_NAMES__0 QUEUE_NAMES__1=$QUEUE_NAMES__1 QUEUE_NAMES__2=$QUEUE_NAMES__2 MAX_RETRIES=$MAX_RETRIES cargo run --bin queue_populator &

# Start Task Executor
echo -e "${GREEN}Starting Task Executor...${NC}"
DATABASE_URL=$DATABASE_URL REDIS_URL=$REDIS_URL QUEUE_NAMES__0=$QUEUE_NAMES__0 QUEUE_NAMES__1=$QUEUE_NAMES__1 QUEUE_NAMES__2=$QUEUE_NAMES__2 MAX_RETRIES=$MAX_RETRIES cargo run --bin task_executor &

# Start Task Failure Watcher
echo -e "${GREEN}Starting Task Failure Watcher...${NC}"
DATABASE_URL=$DATABASE_URL REDIS_URL=$REDIS_URL QUEUE_NAMES__0=$QUEUE_NAMES__0 QUEUE_NAMES__1=$QUEUE_NAMES__1 QUEUE_NAMES__2=$QUEUE_NAMES__2 MAX_RETRIES=$MAX_RETRIES cargo run --bin task_failure_watcher &

# Start Task Recurrence Manager
echo -e "${GREEN}Starting Task Recurrence Manager...${NC}"
DATABASE_URL=$DATABASE_URL REDIS_URL=$REDIS_URL QUEUE_NAMES__0=$QUEUE_NAMES__0 QUEUE_NAMES__1=$QUEUE_NAMES__1 QUEUE_NAMES__2=$QUEUE_NAMES__2 MAX_RETRIES=$MAX_RETRIES cargo run --bin task_recurrence_manager &

echo -e "${GREEN}All services started!${NC}"
echo -e "${YELLOW}Press Ctrl+C to stop all services${NC}"

# Wait for Ctrl+C
trap "echo -e '${YELLOW}Stopping all services...${NC}'; kill 0" EXIT
wait 