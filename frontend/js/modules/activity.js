// frontend/js/modules/activity.js
// === Activity Log Functions ===

import { apiRequest } from './api.js';
import { showToast } from './toast.js';
import { escapeHtml, formatDateTime, formatRelativeTime } from './utils.js';

export async function loadActivityLog(boardId) {
  try {
    const activities = await apiRequest(`/api/boards/${boardId}/activity`);
    return activities;
  } catch (error) {
    console.error(error);
    return [];
  }
}

export async function openActivityModal(boardId) {
  const modal = document.getElementById('activity-modal');
  const content = document.getElementById('activity-list');
  
  if (!modal || !content) return;
  
  modal.classList.add('open');
  content.innerHTML = '<div class="loading">Загрузка активности...</div>';
  
  try {
    const activities = await loadActivityLog(boardId);
    
    content.innerHTML = `
      <div class="activity-list">
        ${activities.length === 0 ? '<p class="empty">Нет записей активности</p>' : ''}
        ${activities.map(a => `
          <div class="activity-item">
            <div class="activity-avatar">${a.username?.charAt(0).toUpperCase() || '?'}</div>
            <div class="activity-content">
              <div class="activity-text">${escapeHtml(a.description)}</div>
              <div class="activity-meta">
                <span>${escapeHtml(a.username || 'Неизвестно')}</span>
                <span>•</span>
                <span>${formatRelativeTime(a.created_at)}</span>
              </div>
            </div>
          </div>
        `).join('')}
      </div>
    `;
  } catch (error) {
    console.error(error);
    content.innerHTML = '<div class="empty-state">Ошибка загрузки активности</div>';
    showToast('Не удалось загрузить активность', 'error');
  }
}

export function closeActivityModal() {
  const modal = document.getElementById('activity-modal');
  if (modal) {
    modal.classList.remove('open');
  }
}

export function renderActivityLog(activities, containerId) {
  const container = document.getElementById(containerId);
  if (!container) return;
  
  if (activities.length === 0) {
    container.innerHTML = '<p class="empty">Нет записей активности</p>';
    return;
  }
  
  container.innerHTML = activities.map(a => `
    <div class="activity-item">
      <div class="activity-avatar">${a.username?.charAt(0).toUpperCase() || '?'}</div>
      <div class="activity-content">
        <div class="activity-text">${escapeHtml(a.description)}</div>
        <div class="activity-meta">
          <span>${escapeHtml(a.username || 'Неизвестно')}</span>
          <span>•</span>
          <span>${formatRelativeTime(a.created_at)}</span>
        </div>
      </div>
    </div>
  `).join('');
}
