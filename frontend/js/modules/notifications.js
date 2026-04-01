// frontend/js/modules/notifications.js
// === Уведомления ===

import { apiRequest } from './api.js';
import { showToast } from './toast.js';

let notificationPollingInterval = null;

export function startNotificationPolling() {
  checkUnreadNotifications();
  notificationPollingInterval = setInterval(checkUnreadNotifications, 10000);
}

export function stopNotificationPolling() {
  if (notificationPollingInterval) {
    clearInterval(notificationPollingInterval);
    notificationPollingInterval = null;
  }
}

export async function checkUnreadNotifications() {
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

export async function loadNotifications() {
  try {
    const notifications = await apiRequest('/api/notifications');
    renderNotifications(notifications);
  } catch (error) {
    console.error(error);
    showToast('Ошибка загрузки уведомлений', 'error');
  }
}

function renderNotifications(notifications) {
  const container = document.getElementById('notifications-list');
  if (!container) return;
  
  if (notifications.length === 0) {
    container.innerHTML = '<div class="empty-state">Нет уведомлений</div>';
    return;
  }
  
  container.innerHTML = notifications.map(n => `
    <div class="notification-item ${n.is_read ? 'read' : 'unread'}" data-id="${n.id}">
      <div class="notification-title">${escapeHtml(n.title)}</div>
      <div class="notification-message">${escapeHtml(n.message)}</div>
      <div class="notification-time">${formatDateTime(n.created_at)}</div>
    </div>
  `).join('');
}

export async function markAllNotificationsRead() {
  try {
    await apiRequest('/api/notifications/read-all', { method: 'POST' });
    showToast('Все уведомления отмечены как прочитанные', 'success');
    checkUnreadNotifications();
    loadNotifications();
  } catch (error) {
    console.error(error);
    showToast('Ошибка отметки уведомлений', 'error');
  }
}

function escapeHtml(text) {
  const div = document.createElement('div');
  div.textContent = text;
  return div.innerHTML;
}

function formatDateTime(timestamp) {
  return new Date(timestamp * 1000).toLocaleString('ru-RU');
}
