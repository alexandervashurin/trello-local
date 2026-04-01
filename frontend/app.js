// frontend/app.js

// === State ===
let draggedCard = null;
let draggedFromList = null;
let searchQuery = '';
let currentBoardId = null;
let currentCardId = null;
let currentCardData = null;
let isLoading = false;

// === Массовые операции ===
let selectedCards = new Set();
let isBulkMode = false;

function toggleBulkMode() {
  isBulkMode = !isBulkMode;
  if (!isBulkMode) {
    selectedCards.clear();
  }
  updateBulkModeUI();
}

function toggleCardSelection(cardId) {
  if (selectedCards.has(cardId)) {
    selectedCards.delete(cardId);
  } else {
    selectedCards.add(cardId);
  }
  updateBulkModeUI();
}

function clearCardSelection() {
  selectedCards.clear();
  updateBulkModeUI();
}

function updateBulkModeUI() {
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

// === Тема (Тёмная/Светлая) ===
// Загрузка сохранённой темы при старте
(function initTheme() {
  const savedTheme = localStorage.getItem('theme') || 'light';
  document.documentElement.setAttribute('data-theme', savedTheme);
  updateThemeButton(savedTheme);
})();

function toggleTheme() {
  const currentTheme = document.documentElement.getAttribute('data-theme');
  const newTheme = currentTheme === 'dark' ? 'light' : 'dark';
  document.documentElement.setAttribute('data-theme', newTheme);
  localStorage.setItem('theme', newTheme);
  updateThemeButton(newTheme);
}

function updateThemeButton(theme) {
  const btn = document.getElementById('theme-toggle');
  if (btn) {
    btn.textContent = theme === 'dark' ? '☀️' : '🌙';
    btn.title = theme === 'dark' ? 'Светлая тема' : 'Тёмная тема';
  }
}

// === DOM Elements ===
const boardsContainer = document.getElementById('boards');
const createBoardBtn = document.getElementById('create-board-btn');
const searchInput = document.getElementById('search-input');
const loadingIndicator = document.getElementById('loading-indicator');
const toastContainer = document.getElementById('toast-container');

// === Event Listeners ===
if (createBoardBtn) {
  createBoardBtn.addEventListener('click', createBoard);
}

if (searchInput) {
  searchInput.addEventListener('input', (e) => {
    searchQuery = e.target.value.trim();
    loadBoards();
  });
}

// === Toast Notifications ===
function showToast(message, type = 'info', duration = 3000) {
  const toast = document.createElement('div');
  toast.className = `toast ${type}`;

  const icon = document.createElement('span');
  icon.textContent = type === 'success' ? '✓' : type === 'error' ? '✕' : 'ℹ';
  
  const text = document.createElement('span');
  text.textContent = message;
  
  toast.appendChild(icon);
  toast.appendChild(text);

  toastContainer.appendChild(toast);

  setTimeout(() => {
    toast.style.animation = 'slideInRight 0.3s ease reverse';
    setTimeout(() => toast.remove(), 300);
  }, duration);
}

// === Loading State ===
function setLoading(loading) {
  isLoading = loading;
  if (loadingIndicator) {
    loadingIndicator.style.display = loading ? 'flex' : 'none';
  }
}

// === API Helper ===
async function apiRequest(url, options = {}) {
  const token = localStorage.getItem('token');
  
  const defaultOptions = {
    headers: {
      'Content-Type': 'application/json',
      ...(token ? { 'Authorization': `Bearer ${token}` } : {}),
    },
  };
  
  try {
    const response = await fetch(url, { ...defaultOptions, ...options });
    
    if (response.status === 401) {
      localStorage.removeItem('token');
      localStorage.removeItem('user');
      window.location.href = '/login.html';
      return null;
    }
    
    const data = await response.json();
    
    if (!response.ok) {
      throw new Error(data.error || 'Ошибка запроса');
    }
    
    return data;
  } catch (error) {
    console.error('API Error:', error);
    throw error;
  }
}

// === Load Boards ===
async function loadBoards() {
  setLoading(true);
  
  try {
    const url = searchQuery
      ? `/api/boards?search=${encodeURIComponent(searchQuery)}`
      : '/api/boards';

    const boards = await apiRequest(url);
    
    if (!boards || boards.length === 0) {
      boardsContainer.innerHTML = `
        <div class="empty-state">
          <div class="empty-state-icon">📋</div>
          <h3>Нет досок</h3>
          <p>Создайте первую доску, нажав кнопку "Новая доска"</p>
        </div>
      `;
      setLoading(false);
      return;
    }

    boardsContainer.innerHTML = boards.map(board => `
      <div class="board" data-board-id="${board.id}">
        <div class="board-header">
          <div style="display:flex; align-items:center; gap:10px; flex-wrap:wrap;">
            <h3>${escapeHtml(board.title)}</h3>
            ${board.is_shared ? '<span class="board-badge">🌐 Общая</span>' : ''}
            ${board.visibility === 'public' ? '<span class="board-badge" style="background:#22c55e;">🌍 Публичная</span>' : ''}
          </div>
          <div class="board-actions">
            <span class="members-count" title="Участники">👥 ${board.members.length}</span>
            <button class="btn btn-secondary" onclick="openBoardStats(${board.id})" title="Статистика" style="padding:4px 8px;">📊</button>
            <button class="btn btn-secondary" onclick="exportBoardJSON(${board.id})" title="Экспорт JSON" style="padding:4px 8px;">📥 JSON</button>
            <button class="btn btn-secondary" onclick="exportBoardCSV(${board.id})" title="Экспорт CSV" style="padding:4px 8px;">📥 CSV</button>
            <button class="btn btn-secondary" onclick="openMembersModal(${board.id})" title="Участники">👤</button>
            <button class="btn btn-secondary" onclick="openInvitationsModal(${board.id})" title="Приглашения">🔗</button>
            <button class="btn btn-secondary" onclick="deleteBoard(${board.id})" title="Удалить доску">🗑️</button>
          </div>
        </div>
        <div class="board-search-bar" data-board-id="${board.id}">
          <input type="text" class="board-search-input" placeholder="🔍 Поиск карточек..." oninput="handleBoardSearch(${board.id}, this.value)" style="padding:6px 10px; border:1px solid #dfe1e6; border-radius:4px; font-size:13px; width:250px;">
          <button class="btn btn-secondary btn-sm" onclick="openLabelFilter(${board.id})" title="Фильтр по меткам" style="padding:4px 8px; margin-left:8px;">🏷️</button>
          <button class="btn btn-secondary btn-sm" onclick="clearBoardSearch(${board.id})" title="Очистить поиск" style="padding:4px 8px;">✕</button>
        </div>
        <div class="lists-container">
          ${board.lists.map(list => `
            <div class="list" data-list-id="${list.id}" data-list-index="${board.lists.indexOf(list)}">
              <div class="list-header">
                <h4>${escapeHtml(list.title)}</h4>
                <button class="btn btn-secondary" onclick="deleteList(${list.id})" style="padding:2px 6px;font-size:10px;" title="Удалить список">🗑️</button>
              </div>
              <div class="cards" data-list-id="${list.id}">
                ${list.cards.map(card => `
                  <div class="card ${selectedCards.has(card.id) ? 'card-selected' : ''}"
                       draggable="true"
                       data-card-id="${card.id}"
                       data-list-id="${list.id}"
                       ondblclick="openCardModal(${card.id}, ${board.id})">
                    <div style="display:flex; justify-content:space-between; align-items:start;">
                      <div style="display:flex; gap:8px; align-items:flex-start;">
                        <input type="checkbox" class="card-checkbox" ${selectedCards.has(card.id) ? 'checked' : ''} onchange="toggleCardSelection(${card.id}); event.stopPropagation();" style="margin-top:3px;display:none;" title="Выделить для массовых операций">
                        <label style="display:flex; align-items:flex-start; gap:8px; cursor:pointer;">
                          <input type="checkbox" ${card.done ? 'checked' : ''} onchange="toggleCardDone(${card.id}, this.checked)" style="margin-top:3px;">
                          <span style="width:100%;">
                          <div style="display:flex; flex-wrap:wrap; gap:4px; margin-bottom:4px;">
                            ${(card.labels || []).map(l => `<span class="label-badge label-${l.color}" title="${escapeHtml(l.name)}">${escapeHtml(l.name)}</span>`).join('')}
                          </div>
                          <div style="display:flex; align-items:center; gap:8px; flex-wrap:wrap;">
                            <strong class="${card.done ? 'done' : ''}">${escapeHtml(card.title)}</strong>
                            ${card.due_date ? `<span class="due-date-badge ${isOverdue(card.due_date) ? 'overdue' : ''}" title="Дедлайн">${formatDueDate(card.due_date)}</span>` : ''}
                          </div>
                          ${card.content ? `<p>${escapeHtml(card.content)}</p>` : ''}
                          ${(card.attachments || []).length > 0 ? `<div style="margin-top:4px;font-size:11px;color:var(--text-secondary);">📎 ${card.attachments.length} влож.</div>` : ''}
                        </span>
                      </label>
                      </div>
                      <div class="card-actions">
                        <button class="btn btn-secondary" onclick="openCardModal(${card.id}, ${board.id})" title="Открыть карточку" style="padding:2px 6px;font-size:10px;">✏️</button>
                        <button class="btn btn-secondary" onclick="openCommentsModal(${card.id})" title="Комментарии" style="padding:2px 6px;font-size:10px;">💬</button>
                        <button class="btn btn-secondary" onclick="deleteCard(${card.id})" style="padding:2px 6px;font-size:10px;" title="Удалить">🗑️</button>
                      </div>
                    </div>
                  </div>
                `).join('')}
                <div class="add-card-placeholder" data-list-id="${list.id}">
                  <button class="btn" onclick="showAddCardForm(${list.id})">➕ Добавить карточку</button>
                </div>
              </div>
            </div>
          `).join('')}
          <div class="add-list-placeholder" data-board-id="${board.id}">
            <button class="btn" onclick="showAddListForm(${board.id})">➕ Добавить список</button>
          </div>
        </div>
      </div>
    `).join('');

    // Назначаем drag-обработчики
    document.querySelectorAll('.card').forEach(card => {
      card.addEventListener('dragstart', handleDragStart);
      card.addEventListener('dragend', handleDragEnd);
    });

    document.querySelectorAll('.cards').forEach(cardsContainer => {
      cardsContainer.addEventListener('dragover', handleDragOver);
      cardsContainer.addEventListener('dragenter', handleDragEnter);
      cardsContainer.addEventListener('dragleave', handleDragLeave);
      cardsContainer.addEventListener('drop', handleDrop);
    });
    
  } catch (error) {
    console.error(error);
    boardsContainer.innerHTML = '<div class="empty-state"><h3>❌ Ошибка загрузки данных</h3><p>Проверьте подключение к серверу</p></div>';
    showToast('Не удалось загрузить доски', 'error');
  } finally {
    setLoading(false);
  }
}

// === Utility Functions ===
function escapeHtml(text) {
  if (!text) return '';
  const div = document.createElement('div');
  div.textContent = text;
  return div.innerHTML;
}

function escapeJs(str) {
  return String(str).replace(/'/g, "\\'").replace(/\\/g, '\\\\');
}

// === Board Actions ===
async function createBoard() {
  const title = prompt('Название новой доски:');
  if (!title || title.trim() === '') return;

  const isShared = confirm('Нажмите OK, чтобы сделать доску общей (доступной другим пользователям), или Отмена для личной доски.');

  try {
    await apiRequest('/api/boards', {
      method: 'POST',
      body: JSON.stringify({ title: title.trim(), is_shared: isShared })
    });
    
    showToast('Доска создана', 'success');
    loadBoards();
  } catch (error) {
    console.error(error);
    showToast('Не удалось создать доску', 'error');
  }
}

async function deleteBoard(boardId) {
  if (!confirm('Удалить доску? Все списки и карточки будут удалены.')) return;
  
  try {
    await apiRequest(`/api/boards/${boardId}`, { method: 'DELETE' });
    showToast('Доска удалена', 'success');
    loadBoards();
  } catch (error) {
    console.error(error);
    showToast('Не удалось удалить доску', 'error');
  }
}

// === List Actions ===
function showAddListForm(boardId) {
  const placeholder = document.querySelector(`.add-list-placeholder[data-board-id="${boardId}"]`);
  if (!placeholder) return;
  
  placeholder.innerHTML = `
    <div class="add-list-form">
      <input type="text" class="add-list-input" placeholder="Название списка" maxlength="50" autofocus>
      <div class="add-list-btns">
        <button class="btn btn-primary" onclick="createList(${boardId}, this)">Добавить</button>
        <button class="btn btn-secondary" onclick="cancelAddList(${boardId})">Отмена</button>
      </div>
    </div>
  `;
  
  // Focus on input
  const input = placeholder.querySelector('.add-list-input');
  if (input) {
    input.focus();
    input.addEventListener('keypress', (e) => {
      if (e.key === 'Enter') {
        createList(boardId, placeholder.querySelector('.btn-primary'));
      } else if (e.key === 'Escape') {
        cancelAddList(boardId);
      }
    });
  }
}

async function createList(boardId, button) {
  const form = button.closest('.add-list-form');
  const input = form.querySelector('.add-list-input');
  const title = input.value.trim();
  
  if (!title) {
    input.focus();
    return;
  }
  
  try {
    await apiRequest(`/api/boards/${boardId}/lists`, {
      method: 'POST',
      body: JSON.stringify({ title })
    });
    
    showToast('Список создан', 'success');
    loadBoards();
  } catch (error) {
    console.error(error);
    showToast('Не удалось создать список', 'error');
  }
}

async function deleteList(listId) {
  if (!confirm('Удалить список? Все карточки будут удалены.')) return;
  
  try {
    await apiRequest(`/api/lists/${listId}`, { method: 'DELETE' });
    showToast('Список удалён', 'success');
    loadBoards();
  } catch (error) {
    console.error(error);
    showToast('Не удалось удалить список', 'error');
  }
}

function cancelAddList(boardId) {
  const placeholder = document.querySelector(`.add-list-placeholder[data-board-id="${boardId}"]`);
  if (placeholder) {
    placeholder.innerHTML = '<button class="btn" onclick="showAddListForm(' + boardId + ')">➕ Добавить список</button>';
  }
}

// === Card Actions ===
function showAddCardForm(listId) {
  const placeholder = document.querySelector(`.add-card-placeholder[data-list-id="${listId}"]`);
  if (!placeholder) return;

  placeholder.innerHTML = `
    <div class="add-card-form">
      <input type="text" class="add-list-input" placeholder="Название карточки" maxlength="100" autofocus>
      <textarea class="add-list-input" placeholder="Описание (необязательно)" rows="2" style="margin-top:5px;resize:vertical;"></textarea>
      <input type="datetime-local" class="add-list-input" placeholder="Дедлайн (необязательно)" style="margin-top:5px;">
      <div class="add-list-btns">
        <button class="btn btn-primary" onclick="createCard(${listId}, this)">Добавить</button>
        <button class="btn btn-secondary" onclick="cancelAddCard(${listId})">Отмена</button>
      </div>
    </div>
  `;

  // Focus on title input
  const input = placeholder.querySelector('input');
  if (input) {
    input.focus();
    input.addEventListener('keypress', (e) => {
      if (e.key === 'Enter' && !e.shiftKey) {
        e.preventDefault();
        createCard(listId, placeholder.querySelector('.btn-primary'));
      } else if (e.key === 'Escape') {
        cancelAddCard(listId);
      }
    });
  }
}

async function createCard(listId, button) {
  const form = button.closest('.add-card-form');
  const titleInput = form.querySelector('input');
  const contentInput = form.querySelector('textarea');
  const dueDateInput = form.querySelector('input[type="datetime-local"]');
  const title = titleInput.value.trim();
  const content = contentInput.value.trim() || null;
  const due_date = dueDateInput && dueDateInput.value ? Math.floor(new Date(dueDateInput.value).getTime() / 1000) : null;

  if (!title) {
    titleInput.focus();
    return;
  }

  try {
    await apiRequest(`/api/lists/${listId}/cards`, {
      method: 'POST',
      body: JSON.stringify({ title, content, due_date })
    });

    showToast('Карточка создана', 'success');
    loadBoards();
  } catch (error) {
    console.error(error);
    showToast('Не удалось создать карточку', 'error');
  }
}

async function deleteCard(cardId) {
  if (!confirm('Удалить карточку?')) return;
  
  try {
    await apiRequest(`/api/cards/${cardId}`, { method: 'DELETE' });
    showToast('Карточка удалена', 'success');
    loadBoards();
  } catch (error) {
    console.error(error);
    showToast('Не удалось удалить карточку', 'error');
  }
}

function cancelAddCard(listId) {
  const placeholder = document.querySelector(`.add-card-placeholder[data-list-id="${listId}"]`);
  if (placeholder) {
    placeholder.innerHTML = '<button class="btn" onclick="showAddCardForm(' + listId + ')">➕ Добавить карточку</button>';
  }
}

// === Edit Card ===
function editCard(cardId, boardId, listId) {
  const cardEl = document.querySelector(`.card[data-card-id="${cardId}"]`);
  if (!cardEl) return;
  
  const checkbox = cardEl.querySelector('input[type="checkbox"]');
  const strong = cardEl.querySelector('strong');
  const contentP = cardEl.querySelector('p');
  
  const title = strong.textContent;
  const content = contentP ? contentP.textContent : '';
  
  const originalHTML = cardEl.innerHTML;
  const formDiv = document.createElement('div');
  formDiv.className = 'add-card-form';

  const input = document.createElement('input');
  input.type = 'text';
  input.className = 'add-list-input';
  input.value = title;
  input.maxLength = 100;

  const textarea = document.createElement('textarea');
  textarea.className = 'add-list-input';
  textarea.rows = 2;
  textarea.style.marginTop = '5px';
  textarea.style.resize = 'vertical';
  textarea.value = content;

  const btnsDiv = document.createElement('div');
  btnsDiv.className = 'add-list-btns';

  const saveBtn = document.createElement('button');
  saveBtn.className = 'btn btn-primary';
  saveBtn.textContent = 'Сохранить';
  saveBtn.onclick = () => saveCardEdit(cardId, saveBtn);

  const cancelBtn = document.createElement('button');
  cancelBtn.className = 'btn btn-secondary';
  cancelBtn.textContent = 'Отмена';
  cancelBtn.onclick = () => cancelCardEdit(cardId, originalHTML);

  btnsDiv.appendChild(saveBtn);
  btnsDiv.appendChild(cancelBtn);

  formDiv.appendChild(input);
  formDiv.appendChild(textarea);
  formDiv.appendChild(btnsDiv);

  cardEl.innerHTML = '';
  cardEl.appendChild(formDiv);
  
  input.focus();
  input.select();
  
  input.addEventListener('keypress', (e) => {
    if (e.key === 'Enter' && !e.shiftKey) {
      e.preventDefault();
      saveCardEdit(cardId, saveBtn);
    } else if (e.key === 'Escape') {
      cancelCardEdit(cardId, originalHTML);
    }
  });
}

async function saveCardEdit(cardId, button) {
  const form = button.closest('.add-card-form');
  const titleInput = form.querySelector('input');
  const contentInput = form.querySelector('textarea');
  const title = titleInput.value.trim();
  const content = contentInput.value.trim() || null;
  
  if (!title) {
    titleInput.focus();
    return;
  }
  
  try {
    await apiRequest(`/api/cards/${cardId}`, {
      method: 'PATCH',
      body: JSON.stringify({ title, content })
    });
    
    showToast('Карточка обновлена', 'success');
    loadBoards();
  } catch (error) {
    console.error(error);
    showToast('Не удалось сохранить карточку', 'error');
  }
}

function cancelCardEdit(cardId, originalHTML) {
  const cardEl = document.querySelector(`.card[data-card-id="${cardId}"]`);
  if (cardEl) cardEl.innerHTML = originalHTML;
}

// === Toggle Done ===
async function toggleCardDone(cardId, done) {
  const checkbox = document.querySelector(`.card[data-card-id="${cardId}"] input[type="checkbox"]`);
  
  try {
    await apiRequest(`/api/cards/${cardId}`, {
      method: 'PATCH',
      body: JSON.stringify({ done })
    });
    showToast(done ? 'Отмечено выполненным' : 'Возвращено в работу', 'success', 1500);
    loadBoards();
  } catch (error) {
    console.error(error);
    showToast('Не удалось обновить статус', 'error');
    if (checkbox) checkbox.checked = !done;
  }
}

// === Drag-and-Drop ===
function handleDragStart(e) {
  draggedCard = this;
  draggedFromList = this.dataset.listId;
  this.classList.add('dragging');
  e.dataTransfer.effectAllowed = 'move';
  e.dataTransfer.setData('text/plain', this.dataset.cardId);
}

function handleDragEnd() {
  this.classList.remove('dragging');
  document.querySelectorAll('.cards').forEach(container => {
    container.classList.remove('drag-over');
  });
  draggedCard = null;
  draggedFromList = null;
}

function handleDragOver(e) {
  e.preventDefault();
  e.dataTransfer.dropEffect = 'move';
}

function handleDragEnter(e) {
  e.preventDefault();
  if (draggedCard && this !== draggedCard.closest('.cards')) {
    this.classList.add('drag-over');
  }
}

function handleDragLeave() {
  this.classList.remove('drag-over');
}

async function handleDrop(e) {
  e.preventDefault();
  this.classList.remove('drag-over');
  
  const targetListId = this.dataset.listId;
  const cardId = draggedCard.dataset.cardId;
  
  if (targetListId === draggedFromList) return;
  
  try {
    await apiRequest(`/api/cards/${cardId}`, {
      method: 'PATCH',
      body: JSON.stringify({ list_id: parseInt(targetListId) })
    });
    
    showToast('Карточка перемещена', 'success', 1500);
    loadBoards();
  } catch (error) {
    console.error(error);
    showToast('Ошибка перемещения карточки', 'error');
  }
}

// === Members Management ===
async function openMembersModal(boardId) {
  currentBoardId = boardId;
  const modal = document.getElementById('members-modal');
  const membersList = document.getElementById('members-list');

  modal.classList.add('open');
  membersList.innerHTML = '<div class="loading">Загрузка...</div>';

  try {
    const members = await apiRequest(`/api/boards/${boardId}/members`);

    if (!members || members.length === 0) {
      membersList.innerHTML = '<div class="empty-state"><p>Нет участников</p></div>';
    } else {
      membersList.innerHTML = members.map(m => `
        <div class="member-item">
          <span><strong>${escapeHtml(m.username)}</strong> <span class="role-badge">(${getRoleName(m.role)})</span></span>
          <select onchange="changeMemberRole(${m.user_id}, this.value)" style="margin-left:8px;font-size:11px;">
            <option value="viewer" ${m.role === 'viewer' ? 'selected' : ''}>👁️</option>
            <option value="member" ${m.role === 'member' ? 'selected' : ''}>✏️</option>
            <option value="editor" ${m.role === 'editor' ? 'selected' : ''}>📝</option>
            <option value="admin" ${m.role === 'admin' ? 'selected' : ''}>⭐</option>
          </select>
          <button class="btn btn-secondary" onclick="removeMember(${m.user_id})" style="padding:4px 10px;font-size:11px;margin-left:8px;">Удалить</button>
        </div>
      `).join('');
    }

    // Focus on input
    setTimeout(() => {
      document.getElementById('new-member-username').focus();
    }, 100);
  } catch (error) {
    console.error(error);
    membersList.innerHTML = '<div class="empty-state"><p>Ошибка загрузки участников</p></div>';
    showToast('Не удалось загрузить участников', 'error');
  }
}

function closeMembersModal() {
  const modal = document.getElementById('members-modal');
  modal.classList.remove('open');
  currentBoardId = null;
  document.getElementById('new-member-username').value = '';
}

async function addMember() {
  const usernameInput = document.getElementById('new-member-username');
  const username = usernameInput.value.trim();

  if (!username) {
    showToast('Введите имя пользователя', 'error');
    return;
  }

  try {
    // Ищем или создаём пользователя
    let users = await apiRequest(`/api/users?username=${encodeURIComponent(username)}`);
    let userId;

    if (users && users.length > 0) {
      userId = users[0].id;
    } else {
      // Создаём нового пользователя
      const user = await apiRequest('/api/users', {
        method: 'POST',
        body: JSON.stringify({ username })
      });
      userId = user.id;
    }

    // Добавляем участника на доску с выбранной ролью
    const roleSelect = document.getElementById('new-member-role');
    const role = roleSelect.value;
    
    await apiRequest(`/api/boards/${currentBoardId}/members`, {
      method: 'POST',
      body: JSON.stringify({ user_id: userId, role: role })
    });

    usernameInput.value = '';
    roleSelect.value = 'member';
    showToast('Участник добавлен', 'success');
    openMembersModal(currentBoardId);
    loadBoards();
  } catch (error) {
    console.error(error);
    showToast(error.message || 'Ошибка добавления участника', 'error');
  }
}

async function changeMemberRole(userId, newRole) {
  try {
    await apiRequest(`/api/boards/${currentBoardId}/members`, {
      method: 'POST',
      body: JSON.stringify({ user_id: userId, role: newRole })
    });

    showToast('Роль изменена', 'success');
    openMembersModal(currentBoardId);
  } catch (error) {
    console.error(error);
    showToast('Не удалось изменить роль', 'error');
  }
}

async function removeMember(userId) {
  if (!confirm('Удалить участника из доски?')) return;

  try {
    await apiRequest(`/api/boards/${currentBoardId}/members/${userId}`, {
      method: 'DELETE'
    });

    showToast('Участник удалён', 'success');
    openMembersModal(currentBoardId);
    loadBoards();
  } catch (error) {
    console.error(error);
    showToast('Не удалось удалить участника', 'error');
  }
}

function getRoleName(role) {
  const roles = {
    'owner': '👑 Владелец',
    'admin': '⭐ Админ',
    'editor': '📝 Редактор',
    'member': '✏️ Участник',
    'viewer': '👁️ Наблюдатель'
  };
  return roles[role] || role;
}

// === Invitations Management ===
async function openInvitationsModal(boardId) {
  currentBoardId = boardId;
  const modal = document.getElementById('invitations-modal');
  const invitationsList = document.getElementById('invitations-list');

  modal.classList.add('open');
  invitationsList.innerHTML = '<div class="loading">Загрузка...</div>';

  try {
    const invitations = await apiRequest(`/api/boards/${boardId}/invitations`);

    if (!invitations || invitations.length === 0) {
      invitationsList.innerHTML = '<div class="empty-state"><p>Нет активных приглашений</p></div>';
    } else {
      invitationsList.innerHTML = invitations.map(inv => `
        <div class="invitation-item">
          <div style="flex:1;">
            <div><strong>Роль:</strong> ${getRoleName(inv.role)}</div>
            <div style="font-size:11px; color:var(--text-secondary);">Ссылка: <code style="background:#f0f0f0;padding:2px 4px;border-radius:3px;">${inv.invite_link}</code></div>
            ${inv.expires_at ? `<div style="font-size:11px; color:var(--text-secondary);">Истекает: ${new Date(inv.expires_at * 1000).toLocaleString('ru-RU')}</div>` : '<div style="font-size:11px; color:var(--text-secondary);">Бессрочно</div>'}
          </div>
          <button class="btn btn-secondary" onclick="copyInviteLink('${inv.invite_link}')" style="padding:4px 10px;font-size:11px;">📋 Копировать</button>
          <button class="btn btn-secondary" onclick="deleteInvitation(${boardId}, '${inv.token}')" style="padding:4px 10px;font-size:11px;margin-left:8px;">🗑️</button>
        </div>
      `).join('');
    }
  } catch (error) {
    console.error(error);
    invitationsList.innerHTML = '<div class="empty-state"><p>Ошибка загрузки приглашений</p></div>';
    showToast('Не удалось загрузить приглашения', 'error');
  }
}

function closeInvitationsModal() {
  const modal = document.getElementById('invitations-modal');
  modal.classList.remove('open');
  currentBoardId = null;
}

async function createInvitation() {
  const roleSelect = document.getElementById('invitation-role');
  const expiresInput = document.getElementById('invitation-expires');
  const role = roleSelect.value;
  const expiresHours = parseInt(expiresInput.value) || 0;

  try {
    const invitation = await apiRequest(`/api/boards/${currentBoardId}/invitations`, {
      method: 'POST',
      body: JSON.stringify({ 
        role: role, 
        expires_in_hours: expiresHours > 0 ? expiresHours : null 
      })
    });

    showToast('Приглашение создано', 'success');
    
    // Копируем ссылку в буфер
    await navigator.clipboard.writeText(invitation.invite_link);
    showToast('Ссылка скопирована в буфер обмена', 'success');
    
    openInvitationsModal(currentBoardId);
  } catch (error) {
    console.error(error);
    showToast('Не удалось создать приглашение', 'error');
  }
}

async function copyInviteLink(link) {
  try {
    await navigator.clipboard.writeText(link);
    showToast('Ссылка скопирована в буфер обмена', 'success');
  } catch (error) {
    // Fallback для старых браузеров
    const textarea = document.createElement('textarea');
    textarea.value = link;
    document.body.appendChild(textarea);
    textarea.select();
    document.execCommand('copy');
    document.body.removeChild(textarea);
    showToast('Ссылка скопирована в буфер обмена', 'success');
  }
}

async function deleteInvitation(boardId, token) {
  if (!confirm('Отозвать приглашение?')) return;

  try {
    await apiRequest(`/api/boards/${boardId}/invitations/${token}`, {
      method: 'DELETE'
    });

    showToast('Приглашение отозвано', 'success');
    openInvitationsModal(currentBoardId);
  } catch (error) {
    console.error(error);
    showToast('Не удалось отозвать приглашение', 'error');
  }
}

// === Comments Management ===
async function openCommentsModal(cardId) {
  currentCardId = cardId;
  const modal = document.getElementById('comments-modal');
  const commentsList = document.getElementById('comments-list');

  modal.classList.add('open');
  commentsList.innerHTML = '<div class="loading">Загрузка...</div>';

  try {
    const comments = await apiRequest(`/api/cards/${cardId}/comments`);

    if (!comments || comments.length === 0) {
      commentsList.innerHTML = '<div class="empty-state"><p>Нет комментариев</p></div>';
    } else {
      const currentUser = JSON.parse(localStorage.getItem('user') || 'null');
      commentsList.innerHTML = comments.map(c => `
        <div class="comment-item">
          <div class="comment-header">
            <strong>${escapeHtml(c.username)}</strong>
            <span class="comment-date">${new Date(c.created_at * 1000).toLocaleString('ru-RU')}</span>
          </div>
          <div class="comment-content">${escapeHtml(c.content)}</div>
          ${c.user_id === currentUser?.user_id ? `
          <div class="comment-actions">
            <button class="btn btn-secondary" onclick="editComment(${c.id})" style="padding:2px 8px;font-size:11px;">✏️</button>
            <button class="btn btn-secondary" onclick="deleteComment(${c.id})" style="padding:2px 8px;font-size:11px;">🗑️</button>
          </div>` : ''}
        </div>
      `).join('');
    }

    // Focus on textarea
    setTimeout(() => {
      document.getElementById('new-comment-content').focus();
    }, 100);
  } catch (error) {
    console.error(error);
    commentsList.innerHTML = '<div class="empty-state"><p>Ошибка загрузки комментариев</p></div>';
    showToast('Не удалось загрузить комментарии', 'error');
  }
}

function closeCommentsModal() {
  const modal = document.getElementById('comments-modal');
  modal.classList.remove('open');
  currentCardId = null;
  document.getElementById('new-comment-content').value = '';
}

async function addComment() {
  const contentInput = document.getElementById('new-comment-content');
  const content = contentInput.value.trim();

  if (!content) {
    showToast('Введите текст комментария', 'error');
    return;
  }

  try {
    await apiRequest(`/api/cards/${currentCardId}/comments`, {
      method: 'POST',
      body: JSON.stringify({ content })
    });

    contentInput.value = '';
    showToast('Комментарий добавлен', 'success');
    openCommentsModal(currentCardId);
  } catch (error) {
    console.error(error);
    showToast('Не удалось добавить комментарий', 'error');
  }
}

async function deleteComment(commentId) {
  if (!confirm('Удалить комментарий?')) return;

  try {
    await apiRequest(`/api/comments/${commentId}`, { method: 'DELETE' });
    showToast('Комментарий удалён', 'success');
    openCommentsModal(currentCardId);
  } catch (error) {
    console.error(error);
    showToast('Не удалось удалить комментарий', 'error');
  }
}

async function editComment(commentId) {
  const commentEl = document.querySelector(`.comment-item:nth-child(${commentId})`);
  const contentDiv = commentEl?.querySelector('.comment-content');
  if (!contentDiv) return;

  const currentContent = contentDiv.textContent;
  const newContent = prompt('Редактировать комментарий:', currentContent);
  
  if (newContent === null || newContent.trim() === '') return;

  try {
    await apiRequest(`/api/comments/${commentId}`, {
      method: 'PATCH',
      body: JSON.stringify({ content: newContent.trim() })
    });

    showToast('Комментарий обновлён', 'success');
    openCommentsModal(currentCardId);
  } catch (error) {
    console.error(error);
    showToast('Не удалось обновить комментарий', 'error');
  }
}

// === Helper Functions for Due Date ===
function formatDueDate(timestamp) {
  const date = new Date(timestamp * 1000);
  const now = new Date();
  const diff = date - now;
  const days = Math.ceil(diff / (1000 * 60 * 60 * 24));
  
  if (days < 0) return `⏰ Просрочено: ${date.toLocaleDateString('ru-RU')}`;
  if (days === 0) return '⏰ Сегодня';
  if (days === 1) return '⏰ Завтра';
  if (days <= 7) return `⏰ Через ${days} дн.`;
  return `⏰ ${date.toLocaleDateString('ru-RU')}`;
}

function isOverdue(timestamp) {
  return timestamp * 1000 < Date.now();
}

// === Card Modal Functions ===
async function openCardModal(cardId, boardId) {
  currentCardId = cardId;
  currentBoardId = boardId;
  const modal = document.getElementById('card-modal');
  
  try {
    // Загружаем карточку (данные уже есть в boards, но обновим)
    const card = await apiRequest(`/api/cards/${cardId}`);
    currentCardData = card;
    
    // Заполняем форму
    document.getElementById('card-title').value = card.title || '';
    document.getElementById('card-content').value = card.content || '';
    
    if (card.due_date) {
      const date = new Date(card.due_date * 1000);
      document.getElementById('card-due-date').value = date.toISOString().slice(0, 16);
    } else {
      document.getElementById('card-due-date').value = '';
    }
    
    // Загружаем метки
    await loadCardLabels(cardId);
    
    // Загружаем вложения
    await loadCardAttachments(cardId);
    
    // Загружаем историю
    await loadBoardActivity(boardId);

    // Загружаем чек-листы
    await loadCardChecklists(cardId);

    // Загружаем исполнителей
    await loadCardAssignees(cardId);

    modal.classList.add('open');
  } catch (error) {
    console.error(error);
    showToast('Не удалось загрузить карточку', 'error');
  }
}

function closeCardModal() {
  const modal = document.getElementById('card-modal');
  modal.classList.remove('open');
  currentCardId = null;
  currentCardData = null;
}

async function saveCardFromModal() {
  const title = document.getElementById('card-title').value.trim();
  const content = document.getElementById('card-content').value.trim() || null;
  const dueDateInput = document.getElementById('card-due-date').value;
  const due_date = dueDateInput ? Math.floor(new Date(dueDateInput).getTime() / 1000) : null;
  
  if (!title) {
    showToast('Введите название карточки', 'error');
    return;
  }
  
  try {
    await apiRequest(`/api/cards/${currentCardId}`, {
      method: 'PATCH',
      body: JSON.stringify({ title, content, due_date })
    });
    
    showToast('Карточка сохранена', 'success');
    closeCardModal();
    loadBoards();
  } catch (error) {
    console.error(error);
    showToast('Не удалось сохранить карточку', 'error');
  }
}

function clearDueDate() {
  document.getElementById('card-due-date').value = '';
}

// === Labels Functions ===
async function loadCardLabels(cardId) {
  const container = document.getElementById('card-labels');

  try {
    const labels = await apiRequest(`/api/cards/${cardId}/labels`);

    if (!labels || labels.length === 0) {
      container.innerHTML = '<div class="empty-state" style="padding:10px;"><p>Нет меток</p></div>';
    } else {
      container.innerHTML = labels.map(l => `
        <div class="label-item label-${l.color}">
          <span>${escapeHtml(l.name)}</span>
          <div style="display:flex; gap:4px; margin-left:auto;">
            <button class="btn btn-secondary btn-sm" onclick="editLabel(${l.id}, '${escapeJs(l.name)}', '${l.color}')" style="padding:2px 6px;" title="Редактировать">✏️</button>
            <button class="btn btn-secondary btn-sm" onclick="deleteLabel(${l.id})" style="padding:2px 6px;" title="Удалить">✕</button>
          </div>
        </div>
      `).join('');
    }
  } catch (error) {
    container.innerHTML = '<div class="empty-state" style="padding:10px;"><p>Ошибка загрузки</p></div>';
  }
}

async function addLabel() {
  const nameInput = document.getElementById('new-label-name');
  const colorSelect = document.getElementById('new-label-color');
  const name = nameInput.value.trim();
  const color = colorSelect.value;
  
  if (!name) {
    showToast('Введите название метки', 'error');
    return;
  }
  
  try {
    await apiRequest(`/api/cards/${currentCardId}/labels`, {
      method: 'POST',
      body: JSON.stringify({ name, color })
    });
    
    nameInput.value = '';
    showToast('Метка добавлена', 'success');
    loadCardLabels(currentCardId);
    loadBoards();
  } catch (error) {
    console.error(error);
    showToast('Не удалось добавить метку', 'error');
  }
}

async function deleteLabel(labelId) {
  try {
    await apiRequest(`/api/cards/${currentCardId}/labels/${labelId}`, {
      method: 'DELETE'
    });

    showToast('Метка удалена', 'success');
    loadCardLabels(currentCardId);
    loadBoards();
  } catch (error) {
    console.error(error);
    showToast('Не удалось удалить метку', 'error');
  }
}

async function editLabel(labelId, currentName, currentColor) {
  const newName = prompt('Новое название метки:', currentName);
  if (newName === null || newName.trim() === '') return;

  const colors = ['blue', 'green', 'yellow', 'red', 'purple', 'orange'];
  let newColor = currentColor;
  
  const colorInput = prompt('Выберите цвет (blue, green, yellow, red, purple, orange):', currentColor);
  if (colorInput !== null && colors.includes(colorInput.trim().toLowerCase())) {
    newColor = colorInput.trim().toLowerCase();
  }

  try {
    await apiRequest(`/api/cards/${currentCardId}/labels/${labelId}`, {
      method: 'PATCH',
      body: JSON.stringify({ name: newName.trim(), color: newColor })
    });

    showToast('Метка обновлена', 'success');
    loadCardLabels(currentCardId);
    loadBoards();
  } catch (error) {
    console.error(error);
    showToast('Не удалось обновить метку', 'error');
  }
}

// === Attachments Functions ===
async function loadCardAttachments(cardId) {
  const container = document.getElementById('card-attachments');

  try {
    const attachments = await apiRequest(`/api/cards/${cardId}/attachments`);

    if (!attachments || attachments.length === 0) {
      container.innerHTML = '<div class="empty-state" style="padding:10px;"><p>Нет вложений</p></div>';
    } else {
      container.innerHTML = attachments.map(a => {
        const isImage = a.mime_type && a.mime_type.startsWith('image/');
        const previewHtml = isImage 
          ? `<div style="margin-top:8px;"><img src="/api/attachments/${a.id}" alt="${escapeHtml(a.filename)}" style="max-width:200px; max-height:150px; border-radius:4px; cursor:pointer;" onclick="openImagePreview('/api/attachments/${a.id}')"></div>`
          : '';
        
        return `
        <div class="attachment-item">
          <div style="display:flex; align-items:center; gap:8px; flex:1;">
            <span style="font-size:18px;">${isImage ? '🖼️' : '📎'}</span>
            <a href="/api/attachments/${a.id}" target="_blank" style="flex:1; overflow:hidden; text-overflow:ellipsis; white-space:nowrap; text-decoration:none; color:var(--text-primary);">
              ${escapeHtml(a.filename)}
            </a>
            <span style="font-size:11px; color:var(--text-secondary);">${formatFileSize(a.file_size)}</span>
          </div>
          <button class="btn btn-secondary btn-sm" onclick="deleteAttachment(${a.id})" style="padding:2px 6px;margin-left:8px;">✕</button>
        </div>
        ${previewHtml}
      `;
      }).join('');
    }
  } catch (error) {
    container.innerHTML = '<div class="empty-state" style="padding:10px;"><p>Ошибка загрузки</p></div>';
  }
}

function formatFileSize(bytes) {
  if (bytes < 1024) return bytes + ' Б';
  if (bytes < 1024 * 1024) return (bytes / 1024).toFixed(1) + ' КБ';
  return (bytes / (1024 * 1024)).toFixed(1) + ' МБ';
}

function openImagePreview(url) {
  const modal = document.getElementById('image-preview-modal');
  const img = document.getElementById('image-preview-img');
  img.src = url;
  modal.classList.add('open');
}

function closeImagePreview() {
  const modal = document.getElementById('image-preview-modal');
  modal.classList.remove('open');
}

async function handleFileSelect(input) {
  const file = input.files[0];
  if (!file) return;
  
  const formData = new FormData();
  formData.append('file', file);
  
  try {
    const token = localStorage.getItem('token');
    const response = await fetch(`/api/cards/${currentCardId}/boards/${currentBoardId}/attachments`, {
      method: 'POST',
      headers: {
        'Authorization': `Bearer ${token}`
      },
      body: formData
    });
    
    if (!response.ok) {
      throw new Error('Ошибка загрузки');
    }
    
    showToast('Файл загружен', 'success');
    loadCardAttachments(currentCardId);
    loadBoards();
  } catch (error) {
    console.error(error);
    showToast('Не удалось загрузить файл', 'error');
  }
  
  input.value = '';
}

async function deleteAttachment(attachmentId) {
  if (!confirm('Удалить вложение?')) return;
  
  try {
    await apiRequest(`/api/cards/${currentCardId}/attachments/${attachmentId}`, {
      method: 'DELETE'
    });
    
    showToast('Вложение удалено', 'success');
    loadCardAttachments(currentCardId);
    loadBoards();
  } catch (error) {
    console.error(error);
    showToast('Не удалось удалить вложение', 'error');
  }
}

// === Activity Log Functions ===
async function loadBoardActivity(boardId) {
  const container = document.getElementById('card-activity');
  
  try {
    const activities = await apiRequest(`/api/boards/${boardId}/activity`);
    
    if (!activities || activities.length === 0) {
      container.innerHTML = '<div class="empty-state" style="padding:10px;"><p>Нет записей</p></div>';
    } else {
      container.innerHTML = activities.map(a => `
        <div class="activity-item">
          <div style="font-size:12px; color:var(--text-secondary);">${new Date(a.created_at * 1000).toLocaleString('ru-RU')}</div>
          <div style="margin-top:4px;">${escapeHtml(a.description)}</div>
        </div>
      `).join('');
    }
  } catch (error) {
    container.innerHTML = '<div class="empty-state" style="padding:10px;"><p>Ошибка загрузки</p></div>';
  }
}

async function openActivityModal(boardId) {
  currentBoardId = boardId;
  const modal = document.getElementById('activity-modal');
  const list = document.getElementById('activity-list');
  
  modal.classList.add('open');
  list.innerHTML = '<div class="loading">Загрузка...</div>';
  
  try {
    const activities = await apiRequest(`/api/boards/${boardId}/activity`);
    
    if (!activities || activities.length === 0) {
      list.innerHTML = '<div class="empty-state"><p>Нет записей в истории</p></div>';
    } else {
      list.innerHTML = activities.map(a => `
        <div class="activity-item">
          <div style="font-size:12px; color:var(--text-secondary);">${new Date(a.created_at * 1000).toLocaleString('ru-RU')}</div>
          <div style="margin-top:4px;">${escapeHtml(a.description)}</div>
        </div>
      `).join('');
    }
  } catch (error) {
    console.error(error);
    list.innerHTML = '<div class="empty-state"><p>Ошибка загрузки</p></div>';
  }
}

function closeActivityModal() {
  const modal = document.getElementById('activity-modal');
  modal.classList.remove('open');
  currentBoardId = null;
}

// === Sessions Management ===
async function openSessionsModal() {
  const modal = document.getElementById('sessions-modal');
  const sessionsList = document.getElementById('sessions-list');

  modal.classList.add('open');
  sessionsList.innerHTML = '<div class="loading">Загрузка...</div>';

  try {
    const sessions = await apiRequest('/api/sessions');

    if (!sessions || sessions.length === 0) {
      sessionsList.innerHTML = '<div class="empty-state"><p>Нет активных сессий</p></div>';
    } else {
      const now = Date.now() / 1000;
      sessionsList.innerHTML = sessions.map(s => {
        const isExpired = s.expires_at < now;
        const isCurrent = s.last_activity > now - 60; // Активна в последнюю минуту
        const expiresDate = new Date(s.expires_at * 1000).toLocaleString('ru-RU');
        const lastActivity = new Date(s.last_activity * 1000).toLocaleString('ru-RU');
        const userAgent = s.user_agent || 'Неизвестно';
        const ip = s.ip_address || 'Неизвестно';

        return `
        <div class="session-item ${isCurrent ? 'session-current' : ''} ${isExpired ? 'session-expired' : ''}">
          <div style="flex:1;">
            <div style="display:flex; align-items:center; gap:8px; margin-bottom:4px;">
              <strong>${isCurrent ? '🟢 Текущая' : isExpired ? '🔴 Истекла' : '🟡 Активна'}</strong>
              <span style="font-size:11px; color:var(--text-secondary);">${userAgent.substring(0, 50)}${userAgent.length > 50 ? '...' : ''}</span>
            </div>
            <div style="font-size:11px; color:var(--text-secondary);">IP: ${ip}</div>
            <div style="font-size:11px; color:var(--text-secondary);">Последняя активность: ${lastActivity}</div>
            <div style="font-size:11px; color:var(--text-secondary);">Истекает: ${expiresDate}</div>
          </div>
          ${!isCurrent ? `<button class="btn btn-secondary" onclick="deleteSession(${s.id})" style="padding:4px 10px;font-size:11px;margin-left:8px;">Завершить</button>` : ''}
        </div>
      `;
      }).join('');
    }
  } catch (error) {
    console.error(error);
    sessionsList.innerHTML = '<div class="empty-state"><p>Ошибка загрузки сессий</p></div>';
    showToast('Не удалось загрузить сессии', 'error');
  }
}

function closeSessionsModal() {
  const modal = document.getElementById('sessions-modal');
  modal.classList.remove('open');
}

async function deleteSession(sessionId) {
  if (!confirm('Завершить эту сессию?')) return;

  try {
    await apiRequest(`/api/sessions/${sessionId}`, { method: 'DELETE' });
    showToast('Сессия завершена', 'success');
    openSessionsModal();
  } catch (error) {
    console.error(error);
    showToast('Не удалось завершить сессию', 'error');
  }
}

async function logoutAllSessions() {
  if (!confirm('Завершить ВСЕ сессии? Вам придётся войти заново.')) return;

  try {
    await apiRequest('/api/sessions', { method: 'DELETE' });
    showToast('Все сессии завершены', 'success');
    localStorage.removeItem('token');
    localStorage.removeItem('user');
    window.location.href = '/login.html';
  } catch (error) {
    console.error(error);
    showToast('Не удалось завершить сессии', 'error');
  }
}

// === Export Functions ===
function exportBoardJSON(boardId) {
  const link = document.createElement('a');
  link.href = `/api/boards/${boardId}/export/json`;
  link.download = `board_${boardId}_export.json`;
  link.target = '_blank';
  document.body.appendChild(link);
  link.click();
  document.body.removeChild(link);
  showToast('Экспорт в JSON начат', 'success');
}

function exportBoardCSV(boardId) {
  const link = document.createElement('a');
  link.href = `/api/boards/${boardId}/export/csv`;
  link.download = `board_${boardId}_export.csv`;
  link.target = '_blank';
  document.body.appendChild(link);
  link.click();
  document.body.removeChild(link);
  showToast('Экспорт в CSV начат', 'success');
}

async function openBoardStats(boardId) {
  const modal = document.getElementById('stats-modal');
  const statsContainer = document.getElementById('stats-content');

  modal.classList.add('open');
  statsContainer.innerHTML = '<div class="loading">Загрузка статистики...</div>';

  try {
    const stats = await apiRequest(`/api/boards/${boardId}/stats`);

    statsContainer.innerHTML = `
      <div class="stats-grid">
        <div class="stat-card">
          <div class="stat-value">${stats.total_lists}</div>
          <div class="stat-label">📋 Списков</div>
        </div>
        <div class="stat-card">
          <div class="stat-value">${stats.total_cards}</div>
          <div class="stat-label">📝 Карточек всего</div>
        </div>
        <div class="stat-card">
          <div class="stat-value">${stats.completed_cards}</div>
          <div class="stat-label">✅ Выполнено</div>
        </div>
        <div class="stat-card">
          <div class="stat-value">${stats.pending_cards}</div>
          <div class="stat-label">⏳ В ожидании</div>
        </div>
        <div class="stat-card">
          <div class="stat-value">${stats.completion_percentage.toFixed(1)}%</div>
          <div class="stat-label">📊 Завершено</div>
        </div>
        <div class="stat-card">
          <div class="stat-value">${stats.total_labels}</div>
          <div class="stat-label">🏷️ Меток</div>
        </div>
        <div class="stat-card">
          <div class="stat-value">${stats.cards_with_due_date}</div>
          <div class="stat-label">📅 С дедлайном</div>
        </div>
        <div class="stat-card ${stats.overdue_cards > 0 ? 'stat-warning' : ''}">
          <div class="stat-value">${stats.overdue_cards}</div>
          <div class="stat-label">⚠️ Просрочено</div>
        </div>
      </div>
    `;
  } catch (error) {
    console.error(error);
    statsContainer.innerHTML = '<div class="empty-state"><p>Ошибка загрузки статистики</p></div>';
    showToast('Не удалось загрузить статистику', 'error');
  }
}

function closeBoardStats() {
  const modal = document.getElementById('stats-modal');
  modal.classList.remove('open');
}

// === Checklist Functions ===
async function loadCardChecklists(cardId) {
  const container = document.getElementById('card-checklists');

  try {
    const checklists = await apiRequest(`/api/cards/${cardId}/checklists`);

    if (!checklists || checklists.length === 0) {
      container.innerHTML = '<div class="empty-state" style="padding:10px;"><p>Нет чек-листов</p></div>';
    } else {
      container.innerHTML = checklists.map(cl => {
        const total = cl.items.length;
        const done = cl.items.filter(i => i.done).length;
        const percent = total > 0 ? Math.round((done / total) * 100) : 0;

        return `
        <div class="checklist-item-container">
          <div class="checklist-header">
            <div style="display:flex; align-items:center; gap:8px; flex:1;">
              <span style="font-weight:600;">${escapeHtml(cl.title)}</span>
              <span style="font-size:11px; color:var(--text-secondary);">${done}/${total} (${percent}%)</span>
            </div>
            <div style="display:flex; gap:4px;">
              <button class="btn btn-secondary btn-sm" onclick="addChecklistItem(${cl.id}, ${cardId})" title="Добавить элемент">➕</button>
              <button class="btn btn-secondary btn-sm" onclick="deleteChecklist(${cardId}, ${cl.id})" title="Удалить чек-лист">🗑️</button>
            </div>
          </div>
          <div class="checklist-progress" style="width:100%; height:4px; background:#dfe1e6; border-radius:2px; margin:8px 0;">
            <div style="width:${percent}%; height:100%; background:var(--success-color); border-radius:2px; transition:width 0.3s;"></div>
          </div>
          <div class="checklist-items">
            ${cl.items.map(item => `
              <div class="checklist-item ${item.done ? 'checklist-item-done' : ''}">
                <input type="checkbox" ${item.done ? 'checked' : ''} onchange="toggleChecklistItem(${cardId}, ${cl.id}, ${item.id}, this.checked)">
                <span style="flex:1; ${item.done ? 'text-decoration:line-through; color:var(--text-secondary);' : ''}">${escapeHtml(item.title)}</span>
                <button class="btn btn-secondary btn-sm" onclick="deleteChecklistItem(${cardId}, ${cl.id}, ${item.id})" style="padding:2px 6px;">✕</button>
              </div>
            `).join('')}
          </div>
        </div>
      `;
      }).join('');
    }
  } catch (error) {
    container.innerHTML = '<div class="empty-state" style="padding:10px;"><p>Ошибка загрузки</p></div>';
  }
}

async function createChecklist(cardId) {
  const title = prompt('Название чек-листа:');
  if (!title || title.trim() === '') return;

  try {
    await apiRequest(`/api/cards/${cardId}/checklists`, {
      method: 'POST',
      body: JSON.stringify({ title: title.trim() })
    });
    showToast('Чек-лист создан', 'success');
    loadCardChecklists(cardId);
  } catch (error) {
    console.error(error);
    showToast('Не удалось создать чек-лист', 'error');
  }
}

async function deleteChecklist(cardId, checklistId) {
  if (!confirm('Удалить чек-лист?')) return;

  try {
    await apiRequest(`/api/cards/${cardId}/checklists/${checklistId}`, { method: 'DELETE' });
    showToast('Чек-лист удалён', 'success');
    loadCardChecklists(cardId);
  } catch (error) {
    console.error(error);
    showToast('Не удалось удалить чек-лист', 'error');
  }
}

async function addChecklistItem(checklistId, cardId) {
  const title = prompt('Название элемента:');
  if (!title || title.trim() === '') return;

  try {
    await apiRequest(`/api/cards/${cardId}/checklists/${checklistId}/items`, {
      method: 'POST',
      body: JSON.stringify({ title: title.trim() })
    });
    showToast('Элемент добавлен', 'success');
    loadCardChecklists(cardId);
  } catch (error) {
    console.error(error);
    showToast('Не удалось добавить элемент', 'error');
  }
}

async function toggleChecklistItem(cardId, checklistId, itemId, done) {
  try {
    await apiRequest(`/api/cards/${cardId}/checklists/${checklistId}/items/${itemId}`, {
      method: 'PATCH',
      body: JSON.stringify({ done })
    });
    loadCardChecklists(cardId);
  } catch (error) {
    console.error(error);
    showToast('Не удалось обновить элемент', 'error');
  }
}

async function deleteChecklistItem(cardId, checklistId, itemId) {
  if (!confirm('Удалить элемент?')) return;

  try {
    await apiRequest(`/api/cards/${cardId}/checklists/${checklistId}/items/${itemId}`, { method: 'DELETE' });
    showToast('Элемент удалён', 'success');
    loadCardChecklists(cardId);
  } catch (error) {
    console.error(error);
    showToast('Не удалось удалить элемент', 'error');
  }
}

// === Assignee Functions ===
async function loadCardAssignees(cardId) {
  const container = document.getElementById('card-assignees');

  try {
    const assignees = await apiRequest(`/api/cards/${cardId}/assignees`);

    if (!assignees || assignees.length === 0) {
      container.innerHTML = '<div class="empty-state" style="padding:10px;"><p>Нет исполнителей</p></div>';
    } else {
      container.innerHTML = `
        <div class="assignees-list">
          ${assignees.map(a => `
            <div class="assignee-item">
              <span style="font-weight:500;">👤 ${escapeHtml(a.username)}</span>
              <button class="btn btn-secondary btn-sm" onclick="removeAssignee(${cardId}, ${a.user_id})" style="padding:2px 6px;">✕</button>
            </div>
          `).join('')}
        </div>
        <button class="btn btn-secondary btn-sm" onclick="addAssignee(${cardId})" style="margin-top:8px;">➕ Добавить исполнителя</button>
      `;
    }
  } catch (error) {
    container.innerHTML = '<div class="empty-state" style="padding:10px;"><p>Ошибка загрузки</p></div>';
  }
}

async function addAssignee(cardId) {
  const username = prompt('Введите имя пользователя для назначения:');
  if (!username || username.trim() === '') return;

  try {
    // Ищем пользователя
    const users = await apiRequest(`/api/users?username=${encodeURIComponent(username.trim())}`);
    if (!users || users.length === 0) {
      showToast('Пользователь не найден', 'error');
      return;
    }

    const userId = users[0].id;

    await apiRequest(`/api/cards/${cardId}/assignees`, {
      method: 'POST',
      body: JSON.stringify({ user_id: userId })
    });
    showToast('Исполнитель назначен', 'success');
    loadCardAssignees(cardId);
  } catch (error) {
    console.error(error);
    showToast('Не удалось назначить исполнителя', 'error');
  }
}

async function removeAssignee(cardId, userId) {
  if (!confirm('Удалить исполнителя?')) return;

  try {
    await apiRequest(`/api/cards/${cardId}/assignees/${userId}`, { method: 'DELETE' });
    showToast('Исполнитель удалён', 'success');
    loadCardAssignees(cardId);
  } catch (error) {
    console.error(error);
    showToast('Не удалось удалить исполнителя', 'error');
  }
}

// === Search and Filter Functions ===
async function handleBoardSearch(boardId, query) {
  const searchResultsContainer = document.querySelector(`.board-search-results[data-board-id="${boardId}"]`);
  
  if (!query || query.trim() === '') {
    if (searchResultsContainer) searchResultsContainer.remove();
    return;
  }

  try {
    const results = await apiRequest(`/api/boards/${boardId}/search?q=${encodeURIComponent(query)}`);
    
    // Удаляем старую панель результатов
    if (searchResultsContainer) searchResultsContainer.remove();
    
    // Создаём панель результатов
    const resultsDiv = document.createElement('div');
    resultsDiv.className = 'board-search-results';
    resultsDiv.setAttribute('data-board-id', boardId);
    resultsDiv.innerHTML = `
      <div class="search-results-header">
        <strong>🔍 Результаты поиска "${escapeHtml(query)}":</strong>
        <span>${results.length} найдено</span>
      </div>
      <div class="search-results-list">
        ${results.length === 0 ? '<div class="empty-state">Ничего не найдено</div>' : ''}
        ${results.map(card => `
          <div class="search-result-card" onclick="openCardModal(${card.id}, ${card.board_id})">
            <div style="display:flex; justify-content:space-between; align-items:start;">
              <div style="flex:1;">
                <div style="display:flex; align-items:center; gap:8px; margin-bottom:4px;">
                  <span style="font-size:11px; color:var(--text-secondary);">${escapeHtml(card.board_title)} / ${escapeHtml(card.list_title)}</span>
                </div>
                <strong>${escapeHtml(card.title)}</strong>
                ${card.content ? `<p style="font-size:12px; color:var(--text-secondary); margin-top:4px;">${escapeHtml(card.content).substring(0, 100)}${card.content.length > 100 ? '...' : ''}</p>` : ''}
                <div style="display:flex; gap:4px; margin-top:6px; flex-wrap:wrap;">
                  ${(card.labels || []).map(l => `<span class="label-badge label-${l.color}" style="font-size:10px;">${escapeHtml(l.name)}</span>`).join('')}
                </div>
              </div>
              ${card.done ? '<span style="color:var(--success-color); font-size:18px;">✅</span>' : ''}
            </div>
          </div>
        `).join('')}
      </div>
    `;
    
    // Вставляем после search bar
    const searchBar = document.querySelector(`.board-search-bar[data-board-id="${boardId}"]`);
    searchBar.parentNode.insertBefore(resultsDiv, searchBar.nextSibling);
  } catch (error) {
    console.error(error);
  }
}

function clearBoardSearch(boardId) {
  const searchInput = document.querySelector(`.board-search-bar[data-board-id="${boardId}"] .board-search-input`);
  if (searchInput) searchInput.value = '';
  
  const resultsContainer = document.querySelector(`.board-search-results[data-board-id="${boardId}"]`);
  if (resultsContainer) resultsContainer.remove();
}

async function openLabelFilter(boardId) {
  try {
    const labels = await apiRequest(`/api/boards/${boardId}/labels`);
    
    const modal = document.getElementById('label-filter-modal');
    const filterContainer = document.getElementById('label-filter-content');
    
    modal.classList.add('open');
    
    if (!labels || labels.length === 0) {
      filterContainer.innerHTML = '<div class="empty-state"><p>Нет меток на этой доске</p></div>';
    } else {
      filterContainer.innerHTML = `
        <div class="label-filter-list">
          ${labels.map(l => `
            <div class="label-filter-item label-${l.color}" onclick="filterByLabel(${boardId}, '${l.color}', '${escapeJs(l.name)}')">
              <span class="label-color-dot" style="background:${getLabelColorHex(l.color)}"></span>
              <span>${escapeHtml(l.name)}</span>
              <span style="font-size:11px; color:var(--text-secondary); margin-left:auto;">${l.color}</span>
            </div>
          `).join('')}
        </div>
      `;
    }
  } catch (error) {
    console.error(error);
    showToast('Не удалось загрузить метки', 'error');
  }
}

function closeLabelFilter() {
  const modal = document.getElementById('label-filter-modal');
  modal.classList.remove('open');
}

async function filterByLabel(boardId, color, name) {
  try {
    const results = await apiRequest(`/api/boards/${boardId}/search?label_color=${encodeURIComponent(color)}`);
    
    closeLabelFilter();
    
    // Показываем результаты в search results панели
    const searchResultsContainer = document.querySelector(`.board-search-results[data-board-id="${boardId}"]`);
    if (searchResultsContainer) searchResultsContainer.remove();
    
    const resultsDiv = document.createElement('div');
    resultsDiv.className = 'board-search-results';
    resultsDiv.setAttribute('data-board-id', boardId);
    resultsDiv.innerHTML = `
      <div class="search-results-header">
        <strong>🏷️ Фильтр по метке "${escapeHtml(name)}" (${color}):</strong>
        <span>${results.length} найдено</span>
        <button class="btn btn-secondary btn-sm" onclick="clearBoardSearch(${boardId})" style="margin-left:auto;">✕ Очистить</button>
      </div>
      <div class="search-results-list">
        ${results.length === 0 ? '<div class="empty-state">Ничего не найдено</div>' : ''}
        ${results.map(card => `
          <div class="search-result-card" onclick="openCardModal(${card.id}, ${card.board_id})">
            <div style="display:flex; justify-content:space-between; align-items:start;">
              <div style="flex:1;">
                <div style="display:flex; align-items:center; gap:8px; margin-bottom:4px;">
                  <span style="font-size:11px; color:var(--text-secondary);">${escapeHtml(card.board_title)} / ${escapeHtml(card.list_title)}</span>
                </div>
                <strong>${escapeHtml(card.title)}</strong>
                ${card.content ? `<p style="font-size:12px; color:var(--text-secondary); margin-top:4px;">${escapeHtml(card.content).substring(0, 100)}${card.content.length > 100 ? '...' : ''}</p>` : ''}
                <div style="display:flex; gap:4px; margin-top:6px; flex-wrap:wrap;">
                  ${(card.labels || []).map(l => `<span class="label-badge label-${l.color}" style="font-size:10px;">${escapeHtml(l.name)}</span>`).join('')}
                </div>
              </div>
              ${card.done ? '<span style="color:var(--success-color); font-size:18px;">✅</span>' : ''}
            </div>
          </div>
        `).join('')}
      </div>
    `;
    
    const searchBar = document.querySelector(`.board-search-bar[data-board-id="${boardId}"]`);
    searchBar.parentNode.insertBefore(resultsDiv, searchBar.nextSibling);
  } catch (error) {
    console.error(error);
    showToast('Не удалось применить фильтр', 'error');
  }
}

function getLabelColorHex(color) {
  const colors = {
    'blue': '#0079bf',
    'green': '#61bd4f',
    'yellow': '#f2d600',
    'red': '#eb5a46',
    'purple': '#c377e0',
    'orange': '#ff9f1a',
  };
  return colors[color] || '#0079bf';
}

// === Calendar Functions ===
let currentCalendarYear = new Date().getFullYear();
let currentCalendarMonth = new Date().getMonth() + 1;
let currentCalendarBoardId = null;

async function openCalendarModal() {
  const modal = document.getElementById('calendar-modal');
  modal.classList.add('open');
  
  // Загружаем календарь для текущего месяца
  currentCalendarYear = new Date().getFullYear();
  currentCalendarMonth = new Date().getMonth() + 1;
  
  // Берём первую доступную доску для календаря
  const boards = await apiRequest('/api/boards');
  if (boards && boards.length > 0) {
    currentCalendarBoardId = boards[0].id;
    loadCalendar(currentCalendarBoardId, currentCalendarYear, currentCalendarMonth);
  } else {
    document.getElementById('calendar-grid').innerHTML = '<div class="empty-state">Нет досок для отображения календаря</div>';
  }
}

function closeCalendarModal() {
  const modal = document.getElementById('calendar-modal');
  modal.classList.remove('open');
  document.getElementById('calendar-day-cards').style.display = 'none';
}

async function loadCalendar(boardId, year, month) {
  const grid = document.getElementById('calendar-grid');
  const monthYearLabel = document.getElementById('calendar-month-year');
  const totalCardsLabel = document.getElementById('calendar-total-cards');
  const overdueCardsLabel = document.getElementById('calendar-overdue-cards');
  
  grid.innerHTML = '<div class="loading">Загрузка календаря...</div>';
  
  try {
    const calendar = await apiRequest(`/api/boards/${boardId}/calendar?year=${year}&month=${month}`);
    
    const monthNames = [
      'Январь', 'Февраль', 'Март', 'Апрель', 'Май', 'Июнь',
      'Июль', 'Август', 'Сентябрь', 'Октябрь', 'Ноябрь', 'Декабрь'
    ];
    
    monthYearLabel.textContent = `${monthNames[month - 1]} ${year}`;
    totalCardsLabel.textContent = calendar.total_cards;
    overdueCardsLabel.textContent = calendar.overdue_cards;
    
    // Создаём сетку календаря
    const daysInMonth = calendar.days.length;
    const firstDay = new Date(year, month - 1, 1).getDay(); // 0 = воскресенье
    const adjustedFirstDay = firstDay === 0 ? 6 : firstDay - 1; // 0 = понедельник
    
    let html = `
      <div class="calendar-weekdays">
        <div class="calendar-weekday">Пн</div>
        <div class="calendar-weekday">Вт</div>
        <div class="calendar-weekday">Ср</div>
        <div class="calendar-weekday">Чт</div>
        <div class="calendar-weekday">Пт</div>
        <div class="calendar-weekday" style="color:#eb5a46;">Сб</div>
        <div class="calendar-weekday" style="color:#eb5a46;">Вс</div>
      </div>
      <div class="calendar-days-grid">
    `;
    
    // Пустые ячейки до первого дня месяца
    for (let i = 0; i < adjustedFirstDay; i++) {
      html += '<div class="calendar-day-empty"></div>';
    }
    
    // Дни месяца
    for (let i = 0; i < daysInMonth; i++) {
      const day = calendar.days[i];
      const hasCards = day.cards_count > 0;
      const isToday = day.has_today;
      const hasOverdue = day.overdue_count > 0;
      
      html += `
        <div class="calendar-day ${isToday ? 'calendar-day-today' : ''} ${hasCards ? 'calendar-day-has-cards' : ''}" 
             onclick="selectCalendarDay(${boardId}, '${day.date}', '${day.date}')">
          <div class="calendar-day-number">${day.day}</div>
          ${hasCards ? `<div class="calendar-day-indicator ${hasOverdue ? 'calendar-day-overdue' : ''}"></div>` : ''}
          ${hasCards ? `<div class="calendar-day-count">${day.cards_count}</div>` : ''}
        </div>
      `;
    }
    
    html += '</div>';
    grid.innerHTML = html;
    
    // Скрываем панель выбранных дней
    document.getElementById('calendar-day-cards').style.display = 'none';
    
  } catch (error) {
    console.error(error);
    grid.innerHTML = '<div class="empty-state">Ошибка загрузки календаря</div>';
    showToast('Не удалось загрузить календарь', 'error');
  }
}

function previousMonth() {
  currentCalendarMonth--;
  if (currentCalendarMonth < 1) {
    currentCalendarMonth = 12;
    currentCalendarYear--;
  }
  if (currentCalendarBoardId) {
    loadCalendar(currentCalendarBoardId, currentCalendarYear, currentCalendarMonth);
  }
}

function nextMonth() {
  currentCalendarMonth++;
  if (currentCalendarMonth > 12) {
    currentCalendarMonth = 1;
    currentCalendarYear++;
  }
  if (currentCalendarBoardId) {
    loadCalendar(currentCalendarBoardId, currentCalendarYear, currentCalendarMonth);
  }
}

function goToToday() {
  currentCalendarYear = new Date().getFullYear();
  currentCalendarMonth = new Date().getMonth() + 1;
  if (currentCalendarBoardId) {
    loadCalendar(currentCalendarBoardId, currentCalendarYear, currentCalendarMonth);
  }
}

async function selectCalendarDay(boardId, date, dateLabel) {
  const [year, month, day] = date.split('-').map(Number);
  
  const cardsList = document.getElementById('calendar-selected-day-list');
  const dayTitle = document.getElementById('calendar-selected-day-title');
  const cardsPanel = document.getElementById('calendar-day-cards');
  
  cardsPanel.style.display = 'block';
  dayTitle.textContent = `📅 ${dateLabel}`;
  cardsList.innerHTML = '<div class="loading">Загрузка...</div>';
  
  try {
    const cards = await apiRequest(`/api/boards/${boardId}/calendar/${year}/${month}/${day}`);
    
    if (!cards || cards.length === 0) {
      cardsList.innerHTML = '<div class="empty-state">Нет карточек на этот день</div>';
    } else {
      cardsList.innerHTML = cards.map(card => `
        <div class="calendar-card-item ${card.done ? 'calendar-card-done' : ''} ${card.is_overdue ? 'calendar-card-overdue' : ''}" 
             onclick="openCardModal(${card.id}, ${card.board_id})">
          <div style="display:flex; justify-content:space-between; align-items:start;">
            <div style="flex:1;">
              <strong>${escapeHtml(card.title)}</strong>
              <div style="font-size:11px; color:var(--text-secondary); margin-top:4px;">
                ${escapeHtml(card.board_title)} / ${escapeHtml(card.list_title)}
              </div>
            </div>
            ${card.done ? '<span style="color:var(--success-color);">✅</span>' : ''}
            ${card.is_overdue ? '<span style="color:var(--danger-color);">⚠️</span>' : ''}
          </div>
        </div>
      `).join('');
    }
  } catch (error) {
    console.error(error);
    cardsList.innerHTML = '<div class="empty-state">Ошибка загрузки</div>';
  }
  
  // Прокрутка к панели карточек
  cardsPanel.scrollIntoView({ behavior: 'smooth', block: 'nearest' });
}
window.deleteBoard = deleteBoard;
window.deleteList = deleteList;
window.deleteCard = deleteCard;
window.editCard = editCard;
window.showAddListForm = showAddListForm;
window.showAddCardForm = showAddCardForm;
window.createList = createList;
window.cancelAddList = cancelAddList;
window.createCard = createCard;
window.cancelAddCard = cancelAddCard;
window.saveCardEdit = saveCardEdit;
window.cancelCardEdit = cancelCardEdit;
window.toggleCardDone = toggleCardDone;
window.openMembersModal = openMembersModal;
window.closeMembersModal = closeMembersModal;
window.addMember = addMember;
window.removeMember = removeMember;
window.changeMemberRole = changeMemberRole;
window.openCommentsModal = openCommentsModal;
window.closeCommentsModal = closeCommentsModal;
window.addComment = addComment;
window.deleteComment = deleteComment;
window.editComment = editComment;
window.logout = logout;
window.openCardModal = openCardModal;
window.closeCardModal = closeCardModal;
window.saveCardFromModal = saveCardFromModal;
window.clearDueDate = clearDueDate;
window.addLabel = addLabel;
window.deleteLabel = deleteLabel;
window.handleFileSelect = handleFileSelect;
window.deleteAttachment = deleteAttachment;
window.openActivityModal = openActivityModal;
window.closeActivityModal = closeActivityModal;
window.openInvitationsModal = openInvitationsModal;
window.closeInvitationsModal = closeInvitationsModal;
window.createInvitation = createInvitation;
window.deleteInvitation = deleteInvitation;
window.copyInviteLink = copyInviteLink;
window.editLabel = editLabel;
window.openImagePreview = openImagePreview;
window.closeImagePreview = closeImagePreview;
window.openSessionsModal = openSessionsModal;
window.closeSessionsModal = closeSessionsModal;
window.deleteSession = deleteSession;
window.logoutAllSessions = logoutAllSessions;
window.exportBoardJSON = exportBoardJSON;
window.exportBoardCSV = exportBoardCSV;
window.openBoardStats = openBoardStats;
window.closeBoardStats = closeBoardStats;
window.loadCardChecklists = loadCardChecklists;
window.createChecklist = createChecklist;
window.deleteChecklist = deleteChecklist;
window.addChecklistItem = addChecklistItem;
window.toggleChecklistItem = toggleChecklistItem;
window.deleteChecklistItem = deleteChecklistItem;
window.loadCardAssignees = loadCardAssignees;
window.addAssignee = addAssignee;
window.removeAssignee = removeAssignee;
window.handleBoardSearch = handleBoardSearch;
window.clearBoardSearch = clearBoardSearch;
window.openLabelFilter = openLabelFilter;
window.closeLabelFilter = closeLabelFilter;
window.filterByLabel = filterByLabel;
window.getLabelColorHex = getLabelColorHex;
window.openCalendarModal = openCalendarModal;
window.closeCalendarModal = closeCalendarModal;
window.previousMonth = previousMonth;
window.nextMonth = nextMonth;
window.goToToday = goToToday;
window.selectCalendarDay = selectCalendarDay;
window.openProfileModal = openProfileModal;
window.closeProfileModal = closeProfileModal;
window.saveProfile = saveProfile;
window.openChangePassword = openChangePassword;
window.openDeleteAccount = openDeleteAccount;

// === Profile Management ===
async function openProfileModal() {
  const modal = document.getElementById('profile-modal');
  const content = document.getElementById('profile-content');

  modal.classList.add('open');
  content.innerHTML = '<div class="loading">Загрузка профиля...</div>';

  try {
    const user = await apiRequest('/api/profile');

    content.innerHTML = `
      <div class="profile-form">
        <div class="profile-avatar" style="width:80px; height:80px; border-radius:50%; background:${user.avatar_color || '#0079bf'}; display:flex; align-items:center; justify-content:center; font-size:32px; font-weight:bold; color:white; margin:0 auto 16px;">
          ${user.username.charAt(0).toUpperCase()}
        </div>

        <div class="profile-info">
          <div class="profile-field">
            <label>👤 Имя пользователя</label>
            <input type="text" id="profile-username" value="${escapeHtml(user.username)}" disabled style="background:#f4f5f7;">
          </div>

          <div class="profile-field">
            <label>📧 Email</label>
            <input type="email" id="profile-email" value="${escapeHtml(user.email || '')}" placeholder="Не указан">
          </div>

          <div class="profile-field">
            <label>🎨 Цвет аватара</label>
            <input type="color" id="profile-avatar-color" value="${user.avatar_color || '#0079bf'}" style="width:100%; height:40px; cursor:pointer;">
          </div>

          <div class="profile-field">
            <label>📝 О себе</label>
            <textarea id="profile-bio" rows="3" placeholder="Расскажите о себе...">${escapeHtml(user.bio || '')}</textarea>
          </div>

          <div class="profile-field">
            <label>📅 Зарегистрирован</label>
            <input type="text" value="${new Date(user.created_at * 1000).toLocaleDateString('ru-RU')}" disabled style="background:#f4f5f7;">
          </div>

          <div class="profile-field">
            <label>🕐 Последний вход</label>
            <input type="text" value="${user.last_login ? new Date(user.last_login * 1000).toLocaleString('ru-RU') : '—'}" disabled style="background:#f4f5f7;">
          </div>
        </div>

        <div class="profile-actions" style="display:flex; gap:10px; margin-top:20px; flex-wrap:wrap;">
          <button class="btn btn-primary" onclick="saveProfile()" style="flex:1;">💾 Сохранить</button>
          <button class="btn btn-secondary" onclick="openChangePassword()" style="flex:1;">🔑 Сменить пароль</button>
          <button class="btn btn-danger" onclick="openDeleteAccount()" style="flex:1;">🗑️ Удалить аккаунт</button>
        </div>
      </div>
    `;
  } catch (error) {
    console.error(error);
    content.innerHTML = '<div class="empty-state">Ошибка загрузки профиля</div>';
    showToast('Не удалось загрузить профиль', 'error');
  }
}

function closeProfileModal() {
  const modal = document.getElementById('profile-modal');
  modal.classList.remove('open');
}

async function saveProfile() {
  const email = document.getElementById('profile-email').value.trim();
  const avatarColor = document.getElementById('profile-avatar-color').value;
  const bio = document.getElementById('profile-bio').value.trim();

  try {
    const user = await apiRequest('/api/profile', {
      method: 'PATCH',
      body: JSON.stringify({ email, avatar_color: avatarColor, bio: bio || null })
    });

    // Обновляем аватар в UI
    const avatarEl = document.querySelector('.profile-avatar');
    if (avatarEl) {
      avatarEl.style.backgroundColor = avatarColor;
    }

    showToast('Профиль обновлён', 'success');
  } catch (error) {
    console.error(error);
    showToast(error.message || 'Не удалось сохранить профиль', 'error');
  }
}

async function openChangePassword() {
  const currentPassword = prompt('Введите текущий пароль:');
  if (!currentPassword) return;

  const newPassword = prompt('Введите новый пароль (минимум 8 символов, заглавные, строчные, цифры):');
  if (!newPassword) return;

  if (newPassword.length < 8) {
    showToast('Пароль должен быть не менее 8 символов', 'error');
    return;
  }
  
  // Проверка сложности пароля
  const hasUpper = /[A-Z]/.test(newPassword);
  const hasLower = /[a-z]/.test(newPassword);
  const hasDigit = /\d/.test(newPassword);
  
  if (!hasUpper || !hasLower || !hasDigit) {
    showToast('Пароль должен содержать заглавные и строчные буквы, а также цифры', 'error');
    return;
  }

  try {
    await apiRequest('/api/profile/change-password', {
      method: 'POST',
      body: JSON.stringify({ current_password: currentPassword, new_password: newPassword })
    });

    showToast('Пароль успешно изменён', 'success');
    closeProfileModal();
  } catch (error) {
    console.error(error);
    showToast(error.message || 'Не удалось сменить пароль', 'error');
  }
}

async function openDeleteAccount() {
  const password = prompt('⚠️ ВНИМАНИЕ! Это действие необратимо.\n\nВведите ваш пароль для подтверждения удаления аккаунта:');
  if (!password) return;

  if (!confirm('Вы уверены, что хотите удалить аккаунт? Все ваши доски, карточки и данные будут безвозвратно удалены.')) {
    return;
  }

  try {
    await apiRequest('/api/profile/delete', {
      method: 'POST',
      body: JSON.stringify({ password })
    });

    localStorage.removeItem('token');
    localStorage.removeItem('user');
    window.location.href = '/login.html';
  } catch (error) {
    console.error(error);
    showToast(error.message || 'Не удалось удалить аккаунт', 'error');
  }
}

// === Уведомления ===

// Опрос сервера на наличие новых уведомлений (каждые 10 секунд)
let notificationPollingInterval = null;

function startNotificationPolling() {
  checkUnreadNotifications();
  notificationPollingInterval = setInterval(checkUnreadNotifications, 10000);
}

function stopNotificationPolling() {
  if (notificationPollingInterval) {
    clearInterval(notificationPollingInterval);
    notificationPollingInterval = null;
  }
}

async function checkUnreadNotifications() {
  try {
    const count = await apiRequest('/api/notifications/unread-count');
    updateNotificationBadge(count);
  } catch (error) {
    console.error('Ошибка проверки уведомлений:', error);
  }
}

function updateNotificationBadge(count) {
  const badge = document.getElementById('notification-badge');
  if (badge) {
    if (count > 0) {
      badge.textContent = count > 99 ? '99+' : count;
      badge.style.display = 'inline-block';
    } else {
      badge.style.display = 'none';
    }
  }
}

async function openNotificationsModal() {
  const modal = document.getElementById('notifications-modal');
  const notificationsList = document.getElementById('notifications-list');

  modal.classList.add('open');
  notificationsList.innerHTML = '<div class="loading">Загрузка...</div>';

  try {
    const notifications = await apiRequest('/api/notifications?limit=50');

    if (!notifications || notifications.length === 0) {
      notificationsList.innerHTML = '<div class="empty-state"><p>Нет уведомлений</p></div>';
    } else {
      notificationsList.innerHTML = notifications.map(n => `
        <div class="notification-item ${n.is_read ? 'notification-read' : ''}" data-notification-id="${n.id}">
          <div class="notification-header">
            <span class="notification-icon">${getNotificationIcon(n.notification_type)}</span>
            <span class="notification-title">${escapeHtml(n.title)}</span>
            <span class="notification-time">${formatNotificationTime(n.created_at)}</span>
          </div>
          <div class="notification-message">${escapeHtml(n.message)}</div>
          ${n.link ? `<button class="btn btn-sm" onclick="navigateTo('${n.link}')" style="margin-top:8px;">Перейти</button>` : ''}
          ${!n.is_read ? `<button class="btn btn-sm btn-secondary" onclick="markNotificationRead(${n.id})" style="margin-top:8px;">Отметить как прочитанное</button>` : ''}
        </div>
      `).join('');
    }
  } catch (error) {
    console.error(error);
    notificationsList.innerHTML = '<div class="empty-state"><p>Ошибка загрузки уведомлений</p></div>';
  }
}

function closeNotificationsModal() {
  const modal = document.getElementById('notifications-modal');
  modal.classList.remove('open');
}

function getNotificationIcon(type) {
  const icons = {
    'info': 'ℹ️',
    'success': '✅',
    'warning': '⚠️',
    'error': '❌'
  };
  return icons[type] || icons['info'];
}

function formatNotificationTime(timestamp) {
  const date = new Date(timestamp * 1000);
  const now = new Date();
  const diff = now - date;
  const minutes = Math.floor(diff / 60000);
  const hours = Math.floor(diff / 3600000);
  const days = Math.floor(diff / 86400000);

  if (minutes < 1) return 'Только что';
  if (minutes < 60) return `${minutes} мин. назад`;
  if (hours < 24) return `${hours} ч. назад`;
  if (days < 7) return `${days} дн. назад`;
  return date.toLocaleDateString('ru-RU');
}

async function markNotificationRead(notificationId) {
  try {
    await apiRequest(`/api/notifications/${notificationId}`, {
      method: 'PATCH',
      body: JSON.stringify({ is_read: true })
    });

    checkUnreadNotifications();
    openNotificationsModal(); // Обновить список
    showToast('Уведомление отмечено как прочитанное', 'success');
  } catch (error) {
    console.error(error);
    showToast('Не удалось отметить уведомление', 'error');
  }
}

async function markAllNotificationsRead() {
  try {
    await apiRequest('/api/notifications/read-all', {
      method: 'POST'
    });

    checkUnreadNotifications();
    openNotificationsModal(); // Обновить список
    showToast('Все уведомления отмечены как прочитанные', 'success');
  } catch (error) {
    console.error(error);
    showToast('Не удалось отметить уведомления', 'error');
  }
}

function navigateTo(link) {
  if (link) {
    window.location.href = link;
  }
}

// === Массовые операции ===

async function bulkMoveCards() {
  if (selectedCards.size === 0) {
    showToast('Выберите карточки для перемещения', 'warning');
    return;
  }
  
  const listId = prompt('Введите ID списка для перемещения:');
  if (!listId) return;
  
  try {
    const response = await apiRequest('/api/cards/bulk/move', {
      method: 'POST',
      body: JSON.stringify({
        card_ids: Array.from(selectedCards),
        list_id: parseInt(listId)
      })
    });
    
    if (response.success) {
      showToast(`Перемещено ${response.processed_count} карточек`, 'success');
      clearCardSelection();
      toggleBulkMode();
      loadBoards();
    } else {
      showToast(`Перемещено: ${response.processed_count}, ошибок: ${response.failed_count}`, 'warning');
    }
  } catch (error) {
    console.error(error);
    showToast('Ошибка массового перемещения', 'error');
  }
}

async function bulkMarkDone() {
  if (selectedCards.size === 0) {
    showToast('Выберите карточки для отметки', 'warning');
    return;
  }
  
  try {
    const response = await apiRequest('/api/cards/bulk/update', {
      method: 'POST',
      body: JSON.stringify({
        card_ids: Array.from(selectedCards),
        done: true
      })
    });
    
    if (response.success) {
      showToast(`Отмечено ${response.processed_count} карточек`, 'success');
      clearCardSelection();
      toggleBulkMode();
      loadBoards();
    } else {
      showToast(`Обновлено: ${response.processed_count}, ошибок: ${response.failed_count}`, 'warning');
    }
  } catch (error) {
    console.error(error);
    showToast('Ошибка массового обновления', 'error');
  }
}

async function bulkMarkTodo() {
  if (selectedCards.size === 0) {
    showToast('Выберите карточки для возврата', 'warning');
    return;
  }
  
  try {
    const response = await apiRequest('/api/cards/bulk/update', {
      method: 'POST',
      body: JSON.stringify({
        card_ids: Array.from(selectedCards),
        done: false
      })
    });
    
    if (response.success) {
      showToast(`Возвращено в работу ${response.processed_count} карточек`, 'success');
      clearCardSelection();
      toggleBulkMode();
      loadBoards();
    } else {
      showToast(`Обновлено: ${response.processed_count}, ошибок: ${response.failed_count}`, 'warning');
    }
  } catch (error) {
    console.error(error);
    showToast('Ошибка массового обновления', 'error');
  }
}

async function bulkDeleteCards() {
  if (selectedCards.size === 0) {
    showToast('Выберите карточки для удаления', 'warning');
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
      loadBoards();
    } else {
      showToast(`Удалено: ${response.processed_count}, ошибок: ${response.failed_count}`, 'warning');
    }
  } catch (error) {
    console.error(error);
    showToast('Ошибка массового удаления', 'error');
  }
}

// === Init ===
// Показываем кнопку уведомлений если пользователь авторизован
if (localStorage.getItem('token')) {
  const notificationsBtn = document.getElementById('notifications-btn');
  if (notificationsBtn) {
    notificationsBtn.style.display = 'inline-block';
  }
  startNotificationPolling();
}

loadBoards();
