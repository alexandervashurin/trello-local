// frontend/js/modules/sessions.js
// === Sessions Management ===

import { apiRequest } from './api.js';
import { showToast } from './toast.js';
import { formatDateTime } from './utils.js';

export async function loadSessions() {
  try {
    const sessions = await apiRequest('/api/sessions');
    renderSessions(sessions);
  } catch (error) {
    console.error(error);
    showToast('Ошибка загрузки сессий', 'error');
  }
}

function renderSessions(sessions) {
  const container = document.getElementById('sessions-list');
  if (!container) return;
  
  if (sessions.length === 0) {
    container.innerHTML = '<div class="empty-state">Нет активных сессий</div>';
    return;
  }
  
  container.innerHTML = `
    <table class="sessions-table">
      <thead>
        <tr>
          <th>Устройство</th>
          <th>IP адрес</th>
          <th>Создана</th>
          <th>Последняя активность</th>
          <th>Действия</th>
        </tr>
      </thead>
      <tbody>
        ${sessions.map(s => `
          <tr class="${s.is_current ? 'current-session' : ''}">
            <td>${s.user_agent || 'Неизвестно'}</td>
            <td>${s.ip_address || 'Неизвестно'}</td>
            <td>${formatDateTime(s.created_at)}</td>
            <td>${formatDateTime(s.last_activity)}</td>
            <td>
              ${!s.is_current ? `<button class="btn btn-sm btn-danger" onclick="window.deleteSession(${s.id})">Завершить</button>` : '<span class="badge badge-success">Текущая</span>'}
            </td>
          </tr>
        `).join('')}
      </tbody>
    </table>
  `;
}

export async function deleteSession(sessionId) {
  if (!confirm('Завершить эту сессию?')) return;
  
  try {
    await apiRequest(`/api/sessions/${sessionId}`, { method: 'DELETE' });
    showToast('Сессия завершена', 'success');
    loadSessions();
  } catch (error) {
    console.error(error);
    showToast('Ошибка завершения сессии', 'error');
  }
}

export async function deleteAllSessions() {
  if (!confirm('Завершить все сессии кроме текущей?')) return;
  
  try {
    await apiRequest('/api/sessions', { method: 'DELETE' });
    showToast('Все сессии завершены', 'success');
    loadSessions();
  } catch (error) {
    console.error(error);
    showToast('Ошибка завершения сессий', 'error');
  }
}
