#!/bin/bash

# Exit on error
set -e

# Colors for output
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
BLUE='\033[0;34m'
MAGENTA='\033[0;35m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

# Service colors
API_COLOR=$CYAN
QUEUE_COLOR=$MAGENTA
EXECUTOR_COLOR=$GREEN
FAILURE_COLOR=$RED
RECURRENCE_COLOR=$BLUE

# Function to check if a command exists
check_command() {
    if ! command -v $1 &> /dev/null; then
        echo -e "${RED}Error: $1 is not installed${NC}"
        exit 1
    fi
}

# Function to check if Docker is running
check_docker_running() {
    if ! docker info &> /dev/null; then
        echo -e "${RED}Error: Docker is not running${NC}"
        exit 1
    fi
}

# Function to clean stale cargo locks
clean_cargo_locks() {
    echo -e "${YELLOW}Cleaning any stale Cargo locks...${NC}"
    find target -name ".rustc_info.json.*.lock" -delete 2>/dev/null || true
    find ~/.cargo -name "*.lock" -mtime +1 -delete 2>/dev/null || true
}

# Check required commands
echo -e "${YELLOW}Checking system requirements...${NC}"
check_command docker
check_command docker-compose
check_command cargo-watch
check_docker_running

# Check if cargo-watch is installed
if ! command -v cargo-watch &> /dev/null; then
    echo -e "${YELLOW}Installing cargo-watch...${NC}"
    cargo install cargo-watch
fi

# Source environment variables
echo -e "${YELLOW}Loading environment variables...${NC}"
if [ -f .env ]; then
    # Export all environment variables from .env file
    set -a
    source .env
    set +a
    
    # Set default queue names if not defined
    : ${QUEUE_NAMES__0:="default"}
    : ${QUEUE_NAMES__1:="high"}
    : ${QUEUE_NAMES__2:="low"}
else
    echo -e "${YELLOW}Warning: .env file not found, using default values${NC}"
    # Set default values
    export QUEUE_NAMES__0="default"
    export QUEUE_NAMES__1="high"
    export QUEUE_NAMES__2="low"
fi

# Verify required environment variables
required_vars=("DATABASE_URL" "REDIS_URL" "MAX_RETRIES")
for var in "${required_vars[@]}"; do
    if [ -z "${!var}" ]; then
        echo -e "${RED}Error: Required environment variable $var is not set${NC}"
        exit 1
    fi
done

echo -e "${YELLOW}Starting local development environment...${NC}"

# Check if containers are already running
if docker-compose ps postgres valkey | grep  "Up"; then
    echo -e "${YELLOW}PostgreSQL and Valkey containers are already running${NC}"
else
    # Start PostgreSQL and Valkey containers
    echo -e "${GREEN}Starting PostgreSQL and Valkey containers...${NC}"
    docker-compose up -d postgres valkey
fi

# Wait for PostgreSQL to be ready
echo -e "${YELLOW}Waiting for PostgreSQL to be ready...${NC}"
until docker-compose exec postgres pg_isready -U task_scheduler; do
    sleep 1
done

# Wait for Valkey to be ready
echo -e "${YELLOW}Waiting for Valkey to be ready...${NC}"
until docker-compose exec valkey redis-cli ping | grep  "PONG"; do
    sleep 1
done

# Clean any stale cargo locks
clean_cargo_locks

# Pre-build all binaries to avoid concurrent build issues
echo -e "${YELLOW}Pre-building all binaries to avoid file locks...${NC}"
cargo build --bin queue_populator --bin task_executor --bin task_failure_watcher --bin task_recurrence_manager --bin task_scheduler_api

# Start Queue Populator with watch
echo -e "${QUEUE_COLOR}[TASK QUEUE POPULATOR]${NC} Starting Queue Populator (with watch)..."
cargo watch -w queue_populator -w scheduler_core -x 'run --bin queue_populator' --no-restart &
sleep 2

# Start Task Executor with watch
echo -e "${EXECUTOR_COLOR}[TASK EXECUTOR]${NC} Starting Task Executor (with watch)..."
cargo watch -w task_executor -w scheduler_core -x 'run --bin task_executor' --no-restart &
sleep 2

# Start Task Failure Watcher with watch
echo -e "${FAILURE_COLOR}[TASK FAILURE WATCHER]${NC} Starting Task Failure Watcher (with watch)..."
cargo watch -w task_failure_watcher -w scheduler_core -x 'run --bin task_failure_watcher' --no-restart &
sleep 2

# Start Task Recurrence Manager with watch
echo -e "${RECURRENCE_COLOR}[TASK RECURRENCE MANAGER]${NC} Starting Task Recurrence Manager (with watch)..."
cargo watch -w task_recurrence_manager -w scheduler_core -x 'run --bin task_recurrence_manager' --no-restart &
sleep 2

# Start the API service with watch
echo -e "${API_COLOR}[TASK SCHEDULER API]${NC} Starting Task Scheduler API (with watch)..."
cargo watch -w task_scheduler_api -w scheduler_core -x 'run --bin task_scheduler_api' --no-restart &

echo -e "${GREEN}All services started in watch mode!${NC}"
echo -e "${YELLOW}Press Ctrl+C to stop all services${NC}"

# Wait for Ctrl+C
trap "echo -e '${YELLOW}Stopping all services...${NC}'; clean_cargo_locks; kill 0" EXIT
wait 