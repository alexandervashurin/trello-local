// frontend/js/modules/utils.js
// === Utility Functions ===

export function escapeHtml(text) {
  const div = document.createElement('div');
  div.textContent = text;
  return div.innerHTML;
}

export function formatDate(timestamp) {
  if (!timestamp) return '';
  return new Date(timestamp * 1000).toLocaleDateString('ru-RU');
}

export function formatDateTime(timestamp) {
  if (!timestamp) return '';
  return new Date(timestamp * 1000).toLocaleString('ru-RU');
}

export function formatRelativeTime(timestamp) {
  if (!timestamp) return '';
  
  const now = Date.now();
  const date = new Date(timestamp * 1000);
  const diff = now - date.getTime();
  
  const minutes = Math.floor(diff / 60000);
  const hours = Math.floor(diff / 3600000);
  const days = Math.floor(diff / 86400000);
  
  if (minutes < 1) return 'только что';
  if (minutes < 60) return `${minutes} мин. назад`;
  if (hours < 24) return `${hours} ч. назад`;
  if (days < 7) return `${days} дн. назад`;
  
  return formatDate(timestamp);
}

export function getInitials(username) {
  if (!username) return '?';
  return username.charAt(0).toUpperCase();
}

// === Helper Functions for Due Date ===
export function getDueDateClass(dueDate, done) {
  if (!dueDate) return '';
  if (done) return 'due-done';
  
  const now = new Date();
  const due = new Date(dueDate * 1000);
  const diff = due - now;
  const days = Math.floor(diff / (1000 * 60 * 60 * 24));
  
  if (diff < 0) return 'due-overdue';
  if (days <= 1) return 'due-today';
  if (days <= 3) return 'due-soon';
  
  return '';
}

export function getDueDateText(dueDate) {
  if (!dueDate) return '';
  
  const now = new Date();
  const due = new Date(dueDate * 1000);
  const diff = due - now;
  const days = Math.floor(diff / (1000 * 60 * 60 * 24));
  
  if (diff < 0) return `Просрочено на ${Math.abs(days)} дн.`;
  if (days === 0) return 'Сегодня';
  if (days === 1) return 'Завтра';
  if (days <= 7) return `Через ${days} дн.`;
  
  return formatDate(dueDate);
}
