#!/usr/bin/env node

/**
 * 🦖 ZIK_ZAK MCP SERVER
 * =====================
 *
 * THE ULTIMATE SIMPLIFICATION:
 * - NO CODE, NO FRAMEWORKS
 * - JUST 2 MCP FUNCTIONS: balance() + transfer()
 * - DOUBLE-ENTRY ACCOUNTING IN 300 LINES
 * - SINGLE JSON DEFINES EVERYTHING
 */

import { Server } from "@modelcontextprotocol/sdk/server/index.js";
import { StdioServerTransport } from "@modelcontextprotocol/sdk/server/stdio.js";
import {
  CallToolRequestSchema,
  ListToolsRequestSchema,
} from "@modelcontextprotocol/sdk/types.js";
import fs from "fs";
import crypto from "crypto";

class ZikZakEngine {
  constructor() {
    // In-memory ledger: account_id -> balance
    this.ledger = new Map();

    // Transaction log: all transfers
    this.transactions = [];

    // Recipe definitions
    this.recipes = this.loadRecipes();

    console.error("🦖 ZikZak Engine initialized");
  }

  loadRecipes() {
    try {
      const recipesPath = "../recipes.json";
      const recipesData = fs.readFileSync(recipesPath, "utf8");
      return JSON.parse(recipesData);
    } catch (error) {
      console.error("Failed to load recipes:", error);
      return { primitives: {}, entities: {}, recipes: {} };
    }
  }

  // ============================================
  // PRIMITIVE 1: GET BALANCE
  // ============================================
  balance(accountId) {
    const balance = this.ledger.get(accountId) || 0;
    console.error(`💰 BALANCE: ${accountId} = ${balance}`);
    return balance;
  }

  // ============================================
  // PRIMITIVE 2: TRANSFER AMOUNT
  // ============================================
  transfer(fromAccount, toAccount, amount, metadata = {}) {
    // Validate transfer
    if (amount <= 0) {
      throw new Error(`Invalid amount: ${amount}`);
    }

    // Special case: system:genesis has unlimited funds
    if (fromAccount !== "system:genesis") {
      const fromBalance = this.balance(fromAccount);
      if (fromBalance < amount) {
        throw new Error(
          `Insufficient funds: ${fromAccount} has ${fromBalance}, needs ${amount}`,
        );
      }
    }

    // Execute transfer
    if (fromAccount !== "system:genesis") {
      this.ledger.set(fromAccount, this.balance(fromAccount) - amount);
    }
    this.ledger.set(toAccount, this.balance(toAccount) + amount);

    // Log transaction
    const transferId = crypto.randomUUID();
    const transaction = {
      id: transferId,
      from: fromAccount,
      to: toAccount,
      amount,
      metadata,
      timestamp: Date.now(),
    };
    this.transactions.push(transaction);

    console.error(
      `💸 TRANSFER: ${fromAccount} → ${toAccount} (${amount}) [${transferId}]`,
    );
    return transferId;
  }

  // ============================================
  // RECIPE EXECUTION ENGINE
  // ============================================
  executeRecipe(recipeName, inputs) {
    const recipe = this.recipes.recipes[recipeName];
    if (!recipe) {
      throw new Error(`Recipe not found: ${recipeName}`);
    }

    console.error(`🍳 EXECUTING RECIPE: ${recipeName}`);
    console.error(`📥 INPUTS:`, inputs);

    const results = {};
    const stored = {};

    for (const operation of recipe.operations) {
      try {
        const result = this.executeOperation(operation, inputs, stored);
        if (operation.store_as) {
          stored[operation.store_as] = result;
        }
        if (operation.on_fail && result === null) {
          return null;
        }
      } catch (error) {
        if (operation.on_fail) {
          return null;
        }
        throw error;
      }
    }

    // Build return value
    if (recipe.return) {
      const returnValue = {};
      for (const [key, template] of Object.entries(recipe.return)) {
        returnValue[key] = this.interpolate(template, inputs, stored);
      }
      return returnValue;
    }

    return stored;
  }

  executeOperation(operation, inputs, stored) {
    switch (operation.type) {
      case "transfer":
        const from = this.interpolate(operation.from, inputs, stored);
        const to = this.interpolate(operation.to, inputs, stored);
        const amount = this.evaluateAmount(operation.amount, inputs, stored);
        const metadata = this.interpolateObject(
          operation.metadata || {},
          inputs,
          stored,
        );
        return this.transfer(from, to, amount, metadata);

      case "balance":
        const account = this.interpolate(operation.account, inputs, stored);
        const balance = this.balance(account);

        if (operation.condition) {
          const condition = operation.condition;
          if (condition === "> 0" && balance <= 0) {
            throw new Error(
              `Balance condition failed: ${account} = ${balance}`,
            );
          }
        }
        return balance;

      case "get_metadata":
        const metaAccount = this.interpolate(operation.account, inputs, stored);
        const field = operation.field;

        // Find the latest transaction for this account to get metadata
        const transactions = this.transactions
          .filter((t) => t.to === metaAccount)
          .sort((a, b) => b.timestamp - a.timestamp);

        if (transactions.length > 0 && transactions[0].metadata[field]) {
          return transactions[0].metadata[field];
        }
        return null;

      default:
        throw new Error(`Unknown operation type: ${operation.type}`);
    }
  }

  interpolate(template, inputs, stored) {
    if (typeof template !== "string") return template;

    let result = template;

    // Replace input variables
    for (const [key, value] of Object.entries(inputs)) {
      result = result.replace(new RegExp(`\\{${key}\\}`, "g"), value);
    }

    // Replace stored variables
    for (const [key, value] of Object.entries(stored)) {
      result = result.replace(new RegExp(`\\{${key}\\}`, "g"), value);
    }

    return result;
  }

  interpolateObject(obj, inputs, stored) {
    const result = {};
    for (const [key, value] of Object.entries(obj)) {
      result[key] = this.interpolate(value, inputs, stored);
    }
    return result;
  }

  evaluateAmount(amountExpr, inputs, stored) {
    if (typeof amountExpr === "number") return amountExpr;

    const interpolated = this.interpolate(amountExpr, inputs, stored);

    // Handle special functions
    if (interpolated.startsWith("hash(") && interpolated.endsWith(")")) {
      const value = interpolated.slice(5, -1);
      return parseInt(
        crypto.createHash("sha256").update(value).digest("hex").slice(0, 8),
        16,
      );
    }

    if (interpolated === "timestamp()") {
      return Date.now();
    }

    // Try to parse as number
    const num = parseFloat(interpolated);
    if (!isNaN(num)) return num;

    throw new Error(`Cannot evaluate amount: ${amountExpr}`);
  }

  // ============================================
  // DEBUG HELPERS
  // ============================================
  getLedgerState() {
    return Object.fromEntries(this.ledger);
  }

  getTransactionHistory() {
    return this.transactions;
  }
}

// ============================================
// MCP SERVER SETUP
// ============================================
const engine = new ZikZakEngine();
const server = new Server(
  {
    name: "zik-zak",
    version: "1.0.0",
  },
  {
    capabilities: {
      tools: {},
    },
  },
);

// List available tools
server.setRequestHandler(ListToolsRequestSchema, async () => {
  return {
    tools: [
      {
        name: "balance",
        description: "Get account balance: balance(account_id) -> number",
        inputSchema: {
          type: "object",
          properties: {
            account_id: {
              type: "string",
              description: "Account ID to check balance for",
            },
          },
          required: ["account_id"],
        },
      },
      {
        name: "transfer",
        description:
          "Transfer amount: transfer(from_account, to_account, amount, metadata) -> transfer_id",
        inputSchema: {
          type: "object",
          properties: {
            from_account: {
              type: "string",
              description: "Source account ID",
            },
            to_account: {
              type: "string",
              description: "Destination account ID",
            },
            amount: {
              type: "number",
              description: "Amount to transfer",
            },
            metadata: {
              type: "object",
              description: "Optional metadata for the transfer",
            },
          },
          required: ["from_account", "to_account", "amount"],
        },
      },
      {
        name: "execute_recipe",
        description: "Execute a CRUD recipe using accounting operations",
        inputSchema: {
          type: "object",
          properties: {
            recipe_name: {
              type: "string",
              description:
                "Name of recipe to execute (e.g., create_product, read_product)",
            },
            inputs: {
              type: "object",
              description: "Input parameters for the recipe",
            },
          },
          required: ["recipe_name", "inputs"],
        },
      },
      {
        name: "debug_ledger",
        description: "Get current ledger state for debugging",
        inputSchema: {
          type: "object",
          properties: {},
        },
      },
    ],
  };
});

// Handle tool calls
server.setRequestHandler(CallToolRequestSchema, async (request) => {
  const { name, arguments: args } = request.params;

  try {
    switch (name) {
      case "balance":
        const balance = engine.balance(args.account_id);
        return {
          content: [
            {
              type: "text",
              text: `Account ${args.account_id}: ${balance}`,
            },
          ],
        };

      case "transfer":
        const transferId = engine.transfer(
          args.from_account,
          args.to_account,
          args.amount,
          args.metadata || {},
        );
        return {
          content: [
            {
              type: "text",
              text: `Transfer completed: ${transferId}`,
            },
          ],
        };

      case "execute_recipe":
        const result = engine.executeRecipe(args.recipe_name, args.inputs);
        return {
          content: [
            {
              type: "text",
              text: JSON.stringify(result, null, 2),
            },
          ],
        };

      case "debug_ledger":
        const ledger = engine.getLedgerState();
        const transactions = engine.getTransactionHistory();
        return {
          content: [
            {
              type: "text",
              text: `LEDGER:\n${JSON.stringify(ledger, null, 2)}\n\nTRANSACTIONS:\n${JSON.stringify(transactions.slice(-10), null, 2)}`,
            },
          ],
        };

      default:
        throw new Error(`Unknown tool: ${name}`);
    }
  } catch (error) {
    return {
      content: [
        {
          type: "text",
          text: `Error: ${error.message}`,
        },
      ],
      isError: true,
    };
  }
});

// Start server
async function main() {
  const transport = new StdioServerTransport();
  await server.connect(transport);
  console.error("🦖 ZikZak MCP Server started");
}

main().catch(console.error);
