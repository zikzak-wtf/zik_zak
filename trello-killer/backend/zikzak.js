/**
 * 🦖 ZIK_ZAK ENGINE - Redis Implementation
 *
 * Pure accounting engine that mimics TigerBeetle behavior using Redis
 *
 * Core Operations:
 * - balance(account_id) -> Number
 * - transfer(from, to, amount, metadata) -> transfer_id
 * - hash_string(text) -> Number (i64-like hash)
 */

const crypto = require('crypto');

class ZikZakEngine {
  constructor(redisClient) {
    this.redis = redisClient;
    this.transferCounter = 0;
  }

  /**
   * Hash a string into an i64-like number (for text encoding)
   * Uses SHA-256 and takes first 8 bytes as signed 64-bit integer
   */
  static hashString(text) {
    const hash = crypto.createHash('sha256').update(text).digest();
    // Take first 8 bytes and convert to number (simulating i64)
    // Use DataView to get a proper 64-bit integer representation
    const buffer = hash.slice(0, 8);
    const view = new DataView(buffer.buffer, buffer.byteOffset, buffer.byteLength);
    // Get as BigInt and convert to regular number (may lose precision but ok for hashing)
    const bigInt = view.getBigInt64(0, false);
    return Number(bigInt);
  }

  /**
   * Get balance of an account
   * Returns 0 if account doesn't exist
   */
  async balance(accountId) {
    const value = await this.redis.hGet('balances', accountId);
    return value ? parseInt(value) : 0;
  }

  /**
   * Transfer amount from one account to another (double-entry accounting)
   * Stores metadata for text fields and audit trail
   *
   * @param {string} from - Source account ID
   * @param {string} to - Destination account ID
   * @param {number} amount - Amount to transfer
   * @param {object} metadata - Additional data (original text, etc.)
   * @returns {string} transfer_id
   */
  async transfer(from, to, amount, metadata = {}) {
    const transferId = `transfer_${++this.transferCounter}_${Date.now()}`;

    // Double-entry accounting (if not from system:genesis)
    if (from !== 'system:genesis') {
      const fromBalance = await this.balance(from);
      await this.redis.hSet('balances', from, fromBalance - amount);
    }

    const toBalance = await this.balance(to);
    await this.redis.hSet('balances', to, toBalance + amount);

    // Store transfer metadata (for audit trail and text storage)
    const transferData = {
      id: transferId,
      from,
      to,
      amount,
      timestamp: Date.now(),
      ...metadata
    };

    await this.redis.hSet('transfers', transferId, JSON.stringify(transferData));

    // If metadata has original_text, store it separately for easy retrieval
    if (metadata.original_text) {
      await this.redis.hSet('metadata', to, metadata.original_text);
    }

    return transferId;
  }

  /**
   * Get original text from metadata storage
   */
  async getMetadata(accountId) {
    return await this.redis.hGet('metadata', accountId);
  }

  /**
   * Get transfer history (for audit trail)
   */
  async getTransferHistory(limit = 100) {
    const transfers = await this.redis.hGetAll('transfers');
    const history = Object.values(transfers)
      .map(t => JSON.parse(t))
      .sort((a, b) => b.timestamp - a.timestamp)
      .slice(0, limit);
    return history;
  }

  /**
   * Get all accounts with a specific prefix (useful for queries)
   */
  async getAccountsByPrefix(prefix) {
    const allBalances = await this.redis.hGetAll('balances');
    const matching = {};

    for (const [account, balance] of Object.entries(allBalances)) {
      if (account.startsWith(prefix)) {
        matching[account] = parseInt(balance);
      }
    }

    return matching;
  }
}

module.exports = ZikZakEngine;
