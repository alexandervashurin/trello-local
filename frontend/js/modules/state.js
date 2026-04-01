// frontend/js/modules/state.js
// === State ===

export let draggedCard = null;
export let draggedFromList = null;
export let searchQuery = '';
export let currentBoardId = null;
export let currentCardId = null;
export let currentCardData = null;
export let isLoading = false;

// === Массовые операции ===
export let selectedCards = new Set();
export let isBulkMode = false;

export function setDraggedCard(card) { draggedCard = card; }
export function setDraggedFromList(list) { draggedFromList = list; }
export function setSearchQuery(query) { searchQuery = query; }
export function setCurrentBoardId(id) { currentBoardId = id; }
export function setCurrentCardId(id) { currentCardId = id; }
export function setCurrentCardData(data) { currentCardData = data; }
export function setIsLoading(loading) { isLoading = loading; }

export function toggleBulkMode() {
  isBulkMode = !isBulkMode;
  if (!isBulkMode) {
    selectedCards.clear();
  }
  updateBulkModeUI();
}

export function toggleCardSelection(cardId) {
  if (selectedCards.has(cardId)) {
    selectedCards.delete(cardId);
  } else {
    selectedCards.add(cardId);
  }
  updateBulkModeUI();
}

export function clearCardSelection() {
  selectedCards.clear();
  updateBulkModeUI();
}

export function updateBulkModeUI() {
  // Показываем/скрываем чекбоксы
  document.querySelectorAll('.card-checkbox').forEach(cb => {
    cb.style.display = isBulkMode ? 'inline-block' : 'none';
  });

  // Показываем панель массовых операций
  const bulkPanel = document.getElementById('bulk-actions-panel');
  if (bulkPanel) {
    bulkPanel.style.display = selectedCards.size > 0 ? 'flex' : 'none';
  }

  // Обновляем счётчик выбранных
  const countEl = document.getElementById('selected-count');
  if (countEl) {
    countEl.textContent = selectedCards.size;
  }
}
