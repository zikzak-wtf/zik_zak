import { useState, useEffect } from 'react';
import { DragDropContext, Droppable, Draggable, DropResult } from '@hello-pangea/dnd';
import { Plus, Trash2 } from 'lucide-react';
import './App.css';

const API_URL = 'http://localhost:3001';

interface Card {
  id: string;
  title: string;
  list: string;
  position: number;
}

interface Board {
  todo: Card[];
  in_progress: Card[];
  done: Card[];
}

const listNames = {
  todo: '📋 To Do',
  in_progress: '🚧 In Progress',
  done: '✅ Done'
};

function App() {
  const [board, setBoard] = useState<Board>({
    todo: [],
    in_progress: [],
    done: []
  });
  const [newCardTitle, setNewCardTitle] = useState<{ [key: string]: string }>({
    todo: '',
    in_progress: '',
    done: ''
  });
  const [isLoading, setIsLoading] = useState(true);

  // Load board from ZIK_ZAK backend
  const loadBoard = async () => {
    try {
      const response = await fetch(`${API_URL}/recipe/get_board`);
      const data = await response.json();
      setBoard(data.board);
      setIsLoading(false);
    } catch (error) {
      console.error('Failed to load board:', error);
      setIsLoading(false);
    }
  };

  useEffect(() => {
    loadBoard();
  }, []);

  // Create card using ZIK_ZAK transfer operations
  const createCard = async (list: string) => {
    const title = newCardTitle[list].trim();
    if (!title) return;

    const cardId = `card_${Date.now()}_${Math.random().toString(36).substr(2, 9)}`;

    try {
      await fetch(`${API_URL}/recipe/create_card`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ id: cardId, title, list })
      });

      // Clear input
      setNewCardTitle({ ...newCardTitle, [list]: '' });

      // Reload board
      await loadBoard();
    } catch (error) {
      console.error('Failed to create card:', error);
    }
  };

  // Delete card
  const deleteCard = async (cardId: string) => {
    try {
      await fetch(`${API_URL}/recipe/delete_card/${cardId}`, {
        method: 'DELETE'
      });
      await loadBoard();
    } catch (error) {
      console.error('Failed to delete card:', error);
    }
  };

  // Handle drag and drop (move card via accounting transfer!)
  const onDragEnd = async (result: DropResult) => {
    const { source, destination, draggableId } = result;

    if (!destination) return;

    const sourceList = source.droppableId as keyof Board;
    const destList = destination.droppableId as keyof Board;

    // Same position, same list - no change
    if (sourceList === destList && source.index === destination.index) return;

    // Move card via ZIK_ZAK accounting transfer!
    try {
      await fetch(`${API_URL}/recipe/move_card`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          card_id: draggableId,
          new_list: destList,
          new_position: destination.index
        })
      });

      await loadBoard();
    } catch (error) {
      console.error('Failed to move card:', error);
    }
  };

  if (isLoading) {
    return (
      <div className="min-h-screen flex items-center justify-center">
        <div className="text-white text-2xl">Loading ZIK_ZAK Trello Killer... 🦖</div>
      </div>
    );
  }

  return (
    <div className="min-h-screen p-8">
      {/* Header */}
      <div className="mb-8 text-center">
        <h1 className="text-5xl font-bold text-white mb-2">
          🦖 TRELLO KILLER 🦖
        </h1>
        <p className="text-white/80 text-lg">
          Built with <span className="font-bold">pure accounting</span> · Powered by ZIK_ZAK
        </p>
        <p className="text-white/60 text-sm mt-2">
          No schemas · No migrations · Just transfers · Backend is DEAD 💀
        </p>
      </div>

      {/* Board */}
      <DragDropContext onDragEnd={onDragEnd}>
        <div className="grid grid-cols-1 md:grid-cols-3 gap-6">
          {(Object.keys(listNames) as Array<keyof Board>).map((listKey) => (
            <div key={listKey} className="flex flex-col">
              {/* List Header */}
              <div className="bg-white/10 backdrop-blur-sm rounded-t-lg p-4 border-b-2 border-white/20">
                <h2 className="text-xl font-semibold text-white">
                  {listNames[listKey]}
                </h2>
                <p className="text-white/60 text-sm mt-1">
                  {board[listKey].length} cards
                </p>
              </div>

              {/* Droppable List */}
              <Droppable droppableId={listKey}>
                {(provided, snapshot) => (
                  <div
                    ref={provided.innerRef}
                    {...provided.droppableProps}
                    className={`flex-1 bg-white/5 backdrop-blur-sm rounded-b-lg p-4 min-h-[400px] transition-colors ${
                      snapshot.isDraggingOver ? 'bg-white/10' : ''
                    }`}
                  >
                    {/* Add Card Input */}
                    <div className="mb-4">
                      <div className="flex gap-2">
                        <input
                          type="text"
                          value={newCardTitle[listKey]}
                          onChange={(e) =>
                            setNewCardTitle({ ...newCardTitle, [listKey]: e.target.value })
                          }
                          onKeyPress={(e) => {
                            if (e.key === 'Enter') createCard(listKey);
                          }}
                          placeholder="Add a card..."
                          className="flex-1 bg-white/10 text-white placeholder-white/40 px-3 py-2 rounded border border-white/20 focus:outline-none focus:border-white/40"
                        />
                        <button
                          onClick={() => createCard(listKey)}
                          className="bg-white/20 hover:bg-white/30 text-white px-4 py-2 rounded transition-colors"
                        >
                          <Plus size={20} />
                        </button>
                      </div>
                    </div>

                    {/* Cards */}
                    <div className="space-y-2">
                      {board[listKey].map((card, index) => (
                        <Draggable key={card.id} draggableId={card.id} index={index}>
                          {(provided, snapshot) => (
                            <div
                              ref={provided.innerRef}
                              {...provided.draggableProps}
                              {...provided.dragHandleProps}
                              className={`bg-white/90 rounded-lg p-3 shadow-lg transition-all ${
                                snapshot.isDragging ? 'rotate-2 scale-105' : ''
                              }`}
                            >
                              <div className="flex items-start justify-between gap-2">
                                <p className="text-gray-800 flex-1">{card.title}</p>
                                <button
                                  onClick={() => deleteCard(card.id)}
                                  className="text-red-500 hover:text-red-700 transition-colors"
                                >
                                  <Trash2 size={16} />
                                </button>
                              </div>
                            </div>
                          )}
                        </Draggable>
                      ))}
                      {provided.placeholder}
                    </div>
                  </div>
                )}
              </Droppable>
            </div>
          ))}
        </div>
      </DragDropContext>

      {/* Footer */}
      <div className="mt-8 text-center text-white/60 text-sm">
        <p>🦖 Everything is accounting · Trello is garbage · Welcome to the revolution</p>
      </div>
    </div>
  );
}

export default App;
