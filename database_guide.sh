#!/bin/bash

echo "🗄️  PocketFlow-RS Database Examples"
echo "======================================"
echo

echo "1️⃣  Testing SQLite (Lightweight, Serverless)"
echo "----------------------------------------------"
echo "Command: cargo run --example database_storage --features database-sqlite"
echo "Use case: Local development, small applications, embedded systems"
echo

echo "2️⃣  Testing PostgreSQL (Advanced Features)"
echo "-------------------------------------------"
echo "Command: cargo run --example postgres_storage --features database-postgres"
echo "Requirements: PostgreSQL server running"
echo "Use case: Complex applications, JSONB data, advanced queries"
echo "Environment: export DATABASE_POSTGRES_URL=\"postgres://user:password@localhost:5432/pocketflow\""
echo

echo "3️⃣  Testing MySQL (Web Applications)"
echo "------------------------------------"
echo "Command: cargo run --example mysql_storage --features database-mysql"
echo "Requirements: MySQL server running"
echo "Use case: Web applications, e-commerce, content management"
echo "Environment: export DATABASE_MYSQL_URL=\"mysql://root:password@localhost:3306/pocketflow\""
echo

echo "4️⃣  All Databases"
echo "-----------------"
echo "Command: cargo run --example database_storage --features database"
echo "Note: Automatically detects and uses SQLite by default"
echo

echo "🐳 Docker Quick Start"
echo "====================="
echo "PostgreSQL: docker run --rm --name postgres -e POSTGRES_PASSWORD=password -e POSTGRES_DB=pocketflow -p 5432:5432 -d postgres:15"
echo "MySQL:      docker run --rm --name mysql -e MYSQL_ROOT_PASSWORD=password -e MYSQL_DATABASE=pocketflow -p 3306:3306 -d mysql:8"
echo

echo "⚡ Performance Comparison"
echo "========================"
echo "SQLite:     Best for < 100GB, single-user applications"
echo "PostgreSQL: Best for complex queries, concurrent users, JSON data"
echo "MySQL:      Best for web apps, read-heavy workloads, replication"
echo

echo "🎯 Feature Matrix"
echo "================"
echo "Feature          | SQLite | PostgreSQL | MySQL"
echo "-----------------|--------|------------|-------"
echo "JSON Support     |   ✓    |    ✓✓✓     |  ✓✓"
echo "Concurrency      |   ✓    |    ✓✓✓     |  ✓✓"
echo "Full-Text Search |   ✓    |    ✓✓✓     |  ✓✓"
echo "Replication      |   ✗    |    ✓✓✓     |  ✓✓✓"
echo "Setup Complexity | None   |   Medium   | Medium"
echo "Memory Usage     |  Low   |   Medium   | Medium"
echo
  
echo "Happy coding! 🚀"