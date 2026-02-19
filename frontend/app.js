// frontend/app.js

let draggedCard = null;
let draggedFromList = null;
let searchQuery = '';

const boardsContainer = document.getElementById('boards');
const createBoardBtn = document.getElementById('create-board-btn');
const searchInput = document.getElementById('search-input');

if (createBoardBtn) {
  createBoardBtn.addEventListener('click', createBoard);
}

if (searchInput) {
  searchInput.addEventListener('input', (e) => {
    searchQuery = e.target.value.trim();
    loadBoards();
  });
}

async function loadBoards() {
  try {
    const url = searchQuery 
      ? `/api/boards?search=${encodeURIComponent(searchQuery)}`
      : '/api/boards';
    
    const res = await fetch(url);
    if (!res.ok) throw new Error('Ошибка загрузки');
    const boards = await res.json();
    
    boardsContainer.innerHTML = boards.map(board => `
      <div class="board" data-board-id="${board.id}">
        <div style="display:flex; justify-content:space-between; align-items:center;">
          <div style="display:flex; align-items:center; gap:10px;">
            <h3>${board.title}</h3>
            ${board.is_shared ? '<span style="background:#4CAF50;color:white;padding:2px 8px;border-radius:4px;font-size:12px;">🌐 Общая</span>' : ''}
          </div>
          <div style="display:flex; gap:8px; align-items:center;">
            <span style="font-size:12px;color:#666;" title="Участники">👥 ${board.members.length}</span>
            <button class="btn btn-secondary" onclick="openMembersModal(${board.id})" style="padding:4px 8px;font-size:12px;">👤</button>
            <button class="btn btn-secondary" onclick="deleteBoard(${board.id})" style="padding:4px 8px;font-size:12px;">🗑️</button>
          </div>
        </div>
        <div class="lists-container">
          ${board.lists.map(list => `
            <div class="list" data-list-id="${list.id}">
              <div style="display:flex; justify-content:space-between; align-items:center;">
                <h4>${list.title}</h4>
                <button class="btn btn-secondary" onclick="deleteList(${list.id})" style="padding:2px 6px;font-size:10px;">🗑️</button>
              </div>
              <div class="cards">
                ${list.cards.map(card => `
                  <div class="card" 
                       draggable="true" 
                       data-card-id="${card.id}" 
                       data-list-id="${list.id}"
                       ondblclick="editCard(${card.id}, \`${card.title.replace(/`/g, '\\`')}\`, ${card.content ? `\`${card.content.replace(/`/g, '\\`')}\`` : 'null'})">
                    <div style="display:flex; justify-content:space-between; align-items:start;">
                      <div>
                        <label style="display:flex; align-items:center; gap:6px;">
                          <input type="checkbox" ${card.done ? 'checked' : ''} onchange="toggleCardDone(${card.id}, this.checked)">
                          <strong style="text-decoration:${card.done ? 'line-through' : 'none'}">${card.title}</strong>
                        </label>
                        ${card.content ? `<p>${card.content}</p>` : ''}
                      </div>
                      <button class="btn btn-secondary" onclick="deleteCard(${card.id})" style="padding:2px 6px;font-size:10px;">🗑️</button>
                    </div>
                  </div>
                `).join('')}
                <div class="add-card-placeholder" data-list-id="${list.id}">
                  <button class="btn" onclick="showAddCardForm(${list.id})">➕ Добавить карточку</button>
                </div>
              </div>
            </div>
          `).join('')}
          <div class="list add-list-placeholder" data-board-id="${board.id}">
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

    document.querySelectorAll('.list').forEach(list => {
      list.addEventListener('dragover', handleDragOver);
      list.addEventListener('dragenter', handleDragEnter);
      list.addEventListener('dragleave', handleDragLeave);
      list.addEventListener('drop', handleDrop);
    });
  } catch (e) {
    console.error(e);
    boardsContainer.innerHTML = '<p>Ошибка загрузки данных</p>';
  }
}

// === Board Actions ===
async function createBoard() {
  const title = prompt('Название новой доски:');
  if (!title || title.trim() === '') return;
  
  const isShared = confirm('Нажмите OK, чтобы сделать доску общей (доступной другим пользователям), или Отмена для личной доски.');
  
  try {
    const res = await fetch('/api/boards', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ title: title.trim(), is_shared: isShared })
    });
    if (res.ok) loadBoards(); else alert('Не удалось создать доску');
  } catch (e) { console.error(e); alert('Ошибка подключения'); }
}

async function deleteBoard(boardId) {
  if (!confirm('Удалить доску? Все списки и карточки будут удалены.')) return;
  try {
    const res = await fetch(`/api/boards/${boardId}`, { method: 'DELETE' });
    if (res.ok) loadBoards(); else alert('Не удалось удалить доску');
  } catch (e) { console.error(e); alert('Ошибка подключения'); }
}

// === List Actions ===
function showAddListForm(boardId) {
  const placeholder = document.querySelector(`.add-list-placeholder[data-board-id="${boardId}"]`);
  if (!placeholder) return;
  placeholder.innerHTML = `
    <div class="add-list-form">
      <input type="text" class="add-list-input" placeholder="Название списка" maxlength="50">
      <div class="add-list-btns">
        <button class="btn btn-primary" onclick="createList(${boardId}, this)">Добавить</button>
        <button class="btn btn-secondary" onclick="cancelAddList(${boardId})">Отмена</button>
      </div>
    </div>
  `;
}

async function createList(boardId, button) {
  const input = button.closest('.add-list-form').querySelector('.add-list-input');
  const title = input.value.trim();
  if (!title) { input.focus(); return; }
  try {
    const res = await fetch(`/api/boards/${boardId}/lists`, { method: 'POST', headers: { 'Content-Type': 'application/json' }, body: JSON.stringify({ title }) });
    if (res.ok) loadBoards(); else alert('Не удалось создать список');
  } catch (e) { console.error(e); alert('Ошибка подключения'); }
}

async function deleteList(listId) {
  if (!confirm('Удалить список? Все карточки будут удалены.')) return;
  try {
    const res = await fetch(`/api/lists/${listId}`, { method: 'DELETE' });
    if (res.ok) loadBoards(); else alert('Не удалось удалить список');
  } catch (e) { console.error(e); alert('Ошибка подключения'); }
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
    <div class="add-list-form">
      <input type="text" class="add-list-input" placeholder="Название карточки" maxlength="100">
      <textarea class="add-list-input" placeholder="Описание (необязательно)" rows="2" style="margin-top:5px;"></textarea>
      <div class="add-list-btns">
        <button class="btn btn-primary" onclick="createCard(${listId}, this)">Добавить</button>
        <button class="btn btn-secondary" onclick="cancelAddCard(${listId})">Отмена</button>
      </div>
    </div>
  `;
}

async function createCard(listId, button) {
  const form = button.closest('.add-list-form');
  const titleInput = form.querySelector('input');
  const contentInput = form.querySelector('textarea');
  const title = titleInput.value.trim();
  const content = contentInput.value.trim() || null;
  if (!title) { titleInput.focus(); return; }
  try {
    const res = await fetch(`/api/lists/${listId}/cards`, { method: 'POST', headers: { 'Content-Type': 'application/json' }, body: JSON.stringify({ title, content }) });
    if (res.ok) loadBoards(); else alert('Не удалось создать карточку');
  } catch (e) { console.error(e); alert('Ошибка подключения'); }
}

async function deleteCard(cardId) {
  if (!confirm('Удалить карточку?')) return;
  try {
    const res = await fetch(`/api/cards/${cardId}`, { method: 'DELETE' });
    if (res.ok) loadBoards(); else alert('Не удалось удалить карточку');
  } catch (e) { console.error(e); alert('Ошибка подключения'); }
}

function cancelAddCard(listId) {
  const placeholder = document.querySelector(`.add-card-placeholder[data-list-id="${listId}"]`);
  if (placeholder) {
    placeholder.innerHTML = '<button class="btn" onclick="showAddCardForm(' + listId + ')">➕ Добавить карточку</button>';
  }
}

// === Edit Card ===
function editCard(cardId, title, content) {
  const cardEl = document.querySelector(`.card[data-card-id="${cardId}"]`);
  if (!cardEl) return;
  const originalHTML = cardEl.innerHTML;
  const formDiv = document.createElement('div');
  formDiv.className = 'add-list-form';

  const input = document.createElement('input');
  input.type = 'text';
  input.className = 'add-list-input';
  input.value = title;
  input.maxLength = 100;

  const textarea = document.createElement('textarea');
  textarea.className = 'add-list-input';
  textarea.rows = 2;
  textarea.style.marginTop = '5px';
  textarea.value = content || '';

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
}

async function saveCardEdit(cardId, button) {
  const form = button.closest('.add-list-form');
  const titleInput = form.querySelector('input');
  const contentInput = form.querySelector('textarea');
  const title = titleInput.value.trim();
  const content = contentInput.value.trim() || null;
  if (!title) { titleInput.focus(); return; }
  try {
    const res = await fetch(`/api/cards/${cardId}`, { method: 'PATCH', headers: { 'Content-Type': 'application/json' }, body: JSON.stringify({ title, content }) });
    if (res.ok) loadBoards(); else alert('Не удалось сохранить карточку');
  } catch (e) { console.error(e); alert('Ошибка подключения'); }
}

function cancelCardEdit(cardId, originalHTML) {
  const cardEl = document.querySelector(`.card[data-card-id="${cardId}"]`);
  if (cardEl) cardEl.innerHTML = originalHTML;
}

// === Toggle Done ===
async function toggleCardDone(cardId, done) {
  try {
    const res = await fetch(`/api/cards/${cardId}`, {
      method: 'PATCH',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ done })
    });
    if (!res.ok) {
      alert('Не удалось обновить статус');
      document.querySelector(`.card[data-card-id="${cardId}"] input[type="checkbox"]`).checked = !done;
    } else {
      loadBoards(); // перезагружаем всё для простоты
    }
  } catch (e) {
    console.error(e);
    alert('Ошибка подключения');
    document.querySelector(`.card[data-card-id="${cardId}"] input[type="checkbox"]`).checked = !done;
  }
}

// === Drag-and-Drop ===
function handleDragStart(e) {
  draggedCard = this;
  draggedFromList = this.dataset.listId;
  this.classList.add('dragging');
}
function handleDragEnd() {
  this.classList.remove('dragging');
  draggedCard = null;
  draggedFromList = null;
}
function handleDragOver(e) { e.preventDefault(); }
function handleDragEnter(e) { e.preventDefault(); this.style.backgroundColor = '#ddd'; }
function handleDragLeave() { this.style.backgroundColor = ''; }
async function handleDrop(e) {
  e.preventDefault();
  this.style.backgroundColor = '';
  const targetListId = this.dataset.listId;
  const cardId = draggedCard.dataset.cardId;
  if (targetListId === draggedFromList) return;
  try {
    await fetch(`/api/cards/${cardId}`, {
      method: 'PATCH',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ list_id: parseInt(targetListId) }) // ← исправлено!
    });
    loadBoards();
  } catch (e) {
    console.error(e);
    alert('Ошибка перемещения карточки');
  }
}

// === Init ===
loadBoards();

// === Экспорт функций для onclick ===
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

// === Members Management ===
let currentBoardId = null;

async function openMembersModal(boardId) {
  currentBoardId = boardId;
  const modal = document.getElementById('members-modal');
  const membersList = document.getElementById('members-list');
  
  modal.style.display = 'block';
  membersList.innerHTML = '<p>Загрузка...</p>';
  
  try {
    const res = await fetch(`/api/boards/${boardId}/members`);
    if (!res.ok) throw new Error('Ошибка загрузки');
    const members = await res.json();
    
    if (members.length === 0) {
      membersList.innerHTML = '<p>Нет участников</p>';
    } else {
      membersList.innerHTML = members.map(m => `
        <div style="display:flex;justify-content:space-between;align-items:center;padding:8px;border-bottom:1px solid #eee;">
          <span><strong>${m.username}</strong></span>
          <button class="btn btn-secondary" onclick="removeMember(${m.id})" style="padding:2px 8px;font-size:11px;">Удалить</button>
        </div>
      `).join('');
    }
  } catch (e) {
    console.error(e);
    membersList.innerHTML = '<p>Ошибка загрузки участников</p>';
  }
}

function closeMembersModal() {
  document.getElementById('members-modal').style.display = 'none';
  currentBoardId = null;
}

async function addMember() {
  const usernameInput = document.getElementById('new-member-username');
  const username = usernameInput.value.trim();
  if (!username) {
    alert('Введите имя пользователя');
    return;
  }
  
  try {
    // Сначала ищем или создаём пользователя
    let userRes = await fetch(`/api/users?username=${encodeURIComponent(username)}`);
    let userId;
    
    if (userRes.ok) {
      const users = await userRes.json();
      if (users.length > 0) {
        userId = users[0].id;
      } else {
        // Создаём нового пользователя
        userRes = await fetch('/api/users', {
          method: 'POST',
          headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify({ username })
        });
        if (!userRes.ok) throw new Error('Не удалось создать пользователя');
        const user = await userRes.json();
        userId = user.id;
      }
    } else {
      // Создаём нового пользователя
      userRes = await fetch('/api/users', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ username })
      });
      if (!userRes.ok) throw new Error('Не удалось создать пользователя');
      const user = await userRes.json();
      userId = user.id;
    }
    
    // Добавляем участника на доску
    const addRes = await fetch(`/api/boards/${currentBoardId}/members`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ user_id: userId, role: 'member' })
    });
    
    if (!addRes.ok) throw new Error('Не удалось добавить участника');
    
    usernameInput.value = '';
    openMembersModal(currentBoardId); // Обновить список
    loadBoards(); // Обновить счётчик участников
  } catch (e) {
    console.error(e);
    alert('Ошибка: ' + e.message);
  }
}

async function removeMember(userId) {
  if (!confirm('Удалить участника из доски?')) return;
  
  try {
    const res = await fetch(`/api/boards/${currentBoardId}/members/${userId}`, {
      method: 'DELETE'
    });
    
    if (!res.ok) throw new Error('Не удалось удалить участника');
    
    openMembersModal(currentBoardId);
    loadBoards();
  } catch (e) {
    console.error(e);
    alert('Ошибка: ' + e.message);
  }
}

window.openMembersModal = openMembersModal;
window.closeMembersModal = closeMembersModal;
window.addMember = addMember;
window.removeMember = removeMember;
window.toggleCardDone = toggleCardDone;