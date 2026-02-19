// frontend/app.js

// === State ===
let draggedCard = null;
let draggedFromList = null;
let searchQuery = '';
let currentBoardId = null;
let isLoading = false;

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
  
  const icon = type === 'success' ? '✓' : type === 'error' ? '✕' : 'ℹ';
  toast.innerHTML = `<span>${icon}</span><span>${message}</span>`;
  
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
          </div>
          <div class="board-actions">
            <span class="members-count" title="Участники">👥 ${board.members.length}</span>
            <button class="btn btn-secondary" onclick="openMembersModal(${board.id})" title="Участники">👤</button>
            <button class="btn btn-secondary" onclick="deleteBoard(${board.id})" title="Удалить доску">🗑️</button>
          </div>
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
                  <div class="card"
                       draggable="true"
                       data-card-id="${card.id}"
                       data-list-id="${list.id}"
                       ondblclick="editCard(${card.id}, '${escapeJs(board.id)}', '${escapeJs(list.id)}')">
                    <div style="display:flex; justify-content:space-between; align-items:start;">
                      <label style="display:flex; align-items:flex-start; gap:8px; cursor:pointer; flex:1;">
                        <input type="checkbox" ${card.done ? 'checked' : ''} onchange="toggleCardDone(${card.id}, this.checked)" style="margin-top:3px;">
                        <span>
                          <strong class="${card.done ? 'done' : ''}">${escapeHtml(card.title)}</strong>
                          ${card.content ? `<p>${escapeHtml(card.content)}</p>` : ''}
                        </span>
                      </label>
                      <div class="card-actions">
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
  const title = titleInput.value.trim();
  const content = contentInput.value.trim() || null;
  
  if (!title) {
    titleInput.focus();
    return;
  }
  
  try {
    await apiRequest(`/api/lists/${listId}/cards`, {
      method: 'POST',
      body: JSON.stringify({ title, content })
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
          <span><strong>${escapeHtml(m.username)}</strong></span>
          <button class="btn btn-secondary" onclick="removeMember(${m.id})" style="padding:4px 10px;font-size:11px;">Удалить</button>
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

    // Добавляем участника на доску
    await apiRequest(`/api/boards/${currentBoardId}/members`, {
      method: 'POST',
      body: JSON.stringify({ user_id: userId, role: 'member' })
    });

    usernameInput.value = '';
    showToast('Участник добавлен', 'success');
    openMembersModal(currentBoardId);
    loadBoards();
  } catch (error) {
    console.error(error);
    showToast(error.message || 'Ошибка добавления участника', 'error');
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

// === Export Functions for Inline Handlers ===
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
window.logout = logout;

// === Init ===
loadBoards();
