# 🦖 ZIK_ZAK Server

The revolutionary Rust server that eliminates backend development forever.

## 🚀 Quick Start

```bash
# Install TigerBeetle (macOS)
curl -L https://mac.tigerbeetle.com | bash

# Format TigerBeetle database
tigerbeetle format --cluster=0 --replica=0 --replica-count=1 0_0.tigerbeetle

# Start TigerBeetle
tigerbeetle start --addresses=3003 0_0.tigerbeetle &

# Run ZIK_ZAK
cargo run --bin zik_zak
```

## 🎯 Test the Revolution

```bash
# Health check
curl localhost:3002/health

# Create a product
curl -X POST localhost:3002/recipe/create_product \
  -H "Content-Type: application/json" \
  -d '{"id": "laptop", "name": "MacBook Pro", "price": 2999.99}'

# Check product exists
curl localhost:3002/balance/product:laptop:existence
# Returns: {"account_id": "product:laptop:existence", "balance": 1}

# Check product price
curl localhost:3002/balance/product:laptop:price  
# Returns: {"account_id": "product:laptop:price", "balance": 299999}
```

## 🔥 The Magic

**Everything is an account:**
- `product:laptop:existence = 1` ← Product exists
- `product:laptop:price = 299999` ← Price in cents ($2999.99)
- `product:laptop:name = hash("MacBook Pro")` ← Name hash

**Every operation is a transfer:**
- Create: `transfer(genesis → product:laptop:existence, 1)`
- Update: `transfer(genesis → product:laptop:price, new_price)`
- Delete: `transfer(product:laptop:* → system:deleted, balance)`

## 📊 Recipes

Edit `recipes.json` to add new operations. No code changes needed:

```json
{
  "create_user": {
    "description": "Create a new user",
    "inputs": ["id", "email", "name"],
    "operations": [
      {
        "type": "transfer",
        "from": "system:genesis",
        "to": "user:{id}:existence",
        "amount": 1
      }
    ]
  }
}
```

## 🦖 The Revolution

- **2 functions** replace entire backends
- **1 JSON file** defines all operations  
- **0 schemas** needed ever
- **∞ flexibility** for any field
- **100% audit trail** built-in

**Backend development is dead. Welcome to the future.** 🚀