// frontend/js/modules/export.js
// === Export Functions ===

import { apiRequest } from './api.js';
import { showToast } from './toast.js';

export async function exportBoardToJson(boardId) {
  try {
    const data = await apiRequest(`/api/boards/${boardId}/export/json`);
    downloadFile(JSON.stringify(data, null, 2), `board-${boardId}.json`, 'application/json');
    showToast('Доска экспортирована в JSON', 'success');
  } catch (error) {
    console.error(error);
    showToast('Ошибка экспорта', 'error');
  }
}

export async function exportBoardToCsv(boardId) {
  try {
    const csv = await apiRequest(`/api/boards/${boardId}/export/csv`);
    downloadFile(csv, `board-${boardId}.csv`, 'text/csv');
    showToast('Доска экспортирована в CSV', 'success');
  } catch (error) {
    console.error(error);
    showToast('Ошибка экспорта', 'error');
  }
}

export async function getBoardStats(boardId) {
  try {
    const stats = await apiRequest(`/api/boards/${boardId}/stats`);
    return stats;
  } catch (error) {
    console.error(error);
    return null;
  }
}

export function closeBoardStats() {
  const modal = document.getElementById('stats-modal');
  if (modal) modal.style.display = 'none';
}

function downloadFile(content, filename, mimeType) {
  const blob = new Blob([content], { type: mimeType });
  const url = URL.createObjectURL(blob);
  const a = document.createElement('a');
  a.href = url;
  a.download = filename;
  document.body.appendChild(a);
  a.click();
  document.body.removeChild(a);
  URL.revokeObjectURL(url);
}
