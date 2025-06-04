#!/usr/bin/env bash

echo "🦖 Testing Realistic Product Recipes"
echo "====================================="

echo "📋 Step 1: Checking if server is running and recipes are loaded..."
python3 pure-accounting-server/check_recipes.py

if [ $? -eq 0 ]; then
    echo ""
    echo "🚀 Step 2: Running comprehensive product tests..."
    python3 pure-accounting-server/test_realistic_product.py
else
    echo ""
    echo "❌ Server not ready. Please:"
    echo "   1. Stop any existing server process (Ctrl+C or kill)"
    echo "   2. Start fresh server: cargo run"
    echo "   3. Run this script again"
fi