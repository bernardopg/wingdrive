#!/bin/bash
# Quick start script for running WingDrive Server in Docker
# Usage: ./docker-run.sh

set -e

echo "🚀 Starting WingDrive Server with Docker Compose..."

# Check if .env exists
if [ ! -f .env ]; then
    echo "⚠️  No .env file found. Creating from template..."
    cp .env.example .env
    echo ""
    echo "📝 IMPORTANT: Edit .env and set your SD_AUTH credentials!"
    echo "   Default is 'admin:changeme' - please change this."
    echo ""
    read -p "Press enter to continue or Ctrl+C to abort..."
fi

# Check if docker-compose is available
if ! command -v docker-compose &> /dev/null; then
    if ! command -v docker &> /dev/null; then
        echo "❌ Docker not found. Please install Docker first."
        exit 1
    fi
    # Try docker compose (newer syntax)
    COMPOSE_CMD="docker compose"
else
    COMPOSE_CMD="docker-compose"
fi

echo "🏗️  Building and starting container..."
$COMPOSE_CMD up -d --build

echo ""
echo "✅ WingDrive Server is running!"
echo ""
echo "📍 Access your server at: http://localhost:8080"
echo "🔐 Login credentials: Check your .env file (SD_AUTH)"
echo ""
echo "Useful commands:"
echo "  - View logs:    $COMPOSE_CMD logs -f wingdrive"
echo "  - Stop server:  $COMPOSE_CMD down"
echo "  - Restart:      $COMPOSE_CMD restart"
echo "  - Shell access: $COMPOSE_CMD exec wingdrive sh"
echo ""
