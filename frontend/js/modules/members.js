// frontend/js/modules/members.js
// === Members and Invitations Management ===

import { apiRequest } from './api.js';
import { showToast } from './toast.js';
import { escapeHtml } from './utils.js';

export async function openMembersModal(boardId) {
  const modal = document.getElementById('members-modal');
  const content = document.getElementById('members-list');
  
  if (!modal || !content) return;
  
  modal.classList.add('open');
  content.innerHTML = '<div class="loading">Загрузка участников...</div>';
  
  try {
    const members = await apiRequest(`/api/boards/${boardId}/members`);
    const users = await apiRequest('/api/users');
    
    content.innerHTML = `
      <div class="members-section">
        <h3>Текущие участники</h3>
        <div class="members-list">
          ${members.map(m => `
            <div class="member-item">
              <span class="avatar" style="background:${m.avatar_color || '#0079bf'}">${m.username.charAt(0).toUpperCase()}</span>
              <span>${escapeHtml(m.username)}</span>
              <span class="badge">${getRoleName(m.role)}</span>
              ${m.role !== 'owner' ? `
                <select onchange="window.changeMemberRole(${m.user_id}, this.value)" style="margin-left:auto;">
                  <option value="member" ${m.role === 'member' ? 'selected' : ''}>Участник</option>
                  <option value="admin" ${m.role === 'admin' ? 'selected' : ''}>Админ</option>
                </select>
                <button class="btn btn-sm btn-danger" onclick="window.removeMember(${m.user_id})">Удалить</button>
              ` : '<span class="badge badge-info">Владелец</span>'}
            </div>
          `).join('')}
        </div>
      </div>
      
      <div class="members-section">
        <h3>Добавить участника</h3>
        <select id="add-member-select" style="width:100%;padding:8px;margin-bottom:8px;">
          <option value="">Выберите пользователя...</option>
          ${users.filter(u => !members.find(m => m.user_id === u.id)).map(u => `
            <option value="${u.id}">${escapeHtml(u.username)}</option>
          `).join('')}
        </select>
        <select id="add-member-role" style="width:100%;padding:8px;margin-bottom:8px;">
          <option value="member">Участник</option>
          <option value="admin">Админ</option>
        </select>
        <button class="btn btn-primary" onclick="window.addMember(${boardId})">Добавить</button>
      </div>
    `;
  } catch (error) {
    console.error(error);
    content.innerHTML = '<div class="empty-state">Ошибка загрузки участников</div>';
    showToast('Не удалось загрузить участников', 'error');
  }
}

export function closeMembersModal() {
  const modal = document.getElementById('members-modal');
  if (modal) {
    modal.classList.remove('open');
  }
}

export async function addMember(boardId) {
  const userId = document.getElementById('add-member-select')?.value;
  const role = document.getElementById('add-member-role')?.value;
  
  if (!userId) {
    showToast('Выберите пользователя', 'error');
    return;
  }
  
  try {
    await apiRequest(`/api/boards/${boardId}/members`, {
      method: 'POST',
      body: JSON.stringify({ user_id: parseInt(userId), role })
    });
    
    showToast('Участник добавлен', 'success');
    openMembersModal(boardId);
  } catch (error) {
    console.error(error);
    showToast(error.message || 'Ошибка добавления участника', 'error');
  }
}

export async function changeMemberRole(userId, newRole) {
  const boardId = window.currentBoardId;
  if (!boardId) return;
  
  try {
    await apiRequest(`/api/boards/${boardId}/members/${userId}`, {
      method: 'PATCH',
      body: JSON.stringify({ role: newRole })
    });
    
    showToast('Роль изменена', 'success');
  } catch (error) {
    console.error(error);
    showToast('Ошибка изменения роли', 'error');
  }
}

export async function removeMember(userId) {
  if (!confirm('Удалить этого участника из доски?')) return;
  
  const boardId = window.currentBoardId;
  if (!boardId) return;
  
  try {
    await apiRequest(`/api/boards/${boardId}/members/${userId}`, { method: 'DELETE' });
    showToast('Участник удалён', 'success');
    openMembersModal(boardId);
  } catch (error) {
    console.error(error);
    showToast('Ошибка удаления участника', 'error');
  }
}

function getRoleName(role) {
  const roles = { owner: 'Владелец', admin: 'Админ', member: 'Участник' };
  return roles[role] || role;
}

// === Invitations ===

export async function openInvitationsModal(boardId) {
  const modal = document.getElementById('invitations-modal');
  const content = document.getElementById('invitations-list');
  
  if (!modal || !content) return;
  
  modal.classList.add('open');
  content.innerHTML = '<div class="loading">Загрузка приглашений...</div>';
  
  try {
    const invitations = await apiRequest(`/api/boards/${boardId}/invitations`);
    
    content.innerHTML = `
      <div class="invitations-section">
        <h3>Активные приглашения</h3>
        <div class="invitations-list">
          ${invitations.map(inv => `
            <div class="invitation-item">
              <span>Ссылка: <code>${inv.token}</code></span>
              <span>Роль: ${getRoleName(inv.role)}</span>
              <span>Создано: ${new Date(inv.created_at * 1000).toLocaleString('ru-RU')}</span>
              <button class="btn btn-sm" onclick="window.copyInviteLink('${inv.token}')">Копировать</button>
              <button class="btn btn-sm btn-danger" onclick="window.deleteInvitation(${boardId}, '${inv.token}')">Отозвать</button>
            </div>
          `).join('')}
        </div>
      </div>
      
      <div class="invitations-section">
        <h3>Создать приглашение</h3>
        <select id="invite-role" style="width:100%;padding:8px;margin-bottom:8px;">
          <option value="member">Участник</option>
          <option value="admin">Админ</option>
        </select>
        <button class="btn btn-primary" onclick="window.createInvitation(${boardId})">Создать ссылку</button>
      </div>
    `;
  } catch (error) {
    console.error(error);
    content.innerHTML = '<div class="empty-state">Ошибка загрузки приглашений</div>';
    showToast('Не удалось загрузить приглашения', 'error');
  }
}

export function closeInvitationsModal() {
  const modal = document.getElementById('invitations-modal');
  if (modal) {
    modal.classList.remove('open');
  }
}

export async function createInvitation(boardId) {
  const role = document.getElementById('invite-role')?.value;
  
  try {
    const invitation = await apiRequest(`/api/boards/${boardId}/invitations`, {
      method: 'POST',
      body: JSON.stringify({ role })
    });
    
    showToast('Приглашение создано', 'success');
    openInvitationsModal(boardId);
  } catch (error) {
    console.error(error);
    showToast('Ошибка создания приглашения', 'error');
  }
}

export async function copyInviteLink(token) {
  const link = `${window.location.origin}/invite/${token}`;
  
  try {
    await navigator.clipboard.writeText(link);
    showToast('Ссылка скопирована', 'success');
  } catch (error) {
    showToast('Ошибка копирования', 'error');
  }
}

export async function deleteInvitation(boardId, token) {
  if (!confirm('Отозвать это приглашение?')) return;
  
  try {
    await apiRequest(`/api/boards/${boardId}/invitations/${token}`, { method: 'DELETE' });
    showToast('Приглашение отозвано', 'success');
    openInvitationsModal(boardId);
  } catch (error) {
    console.error(error);
    showToast('Ошибка отзыва приглашения', 'error');
  }
}
