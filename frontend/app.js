let draggedCard = null;
let draggedFromList = null;

const boardsContainer = document.getElementById('boards');
const createBoardBtn = document.getElementById('create-board-btn');

if (createBoardBtn) {
  createBoardBtn.addEventListener('click', createBoard);
}

async function loadBoards() {
  try {
    const res = await fetch('/api/boards');
    if (!res.ok) throw new Error('Ошибка загрузки');
    const boards = await res.json();
    
    boardsContainer.innerHTML = boards.map(board => `
      <div class="board" data-board-id="${board.id}">
        <div style="display:flex; justify-content:space-between; align-items:center;">
          <h3>${board.title}</h3>
          <button class="btn btn-secondary" onclick="deleteBoard(${board.id})" style="padding:4px 8px;font-size:12px;">🗑️</button>
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
                        <strong>${card.title}</strong>
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
  try {
    const res = await fetch('/api/boards', { method: 'POST', headers: { 'Content-Type': 'application/json' }, body: JSON.stringify({ title: title.trim() }) });
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

  // Сохраняем оригинальный HTML (без изменений)
  const originalHTML = cardEl.innerHTML;

  // Создаём новую форму через DOM
  const formDiv = document.createElement('div');
  formDiv.className = 'add-list-form';

  // Input
  const input = document.createElement('input');
  input.type = 'text';
  input.className = 'add-list-input';
  input.value = title;
  input.maxLength = 100;

  // Textarea
  const textarea = document.createElement('textarea');
  textarea.className = 'add-list-input';
  textarea.rows = 2;
  textarea.style.marginTop = '5px';
  textarea.value = content || '';

  // Кнопки
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

  // Собираем форму
  formDiv.appendChild(input);
  formDiv.appendChild(textarea);
  formDiv.appendChild(btnsDiv);

  // Заменяем содержимое карточки
  cardEl.innerHTML = '';
  cardEl.appendChild(formDiv);

  // Фокус на поле ввода
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
  if (cardEl) {
    // Просто вставляем оригинальный HTML
    cardEl.innerHTML = originalHTML;
    // Восстанавливаем обработчик двойного клика
    cardEl.ondblclick = () => editCard(cardId, cardEl.querySelector('strong').textContent, 
                                      cardEl.querySelector('p')?.textContent || null);
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
    await fetch(`/api/cards/${cardId}`, { method: 'PATCH', headers: { 'Content-Type': 'application/json' }, body: JSON.stringify({ list_id: parseInt(targetListId), title: "" }) });
    loadBoards();
  } catch (e) { console.error(e); alert('Ошибка перемещения карточки'); }
}

// === Init ===
loadBoards();

// === Make functions global for onclick ===
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