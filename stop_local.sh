#!/bin/bash

echo "Stopping all services..."

# Stop all running containers
docker-compose down

# Kill any remaining processes
pkill -f task_scheduler_api
pkill -f queue_populator
pkill -f task_executor
pkill -f task_failure_watcher
pkill -f task_recurrence_manager

echo "All services stopped!" 