// frontend/js/modules/checklists.js
// === Checklists Management ===

import { apiRequest } from './api.js';
import { showToast } from './toast.js';
import { escapeHtml } from './utils.js';

export async function createChecklist(cardId) {
  const title = prompt('Введите название чек-листа:', 'Чек-лист');
  if (!title) return;
  
  try {
    await apiRequest(`/api/cards/${cardId}/checklists`, {
      method: 'POST',
      body: JSON.stringify({ title })
    });
    
    showToast('Чек-лист создан', 'success');
    if (window.currentCardId) {
      window.openCard(window.currentCardId);
    }
  } catch (error) {
    console.error(error);
    showToast('Ошибка создания чек-листа', 'error');
  }
}

export async function deleteChecklist(cardId, checklistId) {
  if (!confirm('Удалить этот чек-лист?')) return;
  
  try {
    await apiRequest(`/api/cards/${cardId}/checklists/${checklistId}`, { method: 'DELETE' });
    showToast('Чек-лист удалён', 'success');
    if (window.currentCardId) {
      window.openCard(window.currentCardId);
    }
  } catch (error) {
    console.error(error);
    showToast('Ошибка удаления чек-листа', 'error');
  }
}

export async function addChecklistItem(cardId, checklistId) {
  const title = prompt('Введите название элемента:');
  if (!title) return;
  
  try {
    await apiRequest(`/api/cards/${cardId}/checklists/${checklistId}/items`, {
      method: 'POST',
      body: JSON.stringify({ title })
    });
    
    showToast('Элемент добавлен', 'success');
    if (window.currentCardId) {
      window.openCard(window.currentCardId);
    }
  } catch (error) {
    console.error(error);
    showToast('Ошибка добавления элемента', 'error');
  }
}

export async function toggleChecklistItem(cardId, checklistId, itemId, done) {
  try {
    await apiRequest(`/api/cards/${cardId}/checklists/${checklistId}/items/${itemId}`, {
      method: 'PATCH',
      body: JSON.stringify({ done: !done })
    });
  } catch (error) {
    console.error(error);
    showToast('Ошибка обновления элемента', 'error');
  }
}

export async function deleteChecklistItem(cardId, checklistId, itemId) {
  if (!confirm('Удалить этот элемент?')) return;
  
  try {
    await apiRequest(`/api/cards/${cardId}/checklists/${checklistId}/items/${itemId}`, { method: 'DELETE' });
    showToast('Элемент удалён', 'success');
    if (window.currentCardId) {
      window.openCard(window.currentCardId);
    }
  } catch (error) {
    console.error(error);
    showToast('Ошибка удаления элемента', 'error');
  }
}

export function renderChecklists(checklists) {
  if (!checklists || checklists.length === 0) return '<p class="empty">Нет чек-листов</p>';
  
  return checklists.map(c => {
    const total = c.items?.length || 0;
    const completed = c.items?.filter(i => i.done).length || 0;
    const progress = total > 0 ? Math.round((completed / total) * 100) : 0;
    
    return `
      <div class="checklist-item">
        <div class="checklist-header">
          <h4>${escapeHtml(c.title)}</h4>
          <button class="btn btn-sm" onclick="window.addChecklistItem(${c.card_id}, ${c.id})">+ Элемент</button>
          <button class="btn btn-sm btn-danger" onclick="window.deleteChecklist(${c.card_id}, ${c.id})">🗑️</button>
        </div>
        <progress value="${completed}" max="${total}">${progress}%</progress>
        <div class="checklist-progress">${progress}% завершено</div>
        <ul class="checklist-items">
          ${c.items?.map(i => `
            <li class="checklist-item-row ${i.done ? 'done' : ''}">
              <input type="checkbox" ${i.done ? 'checked' : ''} onchange="window.toggleChecklistItem(${c.card_id}, ${c.id}, ${i.id}, ${i.done})">
              <span>${escapeHtml(i.title)}</span>
              <button class="btn btn-sm btn-danger" onclick="window.deleteChecklistItem(${c.card_id}, ${c.id}, ${i.id})">🗑️</button>
            </li>
          `).join('') || ''}
        </ul>
      </div>
    `;
  }).join('');
}
