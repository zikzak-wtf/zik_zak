/**
 * 🦖 TRELLO KILLER - ZIK_ZAK Backend Server
 *
 * Everything is accounting. No schemas. No migrations. Just transfers.
 */

const express = require('express');
const cors = require('cors');
const { createClient } = require('redis');
const ZikZakEngine = require('./zikzak');

const app = express();
const PORT = 3001;

app.use(cors());
app.use(express.json());

let engine;
let redisClient;

// Initialize Redis and ZIK_ZAK engine
async function initializeEngine() {
  redisClient = createClient({
    url: 'redis://localhost:6379'
  });

  redisClient.on('error', (err) => console.log('Redis Client Error', err));

  await redisClient.connect();
  console.log('✅ Connected to Redis');

  engine = new ZikZakEngine(redisClient);
  console.log('✅ ZIK_ZAK Engine initialized');
}

// ========================================
// CORE ZIK_ZAK API
// ========================================

/**
 * GET /balance/:account_id
 * Get balance of any account
 */
app.get('/balance/:account_id', async (req, res) => {
  try {
    const balance = await engine.balance(req.params.account_id);
    res.json({ account_id: req.params.account_id, balance });
  } catch (error) {
    res.status(500).json({ error: error.message });
  }
});

/**
 * POST /transfer
 * Transfer between accounts (the ONLY write operation!)
 */
app.post('/transfer', async (req, res) => {
  try {
    const { from, to, amount, metadata } = req.body;
    const transferId = await engine.transfer(from, to, amount, metadata);
    res.json({ transfer_id: transferId, success: true });
  } catch (error) {
    res.status(500).json({ error: error.message });
  }
});

/**
 * GET /metadata/:account_id
 * Get metadata (original text) for an account
 */
app.get('/metadata/:account_id', async (req, res) => {
  try {
    const metadata = await engine.getMetadata(req.params.account_id);
    res.json({ account_id: req.params.account_id, metadata });
  } catch (error) {
    res.status(500).json({ error: error.message });
  }
});

// ========================================
// TRELLO-SPECIFIC RECIPES (Built on transfers!)
// ========================================

/**
 * POST /recipe/create_card
 * Create a card using pure accounting
 */
app.post('/recipe/create_card', async (req, res) => {
  try {
    const { id, title, list } = req.body; // list: 'todo' | 'in_progress' | 'done'

    // Hash the title
    const titleHash = ZikZakEngine.hashString(title);

    // Execute transfers to create the card
    // 1. Card existence
    await engine.transfer('system:genesis', `card:${id}:existence`, 1, {
      operation: 'create_card',
      card_id: id
    });

    // 2. Card title (as hash)
    await engine.transfer('system:genesis', `card:${id}:title_hash`, titleHash, {
      operation: 'store_title',
      original_text: title
    });

    // 3. Card list (1=todo, 2=in_progress, 3=done)
    const listNum = { todo: 1, in_progress: 2, done: 3 }[list] || 1;
    await engine.transfer('system:genesis', `card:${id}:list`, listNum, {
      operation: 'set_list',
      list_name: list
    });

    // 4. Card position (start at 0)
    await engine.transfer('system:genesis', `card:${id}:position`, 0, {
      operation: 'set_position'
    });

    // 5. Timestamp
    await engine.transfer('system:genesis', `card:${id}:timestamp`, Date.now(), {
      operation: 'set_timestamp'
    });

    res.json({
      success: true,
      card_id: id,
      message: 'Card created via pure accounting! 🦖'
    });
  } catch (error) {
    res.status(500).json({ error: error.message });
  }
});

/**
 * POST /recipe/move_card
 * Move card between lists (accounting transfer!)
 */
app.post('/recipe/move_card', async (req, res) => {
  try {
    const { card_id, new_list, new_position } = req.body;

    // Get current list
    const currentList = await engine.balance(`card:${card_id}:list`);

    // Transfer to new list (subtract from old, add to new)
    const listNum = { todo: 1, in_progress: 2, done: 3 }[new_list] || 1;

    await engine.transfer(
      `card:${card_id}:list`,
      `card:${card_id}:list`,
      listNum - currentList, // Delta
      { operation: 'move_card', new_list }
    );

    // Update position
    const currentPosition = await engine.balance(`card:${card_id}:position`);
    await engine.transfer(
      `card:${card_id}:position`,
      `card:${card_id}:position`,
      new_position - currentPosition,
      { operation: 'update_position', new_position }
    );

    res.json({
      success: true,
      card_id,
      new_list,
      new_position,
      message: 'Card moved via accounting transfer! 🦖'
    });
  } catch (error) {
    res.status(500).json({ error: error.message });
  }
});

/**
 * GET /recipe/get_board
 * Get all cards organized by lists
 */
app.get('/recipe/get_board', async (req, res) => {
  try {
    // Get all card accounts
    const cardAccounts = await engine.getAccountsByPrefix('card:');

    // Extract unique card IDs
    const cardIds = new Set();
    Object.keys(cardAccounts).forEach(account => {
      const match = account.match(/^card:([^:]+):/);
      if (match) cardIds.add(match[1]);
    });

    // Build card objects
    const cards = [];
    for (const cardId of cardIds) {
      const existence = await engine.balance(`card:${cardId}:existence`);
      if (existence === 0) continue; // Card deleted

      const list = await engine.balance(`card:${cardId}:list`);
      const position = await engine.balance(`card:${cardId}:position`);
      const titleHash = await engine.balance(`card:${cardId}:title_hash`);
      const title = await engine.getMetadata(`card:${cardId}:title_hash`) || 'Untitled';

      const listName = ['', 'todo', 'in_progress', 'done'][list];

      cards.push({
        id: cardId,
        title,
        list: listName,
        position,
        titleHash
      });
    }

    // Organize by lists
    const board = {
      todo: cards.filter(c => c.list === 'todo').sort((a, b) => a.position - b.position),
      in_progress: cards.filter(c => c.list === 'in_progress').sort((a, b) => a.position - b.position),
      done: cards.filter(c => c.list === 'done').sort((a, b) => a.position - b.position)
    };

    res.json({ board, total_cards: cards.length });
  } catch (error) {
    res.status(500).json({ error: error.message });
  }
});

/**
 * DELETE /recipe/delete_card/:card_id
 * Delete a card (set existence to 0)
 */
app.delete('/recipe/delete_card/:card_id', async (req, res) => {
  try {
    const { card_id } = req.params;

    // Transfer existence back to system (deletion via accounting!)
    await engine.transfer(
      `card:${card_id}:existence`,
      'system:deleted',
      1,
      { operation: 'delete_card', card_id }
    );

    res.json({
      success: true,
      card_id,
      message: 'Card deleted via accounting transfer! 🦖'
    });
  } catch (error) {
    res.status(500).json({ error: error.message });
  }
});

// ========================================
// HEALTH & DEBUG
// ========================================

app.get('/health', (req, res) => {
  res.json({
    status: 'alive',
    message: 'ZIK_ZAK Trello Killer is running! 🦖',
    backend: 'DEAD',
    accounting: 'ALIVE'
  });
});

app.get('/debug/all-balances', async (req, res) => {
  try {
    const balances = await redisClient.hGetAll('balances');
    res.json({ balances });
  } catch (error) {
    res.status(500).json({ error: error.message });
  }
});

app.get('/debug/transfers', async (req, res) => {
  try {
    const history = await engine.getTransferHistory(50);
    res.json({ transfers: history });
  } catch (error) {
    res.status(500).json({ error: error.message });
  }
});

// Start server
initializeEngine().then(() => {
  app.listen(PORT, () => {
    console.log(`
🦖 TRELLO KILLER SERVER RUNNING! 🦖

Backend Development: DEAD 💀
Accounting Principles: ALIVE 🚀

Server: http://localhost:${PORT}
Health: http://localhost:${PORT}/health
Board: http://localhost:${PORT}/recipe/get_board

Core Operations:
- GET  /balance/:account_id
- POST /transfer
- GET  /metadata/:account_id

Trello Recipes:
- POST   /recipe/create_card
- POST   /recipe/move_card
- GET    /recipe/get_board
- DELETE /recipe/delete_card/:id

Everything is accounting. Trello is GARBAGE! 🦖
    `);
  });
});
