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

# Run migrations
sqlx migrate run --database-url "${DATABASE_URL}" --source ./migrations

# Check if migration was successful
if [ $? -eq 0 ]; then
    echo "Migrations completed successfully"
else
    echo "Migration failed"
    exit 1
fi 