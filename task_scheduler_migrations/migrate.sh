#!/bin/bash

# Load environment variables
if [ -f .env ]; then
    export $(cat .env | grep -v '^#' | xargs)
fi

# Default values
DATABASE_HOST=${DATABASE_HOST:-localhost}
DATABASE_PORT=${DATABASE_PORT:-5432}
DATABASE_USER=${DATABASE_USER:-task_scheduler}
DATABASE_PASSWORD=${DATABASE_PASSWORD:-task_scheduler}
DATABASE_NAME=${DATABASE_NAME:-task_scheduler}

# Construct database URL
DATABASE_URL="postgres://${DATABASE_USER}:${DATABASE_PASSWORD}@${DATABASE_HOST}:${DATABASE_PORT}/${DATABASE_NAME}"

echo "Running migrations with database URL: postgres://${DATABASE_USER}:****@${DATABASE_HOST}:${DATABASE_PORT}/${DATABASE_NAME}"

# Run cleanup script using Docker's psql
echo "Running cleanup script..."
docker-compose exec -T postgres psql -U ${DATABASE_USER} -d ${DATABASE_NAME} -f - < "$(dirname "$0")/cleanup.sql"

# Run migrations
echo "Running migrations..."
cd "$(dirname "$0")"
sqlx migrate run --database-url "${DATABASE_URL}" --source ./migrations

# Check if migration was successful
if [ $? -eq 0 ]; then
    echo "Migrations completed successfully"
else
    echo "Migration failed"
    exit 1
fi 