// frontend/js/modules/bulk-ops.js
// === Массовые операции ===

import { selectedCards, isBulkMode, clearCardSelection, toggleBulkMode, updateBulkModeUI } from './state.js';
import { apiRequest } from './api.js';
import { showToast } from './toast.js';

export function toggleBulkModeFromModule() {
  toggleBulkMode();
}

export function toggleCardSelectionFromModule(cardId) {
  if (selectedCards.has(cardId)) {
    selectedCards.delete(cardId);
  } else {
    selectedCards.add(cardId);
  }
  updateBulkModeUI();
}

export async function bulkMoveCards(targetListId) {
  if (selectedCards.size === 0) {
    showToast('Выберите карточки для перемещения', 'error');
    return;
  }

  try {
    const response = await apiRequest('/api/cards/bulk/move', {
      method: 'POST',
      body: JSON.stringify({
        card_ids: Array.from(selectedCards),
        list_id: targetListId
      })
    });

    if (response.success) {
      showToast(`Перемещено ${response.processed_count} карточек`, 'success');
      clearCardSelection();
      toggleBulkMode();
      window.location.reload(); // Перезагружаем для обновления
    } else {
      showToast(`Перемещено: ${response.processed_count}, ошибок: ${response.failed_count}`, 'warning');
    }
  } catch (error) {
    console.error(error);
    showToast('Ошибка массового перемещения', 'error');
  }
}

export async function bulkUpdateCards(updates) {
  if (selectedCards.size === 0) {
    showToast('Выберите карточки для обновления', 'error');
    return;
  }

  try {
    const response = await apiRequest('/api/cards/bulk/update', {
      method: 'POST',
      body: JSON.stringify({
        card_ids: Array.from(selectedCards),
        updates
      })
    });

    if (response.success) {
      showToast(`Обновлено ${response.processed_count} карточек`, 'success');
      clearCardSelection();
      toggleBulkMode();
      window.location.reload();
    } else {
      showToast(`Обновлено: ${response.processed_count}, ошибок: ${response.failed_count}`, 'warning');
    }
  } catch (error) {
    console.error(error);
    showToast('Ошибка массового обновления', 'error');
  }
}

export async function bulkDeleteCards() {
  if (selectedCards.size === 0) {
    showToast('Выберите карточки для удаления', 'error');
    return;
  }

  if (!confirm(`Удалить ${selectedCards.size} карточек? Это действие необратимо.`)) {
    return;
  }

  try {
    const response = await apiRequest('/api/cards/bulk/delete', {
      method: 'POST',
      body: JSON.stringify({
        card_ids: Array.from(selectedCards)
      })
    });

    if (response.success) {
      showToast(`Удалено ${response.processed_count} карточек`, 'success');
      clearCardSelection();
      toggleBulkMode();
      window.location.reload();
    } else {
      showToast(`Удалено: ${response.processed_count}, ошибок: ${response.failed_count}`, 'warning');
    }
  } catch (error) {
    console.error(error);
    showToast('Ошибка массового удаления', 'error');
  }
}

export async function bulkMarkDone() {
  if (selectedCards.size === 0) {
    showToast('Выберите карточки для отметки', 'error');
    return;
  }

  await bulkUpdateCards({ done: true });
}

export async function bulkMarkTodo() {
  if (selectedCards.size === 0) {
    showToast('Выберите карточки для отметки', 'error');
    return;
  }

  await bulkUpdateCards({ done: false });
}
