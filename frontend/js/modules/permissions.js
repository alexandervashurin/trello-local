// frontend/js/modules/permissions.js
// === Granular Permissions Management ===

import { apiRequest } from './api.js';
import { showToast } from './toast.js';
import { escapeHtml } from './utils.js';

const PERMISSION_LABELS = {
  can_view: 'Просмотр доски',
  can_create_cards: 'Создание карточек',
  can_edit_cards: 'Редактирование карточек',
  can_delete_cards: 'Удаление карточек',
  can_move_cards: 'Перемещение карточек',
  can_create_lists: 'Создание списков',
  can_edit_lists: 'Редактирование списков',
  can_delete_lists: 'Удаление списков',
  can_manage_members: 'Управление участниками',
  can_manage_settings: 'Управление настройками доски'
};

const DEFAULT_ROLES = ['owner', 'admin', 'member', 'viewer'];

export async function openPermissionsModal(boardId) {
  const modal = document.getElementById('permissions-modal');
  const content = document.getElementById('permissions-content');
  
  if (!modal || !content) return;
  
  modal.classList.add('open');
  content.innerHTML = '<div class="loading">Загрузка прав...</div>';
  
  try {
    const permissions = await apiRequest(`/api/boards/${boardId}/permissions`);
    
    content.innerHTML = `
      <div class="permissions-info">
        <h3>🔐 Гранулярные права доступа</h3>
        <p style="color:#6b778c;font-size:14px;margin-bottom:20px;">
          Настройте права доступа для каждой роли на этой доске
        </p>
      </div>
      
      <div class="roles-list">
        ${DEFAULT_ROLES.map(role => {
          const rolePerms = permissions.find(p => p.role === role);
          return renderRolePermissions(boardId, role, rolePerms);
        }).join('')}
      </div>
    `;
  } catch (error) {
    console.error(error);
    content.innerHTML = '<div class="empty-state">Ошибка загрузки прав</div>';
    showToast('Не удалось загрузить права', 'error');
  }
}

function renderRolePermissions(boardId, role, perms) {
  const roleNames = {
    owner: '👑 Владелец',
    admin: '🔧 Администратор',
    member: '👤 Участник',
    viewer: '👁️ Наблюдатель'
  };
  
  const roleDescriptions = {
    owner: 'Полный доступ ко всем функциям',
    admin: 'Почти полный доступ, кроме управления настройками доски',
    member: 'Может создавать карточки, но не может редактировать чужие',
    viewer: 'Только просмотр без права редактирования'
  };
  
  if (!perms) {
    perms = {
      can_view: true,
      can_create_cards: false,
      can_edit_cards: false,
      can_delete_cards: false,
      can_move_cards: false,
      can_create_lists: false,
      can_edit_lists: false,
      can_delete_lists: false,
      can_manage_members: false,
      can_manage_settings: false
    };
  }
  
  return `
    <div class="role-permissions" data-role="${role}">
      <div class="role-header">
        <h4>${roleNames[role] || role}</h4>
        <p class="role-description">${roleDescriptions[role] || ''}</p>
      </div>
      
      <div class="permissions-grid">
        ${Object.entries(PERMISSION_LABELS).map(([key, label]) => `
          <label class="permission-checkbox">
            <input type="checkbox" 
                   data-permission="${key}" 
                   ${perms[key] ? 'checked' : ''} 
                   onchange="window.updatePermission(${boardId}, '${role}', '${key}', this.checked)"
                   ${role === 'owner' ? 'disabled checked' : ''}>
            <span>${label}</span>
          </label>
        `).join('')}
      </div>
    </div>
  `;
}

export function closePermissionsModal() {
  const modal = document.getElementById('permissions-modal');
  if (modal) {
    modal.classList.remove('open');
  }
}

export async function updatePermission(boardId, role, permission, value) {
  try {
    await apiRequest(`/api/boards/${boardId}/permissions/${role}`, {
      method: 'PATCH',
      body: JSON.stringify({ [permission]: value })
    });
    
    showToast('Права обновлены', 'success');
  } catch (error) {
    console.error(error);
    showToast(error.message || 'Ошибка обновления прав', 'error');
  }
}

export async function loadBoardPermissions(boardId) {
  try {
    const permissions = await apiRequest(`/api/boards/${boardId}/permissions`);
    return permissions;
  } catch (error) {
    console.error(error);
    return [];
  }
}

export async function checkPermission(boardId, permission) {
  try {
    const user = window.getUser();
    if (!user) return false;
    
    const board = await apiRequest(`/api/boards/${boardId}`);
    if (board.owner_id === user.id) return true; // Владелец всегда имеет все права
    
    const permissions = await loadBoardPermissions(boardId);
    const userBoard = await apiRequest(`/api/boards/${boardId}/members/${user.id}`);
    
    const rolePerms = permissions.find(p => p.role === userBoard.role);
    return rolePerms ? rolePerms[permission] : false;
  } catch (error) {
    console.error(error);
    return false;
  }
}

export function renderPermissionsSettings(boardId) {
  const container = document.getElementById('board-permissions-settings');
  if (!container) return;
  
  container.innerHTML = `
    <div class="permissions-section">
      <h4>🔐 Права доступа</h4>
      <button class="btn btn-secondary" onclick="window.openPermissionsModal(${boardId})">
        Настроить права
      </button>
    </div>
  `;
}
