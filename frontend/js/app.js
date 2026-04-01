// frontend/js/app.js
// Точка входа - импортирует модули и экспортирует функции в глобальную область

import { initTheme, toggleTheme } from './modules/theme.js';
import { showToast } from './modules/toast.js';
import { showLoading, hideLoading } from './modules/loading.js';
import { apiRequest, getToken, getUser, isAuthenticated, logout } from './modules/api.js';
import { 
  draggedCard, draggedFromList, searchQuery, currentBoardId, currentCardId, currentCardData, isLoading,
  selectedCards, isBulkMode,
  setDraggedCard, setDraggedFromList, setSearchQuery, setCurrentBoardId, setCurrentCardId, setCurrentCardData, setIsLoading,
  toggleBulkMode, toggleCardSelection, clearCardSelection, updateBulkModeUI
} from './modules/state.js';
import { escapeHtml, formatDate, formatDateTime, formatRelativeTime, getInitials, getDueDateClass, getDueDateText } from './modules/utils.js';
import { handleDragStart, handleDragOver, handleDrop, handleDragEnd } from './modules/drag-drop.js';
import { startNotificationPolling, stopNotificationPolling, checkUnreadNotifications, loadNotifications, markAllNotificationsRead } from './modules/notifications.js';
import { openProfileModal, closeProfileModal, saveProfile, openChangePassword, openDeleteAccount, setup2FA, confirm2FAEnable, disable2FA } from './modules/profile.js';
import { loadSessions, deleteSession, deleteAllSessions, logoutAllSessions, openSessionsModal, closeSessionsModal } from './modules/sessions.js';
import { exportBoardToJson, exportBoardToCsv, getBoardStats, closeBoardStats } from './modules/export.js';
import { toggleBulkModeFromModule, toggleCardSelectionFromModule, bulkMoveCards, bulkUpdateCards, bulkDeleteCards, bulkMarkDone, bulkMarkTodo } from './modules/bulk-ops.js';
import { 
  loadBoards, renderBoards, createBoard, openBoard, deleteBoard, 
  loadBoardDetails, loadBoardLists, createList, deleteList, createCard,
  openCard, showCardModal, saveCardFromModal, closeCardModal
} from './modules/boards.js';
import { initCalendar, renderCalendar, previousMonth, nextMonth, goToToday, selectCalendarDay, openCalendarModal, closeCalendarModal } from './modules/calendar.js';
import { openMembersModal, closeMembersModal, addMember, changeMemberRole, removeMember } from './modules/members.js';
import { openInvitationsModal, closeInvitationsModal, createInvitation, copyInviteLink, deleteInvitation } from './modules/members.js';
import { openCommentsModal, closeCommentsModal, addComment, deleteComment, editComment } from './modules/comments.js';
import { openLabelsModal, closeLabelsModal, selectLabelColor, addLabel, deleteLabel, openLabelFilter, closeLabelFilter, applyLabelFilter } from './modules/labels.js';
import { createChecklist, deleteChecklist, addChecklistItem, toggleChecklistItem, deleteChecklistItem, renderChecklists } from './modules/checklists.js';
import { openActivityModal, closeActivityModal, loadActivityLog, renderActivityLog } from './modules/activity.js';
import { openAttachmentsModal, closeAttachmentsModal, closeImagePreview, openImagePreview, uploadAttachment, deleteAttachment, renderAttachments } from './modules/attachments.js';
import { openPermissionsModal, closePermissionsModal, updatePermission, loadBoardPermissions, checkPermission, renderPermissionsSettings } from './modules/permissions.js';

// Экспорт в глобальную область для HTML onclick handlers
window.toggleTheme = toggleTheme;
window.showToast = showToast;
window.showLoading = showLoading;
window.hideLoading = hideLoading;
window.apiRequest = apiRequest;
window.getToken = getToken;
window.getUser = getUser;
window.isAuthenticated = isAuthenticated;
window.logout = logout;

window.draggedCard = draggedCard;
window.draggedFromList = draggedFromList;
window.searchQuery = searchQuery;
window.currentBoardId = currentBoardId;
window.currentCardId = currentCardId;
window.currentCardData = currentCardData;
window.isLoading = isLoading;
window.selectedCards = selectedCards;
window.isBulkMode = isBulkMode;

window.setDraggedCard = setDraggedCard;
window.setDraggedFromList = setDraggedFromList;
window.setSearchQuery = setSearchQuery;
window.setCurrentBoardId = setCurrentBoardId;
window.setCurrentCardId = setCurrentCardId;
window.setCurrentCardData = setCurrentCardData;
window.setIsLoading = setIsLoading;

window.toggleBulkMode = toggleBulkModeFromModule;
window.toggleCardSelection = toggleCardSelectionFromModule;
window.clearCardSelection = clearCardSelection;
window.updateBulkModeUI = updateBulkModeUI;
window.bulkMoveCards = bulkMoveCards;
window.bulkUpdateCards = bulkUpdateCards;
window.bulkDeleteCards = bulkDeleteCards;

window.escapeHtml = escapeHtml;
window.formatDate = formatDate;
window.formatDateTime = formatDateTime;
window.formatRelativeTime = formatRelativeTime;
window.getInitials = getInitials;
window.getDueDateClass = getDueDateClass;
window.getDueDateText = getDueDateText;

window.handleDragStart = handleDragStart;
window.handleDragOver = handleDragOver;
window.handleDrop = handleDrop;
window.handleDragEnd = handleDragEnd;

window.startNotificationPolling = startNotificationPolling;
window.stopNotificationPolling = stopNotificationPolling;
window.checkUnreadNotifications = checkUnreadNotifications;
window.loadNotifications = loadNotifications;
window.markAllNotificationsRead = markAllNotificationsRead;

window.openProfileModal = openProfileModal;
window.closeProfileModal = closeProfileModal;
window.saveProfile = saveProfile;
window.openChangePassword = openChangePassword;
window.openDeleteAccount = openDeleteAccount;
window.setup2FA = setup2FA;
window.confirm2FAEnable = confirm2FAEnable;
window.disable2FA = disable2FA;

window.loadSessions = loadSessions;
window.deleteSession = deleteSession;
window.deleteAllSessions = deleteAllSessions;

window.exportBoardToJson = exportBoardToJson;
window.exportBoardToCsv = exportBoardToCsv;
window.getBoardStats = getBoardStats;
window.closeBoardStats = closeBoardStats;
window.openSessionsModal = openSessionsModal;
window.closeSessionsModal = closeSessionsModal;
window.logoutAllSessions = logoutAllSessions;

window.loadBoards = loadBoards;
window.renderBoards = renderBoards;
window.createBoard = createBoard;
window.openBoard = openBoard;
window.deleteBoard = deleteBoard;
window.loadBoardDetails = loadBoardDetails;
window.loadBoardLists = loadBoardLists;
window.createList = createList;
window.deleteList = deleteList;
window.createCard = createCard;
window.openCard = openCard;
window.showCardModal = showCardModal;
window.saveCardFromModal = saveCardFromModal;
window.closeCardModal = closeCardModal;

window.initCalendar = initCalendar;
window.renderCalendar = renderCalendar;
window.previousMonth = previousMonth;
window.nextMonth = nextMonth;
window.goToToday = goToToday;
window.selectCalendarDay = selectCalendarDay;
window.openCalendarModal = openCalendarModal;
window.closeCalendarModal = closeCalendarModal;
window.closeNotificationsModal = closeNotificationsModal;
window.openNotificationsModal = openNotificationsModal;
window.closeMembersModal = closeMembersModal;
window.closeInvitationsModal = closeInvitationsModal;
window.closeCommentsModal = closeCommentsModal;
window.closeActivityModal = closeActivityModal;
window.closeImagePreview = closeImagePreview;
window.closeLabelFilter = closeLabelFilter;
window.openMembersModal = openMembersModal;
window.closeMembersModal = closeMembersModal;
window.addMember = addMember;
window.changeMemberRole = changeMemberRole;
window.removeMember = removeMember;

window.openInvitationsModal = openInvitationsModal;
window.closeInvitationsModal = closeInvitationsModal;
window.createInvitation = createInvitation;
window.copyInviteLink = copyInviteLink;
window.deleteInvitation = deleteInvitation;

window.openCommentsModal = openCommentsModal;
window.closeCommentsModal = closeCommentsModal;
window.addComment = addComment;
window.deleteComment = deleteComment;
window.editComment = editComment;

window.openLabelsModal = openLabelsModal;
window.closeLabelsModal = closeLabelsModal;
window.selectLabelColor = selectLabelColor;
window.addLabel = addLabel;
window.deleteLabel = deleteLabel;
window.openLabelFilter = openLabelFilter;
window.closeLabelFilter = closeLabelFilter;
window.applyLabelFilter = applyLabelFilter;

window.createChecklist = createChecklist;
window.deleteChecklist = deleteChecklist;
window.addChecklistItem = addChecklistItem;
window.toggleChecklistItem = toggleChecklistItem;
window.deleteChecklistItem = deleteChecklistItem;

window.openActivityModal = openActivityModal;
window.closeActivityModal = closeActivityModal;

window.openAttachmentsModal = openAttachmentsModal;
window.closeAttachmentsModal = closeAttachmentsModal;
window.closeImagePreview = closeImagePreview;
window.openImagePreview = openImagePreview;
window.uploadAttachment = uploadAttachment;
window.deleteAttachment = deleteAttachment;

window.openPermissionsModal = openPermissionsModal;
window.closePermissionsModal = closePermissionsModal;
window.updatePermission = updatePermission;
window.loadBoardPermissions = loadBoardPermissions;
window.checkPermission = checkPermission;
window.renderPermissionsSettings = renderPermissionsSettings;

window.bulkMarkDone = bulkMarkDone;
window.bulkMarkTodo = bulkMarkTodo;
window.clearDueDate = clearDueDate;

function clearDueDate() { 
  const input = document.getElementById('card-due-date');
  if (input) input.value = ''; 
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
    setSearchQuery(e.target.value.trim());
    loadBoards();
  });
}

// === Init ===
(function init() {
  initTheme();
  
  if (isAuthenticated()) {
    const notificationsBtn = document.getElementById('notifications-btn');
    if (notificationsBtn) {
      notificationsBtn.style.display = 'inline-block';
    }
    startNotificationPolling();
  }
  
  loadBoards();
})();

// === Load Boards ===
async function loadBoards() {
  if (!isAuthenticated()) return;
  
  showLoading();
  
  try {
    const boards = await apiRequest('/api/boards');
    renderBoards(boards);
  } catch (error) {
    console.error(error);
    showToast('Ошибка загрузки досок', 'error');
  } finally {
    hideLoading();
  }
}

function renderBoards(boards) {
  if (!boardsContainer) return;
  
  if (boards.length === 0) {
    boardsContainer.innerHTML = `
      <div class="empty-state">
        <p>Нет досок</p>
        <button class="btn btn-primary" onclick="createBoard()">Создать первую доску</button>
      </div>
    `;
    return;
  }
  
  boardsContainer.innerHTML = boards.map(board => `
    <div class="board-card" data-board-id="${board.id}">
      <div class="board-header">
        <h3>${escapeHtml(board.title)}</h3>
        <div class="board-actions">
          <button class="btn btn-sm" onclick="openBoard(${board.id})">Открыть</button>
          <button class="btn btn-sm btn-danger" onclick="deleteBoard(${board.id})">Удалить</button>
        </div>
      </div>
      <div class="board-meta">
        <span class="badge">${board.visibility === 'public' ? 'Публичная' : 'Приватная'}</span>
        ${board.is_shared ? '<span class="badge badge-info">Общая</span>' : ''}
      </div>
    </div>
  `).join('');
}

// === Board Actions ===
async function createBoard() {
  const title = prompt('Введите название доски:');
  if (!title) return;
  
  try {
    const board = await apiRequest('/api/boards', {
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

async function openBoard(boardId) {
  setCurrentBoardId(boardId);
  window.location.href = `/?board=${boardId}`;
}

async function deleteBoard(boardId) {
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
