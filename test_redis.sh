#!/bin/bash

echo "🧪 Testing Redis Storage Example"
echo

# Check if redis feature compiles
echo "📦 Checking Redis feature compilation..."
if cargo check --features redis --quiet; then
    echo "✅ Redis feature compiles successfully"
else
    echo "❌ Redis feature compilation failed"
    exit 1
fi

# Check if redis example compiles  
echo "🔧 Checking Redis example compilation..."
if cargo check --example redis_storage --features redis --quiet; then
    echo "✅ Redis example compiles successfully"
else
    echo "❌ Redis example compilation failed"
    exit 1
fi

echo
echo "🎯 All checks passed! The Redis storage backend is ready to use."
echo
echo "📋 To run the example:"
echo "   1. Start Redis: docker run --rm -p 6379:6379 redis:latest"
echo "   2. Run example: cargo run --example redis_storage --features redis"
echo
echo "📚 See examples/README_redis.md for detailed instructions."