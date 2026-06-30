// frontend/js/modules/boards.js
// === Boards, Lists, Cards Management ===

import { apiRequest } from './api.js';
import { showToast } from './toast.js';
import { escapeHtml, formatDateTime, getDueDateClass, getDueDateText, getInitials } from './utils.js';
import { setCurrentBoardId, setCurrentCardId, setCurrentCardData } from './state.js';

export async function loadBoards() {
  try {
    const boards = await apiRequest('/api/boards');
    renderBoards(boards);
    return boards;
  } catch (error) {
    console.error(error);
    showToast('Ошибка загрузки досок', 'error');
    return [];
  }
}

export function renderBoards(boards) {
  const container = document.getElementById('boards');
  if (!container) return;
  
  if (boards.length === 0) {
    container.innerHTML = `
      <div class="empty-state">
        <p>Нет досок</p>
        <button class="btn btn-primary" onclick="window.createBoard()">Создать первую доску</button>
      </div>
    `;
    return;
  }
  
  container.innerHTML = boards.map(board => `
    <div class="board-card" data-board-id="${board.id}">
      <div class="board-header">
        <h3>${escapeHtml(board.title)}</h3>
        <div class="board-actions">
          <button class="btn btn-sm" onclick="window.openBoard(${board.id})">Открыть</button>
          <button class="btn btn-sm btn-danger" onclick="window.deleteBoard(${board.id})">Удалить</button>
        </div>
      </div>
      <div class="board-meta">
        <span class="badge">${board.visibility === 'public' ? 'Публичная' : 'Приватная'}</span>
        ${board.is_shared ? '<span class="badge badge-info">Общая</span>' : ''}
      </div>
    </div>
  `).join('');
}

export async function createBoard() {
  const title = prompt('Введите название доски:');
  if (!title) return;
  
  try {
    await apiRequest('/api/boards', {
      method: 'POST',
      body: JSON.stringify({ title })
    });
    
    showToast('Доска создана', 'success');
    loadBoards();
  } catch (error) {
    console.error(error);
    showToast('Ошибка создания доски', 'error');
  }
}

export async function openBoard(boardId) {
  setCurrentBoardId(boardId);
  window.location.href = `/?board=${boardId}`;
}

export async function deleteBoard(boardId) {
  if (!confirm('Удалить эту доску? Это действие необратимо.')) return;
  
  try {
    await apiRequest(`/api/boards/${boardId}`, { method: 'DELETE' });
    showToast('Доска удалена', 'success');
    loadBoards();
  } catch (error) {
    console.error(error);
    showToast('Ошибка удаления доски', 'error');
  }
}

export async function loadBoardDetails(boardId) {
  try {
    const board = await apiRequest(`/api/boards/${boardId}`);
    return board;
  } catch (error) {
    console.error(error);
    return null;
  }
}

export async function loadBoardLists(boardId) {
  try {
    const lists = await apiRequest(`/api/boards/${boardId}/lists`);
    return lists;
  } catch (error) {
    console.error(error);
    return [];
  }
}

export async function createList(boardId, title) {
  try {
    await apiRequest(`/api/boards/${boardId}/lists`, {
      method: 'POST',
      body: JSON.stringify({ title })
    });
    showToast('Список создан', 'success');
  } catch (error) {
    console.error(error);
    showToast('Ошибка создания списка', 'error');
  }
}

export async function deleteList(listId) {
  if (!confirm('Удалить этот список и все карточки в нём?')) return;
  
  try {
    await apiRequest(`/api/lists/${listId}`, { method: 'DELETE' });
    showToast('Список удалён', 'success');
  } catch (error) {
    console.error(error);
    showToast('Ошибка удаления списка', 'error');
  }
}

export async function createCard(listId, title) {
  try {
    await apiRequest(`/api/lists/${listId}/cards`, {
      method: 'POST',
      body: JSON.stringify({ title })
    });
    showToast('Карточка создана', 'success');
  } catch (error) {
    console.error(error);
    showToast('Ошибка создания карточки', 'error');
  }
}

export async function openCard(cardId) {
  try {
    const card = await apiRequest(`/api/cards/${cardId}`);
    setCurrentCardId(cardId);
    setCurrentCardData(card);
    showCardModal(card);
  } catch (error) {
    console.error(error);
    showToast('Ошибка загрузки карточки', 'error');
  }
}

export function showCardModal(card) {
  const modal = document.getElementById('card-modal');
  const content = document.getElementById('card-modal-content');
  
  if (!modal || !content) return;
  
  content.innerHTML = `
    <div class="modal-header">
      <h2>📝 ${escapeHtml(card.title)}</h2>
      <button class="modal-close" onclick="window.closeCardModal()">&times;</button>
    </div>
    <div class="modal-body">
      <div class="card-section">
        <label>📝 Описание</label>
        <textarea id="card-content" rows="4">${escapeHtml(card.content || '')}</textarea>
      </div>
      
      <div class="card-section">
        <label>⏰ Срок выполнения</label>
        <input type="datetime-local" id="card-due-date" value="${card.due_date ? new Date(card.due_date * 1000).toISOString().slice(0, 16) : ''}">
      </div>
      
      <div class="card-section">
        <label>✅ Статус</label>
        <label><input type="checkbox" id="card-done" ${card.done ? 'checked' : ''}> Выполнено</label>
      </div>
      
      <div class="card-section">
        <label>👥 Исполнители</label>
        <div id="card-assignees">${renderAssignees(card.assignees || [])}</div>
      </div>
      
      <div class="card-section">
        <label>🏷️ Метки</label>
        <div id="card-labels">${renderLabels(card.labels || [])}</div>
      </div>
      
      <div class="card-section">
        <label>📋 Чек-листы</label>
        <div id="card-checklists">${renderChecklists(card.checklists || [])}</div>
      </div>
      
      <div class="card-section">
        <label>💬 Комментарии</label>
        <div id="card-comments">${renderComments(card.comments || [])}</div>
      </div>
    </div>
    <div class="modal-footer">
      <button class="btn btn-primary" onclick="window.saveCardFromModal()">💾 Сохранить</button>
      <button class="btn btn-secondary" onclick="window.closeCardModal()">Закрыть</button>
    </div>
  `;
  
  modal.classList.add('open');
}

function renderAssignees(assignees) {
  if (!assignees.length) return '<p class="empty">Нет исполнителей</p>';
  return assignees.map(a => `
    <span class="avatar" style="background:${a.avatar_color || '#0079bf'}" title="${escapeHtml(a.username)}">
      ${getInitials(a.username)}
    </span>
  `).join('');
}

function renderLabels(labels) {
  if (!labels.length) return '<p class="empty">Нет меток</p>';
  return labels.map(l => `
    <span class="label" style="background:${l.color}">${escapeHtml(l.name)}</span>
  `).join('');
}

function renderChecklists(checklists) {
  if (!checklists.length) return '<p class="empty">Нет чек-листов</p>';
  return checklists.map(c => `
    <div class="checklist">
      <h4>${escapeHtml(c.title)}</h4>
      <progress value="${c.completed || 0}" max="${c.total || 0}"></progress>
    </div>
  `).join('');
}

function renderComments(comments) {
  if (!comments.length) return '<p class="empty">Нет комментариев</p>';
  return comments.map(c => `
    <div class="comment">
      <strong>${escapeHtml(c.username)}</strong>
      <small>${formatDateTime(c.created_at)}</small>
      <p>${escapeHtml(c.content)}</p>
    </div>
  `).join('');
}

export async function saveCardFromModal() {
  const cardId = window.currentCardId;
  if (!cardId) return;
  
  const content = document.getElementById('card-content')?.value;
  const dueDate = document.getElementById('card-due-date')?.value;
  const done = document.getElementById('card-done')?.checked;
  
  try {
    await apiRequest(`/api/cards/${cardId}`, {
      method: 'PATCH',
      body: JSON.stringify({
        content,
        due_date: dueDate ? new Date(dueDate).getTime() / 1000 : null,
        done
      })
    });
    
    showToast('Карточка сохранена', 'success');
    window.closeCardModal();
  } catch (error) {
    console.error(error);
    showToast('Ошибка сохранения', 'error');
  }
}

export async function closeCardModal() {
  const modal = document.getElementById('card-modal');
  if (modal) {
    modal.classList.remove('open');
  }
  setCurrentCardId(null);
  setCurrentCardData(null);
}
