// frontend/js/modules/drag-drop.js
// === Drag-and-Drop ===

import { setDraggedCard, setDraggedFromList, draggedCard, draggedFromList } from './state.js';
import { apiRequest } from './api.js';
import { showToast } from './toast.js';

export function handleDragStart(e, card, list) {
  setDraggedCard(card);
  setDraggedFromList(list);
  e.dataTransfer.effectAllowed = 'move';
  e.dataTransfer.setData('text/plain', JSON.stringify(card));
}

export function handleDragOver(e) {
  e.preventDefault();
  e.dataTransfer.dropEffect = 'move';
}

export function handleDrop(e, targetListId, newPosition) {
  e.preventDefault();
  
  if (!draggedCard) return;
  
  const cardId = draggedCard.id;
  const sourceListId = draggedFromList?.id;
  
  if (sourceListId === targetListId) {
    // Перемещение внутри того же списка
    moveCardToList(cardId, targetListId, newPosition);
  } else {
    // Перемещение в другой список
    moveCardToList(cardId, targetListId, newPosition);
  }
  
  setDraggedCard(null);
  setDraggedFromList(null);
}

export function handleDragEnd() {
  setDraggedCard(null);
  setDraggedFromList(null);
}

async function moveCardToList(cardId, listId, position) {
  try {
    await apiRequest(`/api/cards/${cardId}`, {
      method: 'PATCH',
      body: JSON.stringify({
        list_id: listId,
        position: position
      })
    });
    showToast('Карточка перемещена', 'success');
  } catch (error) {
    console.error(error);
    showToast('Ошибка при перемещении карточки', 'error');
  }
}
