#!/usr/bin/env node

/**
 * 🦖 DEMO: Pure Accounting System
 * ================================
 *
 * Shows how ALL CRUD operations map to just 2 accounting primitives!
 */

console.log("🦖 PURE ACCOUNTING DEMO");
console.log("========================");
console.log("ALL CRUD OPERATIONS VIA 2 PRIMITIVES: balance() + transfer()");
console.log("");

// Simulate MCP calls (in real usage, these would be MCP function calls)
class AccountingMCPClient {
  constructor() {
    this.ledger = new Map();
    this.transactions = [];
  }

  balance(accountId) {
    const balance = this.ledger.get(accountId) || 0;
    console.log(`💰 balance("${accountId}") = ${balance}`);
    return balance;
  }

  transfer(from, to, amount, metadata = {}) {
    console.log(
      `💸 transfer("${from}", "${to}", ${amount}, ${JSON.stringify(metadata)})`,
    );

    if (from !== "system:genesis") {
      const fromBalance = this.balance(from);
      if (fromBalance < amount) {
        throw new Error(`Insufficient funds`);
      }
      this.ledger.set(from, fromBalance - amount);
    }

    this.ledger.set(to, this.balance(to) + amount);

    const transferId = `tx_${Date.now()}`;
    this.transactions.push({
      id: transferId,
      from,
      to,
      amount,
      metadata,
      timestamp: Date.now(),
    });
    return transferId;
  }

  getMetadata(account, field) {
    const tx = this.transactions.filter((t) => t.to === account).pop();
    return tx?.metadata[field] || null;
  }
}

const client = new AccountingMCPClient();

console.log("🍳 RECIPE 1: CREATE PRODUCT");
console.log("============================");

// Create Product Recipe (from recipes.json)
const productId = "prod_123";
const productData = {
  name: "Gaming Laptop",
  price: 1299.99,
  description: "High-performance gaming laptop",
};

console.log(`Creating product: ${JSON.stringify(productData)}`);
console.log("");

// Recipe execution: create_product
console.log("Executing create_product recipe...");

// Step 1: Mark existence
client.transfer("system:genesis", `product:${productId}:existence`, 1, {
  operation: "create_product",
  id: productId,
});

// Step 2: Set price
client.transfer(
  "system:genesis",
  `product:${productId}:price`,
  productData.price,
  { operation: "set_price", price: productData.price },
);

// Step 3: Set name (as hash)
const nameHash = parseInt(
  productData.name.split("").reduce((a, b) => a + b.charCodeAt(0), 0),
);
client.transfer("system:genesis", `product:${productId}:name`, nameHash, {
  operation: "set_name",
  name: productData.name,
});

// Step 4: Set description (as hash)
const descHash = parseInt(
  productData.description.split("").reduce((a, b) => a + b.charCodeAt(0), 0),
);
client.transfer(
  "system:genesis",
  `product:${productId}:description`,
  descHash,
  { operation: "set_description", description: productData.description },
);

// Step 5: Set timestamp
client.transfer(
  "system:genesis",
  `product:${productId}:created_at`,
  Date.now(),
  { operation: "set_created_at", created_at: new Date().toISOString() },
);

console.log("✅ Product created via pure accounting!");
console.log("");

console.log("🍳 RECIPE 2: READ PRODUCT");
console.log("==========================");

// Recipe execution: read_product
console.log(`Reading product: ${productId}`);
console.log("");

// Step 1: Check existence
const exists = client.balance(`product:${productId}:existence`);
if (exists <= 0) {
  console.log("❌ Product not found");
  process.exit(1);
}

// Step 2: Get all attributes
const price = client.balance(`product:${productId}:price`);
const name = client.getMetadata(`product:${productId}:name`, "name");
const description = client.getMetadata(
  `product:${productId}:description`,
  "description",
);
const createdAt = client.getMetadata(
  `product:${productId}:created_at`,
  "created_at",
);

const product = {
  id: productId,
  name,
  price,
  description,
  created_at: createdAt,
};

console.log("✅ Product read via pure accounting:");
console.log(JSON.stringify(product, null, 2));
console.log("");

console.log("📊 FINAL LEDGER STATE");
console.log("=====================");
const ledger = Object.fromEntries(client.ledger);
console.log(JSON.stringify(ledger, null, 2));

console.log("");
console.log("🎯 KEY INSIGHTS:");
console.log("================");
console.log("✨ NO CODE - just accounting recipes!");
console.log("✨ NO FRAMEWORKS - just 2 primitives!");
console.log("✨ NO DATABASES - just account balances!");
console.log("✨ UNIVERSAL - works for ANY entity type!");
console.log("✨ AUDITABLE - complete transaction history!");
console.log("");
console.log("🦖 This is the FUTURE of software architecture!");
