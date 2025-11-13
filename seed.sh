#!/usr/bin/env bash
# 数据库种子数据脚本

set -e

DATABASE_FILE="ferrusgate.db"

echo "🌱 Seeding database: $DATABASE_FILE"

if [ ! -f "$DATABASE_FILE" ]; then
    echo "❌ Database file not found. Please run 'cargo run' first to create the database."
    exit 1
fi

echo "📝 Executing seed.sql..."
sqlite3 "$DATABASE_FILE" < seed.sql

echo "✅ Database seeded successfully!"
echo ""
echo "📋 Seed data:"
echo "   Users:"
echo "     - testuser / password123"
echo "     - admin / password123"
echo ""
echo "   OAuth Clients:"
echo "     - client_id: test_client_123"
echo "       client_secret: test_secret_456"
echo "       redirect_uris: http://localhost:3000/callback, http://localhost:8080/callback"
echo ""
echo "     - client_id: demo_app"
echo "       client_secret: demo_secret_xyz"
echo "       redirect_uris: http://localhost:4000/auth/callback"
