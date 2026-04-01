// frontend/js/modules/backup.js
// === Backup Management ===

import { apiRequest } from './api.js';
import { showToast } from './toast.js';
import { formatDateTime } from './utils.js';

export async function openBackupModal() {
  const modal = document.getElementById('backup-modal');
  const content = document.getElementById('backup-content');
  
  if (!modal || !content) return;
  
  modal.classList.add('open');
  content.innerHTML = '<div class="loading">Загрузка backup...</div>';
  
  try {
    const backups = await apiRequest('/api/backup');
    
    content.innerHTML = `
      <div class="backup-container">
        <div class="backup-header">
          <h2>💾 Резервное копирование</h2>
        </div>
        
        <div class="create-backup-form">
          <input type="text" id="backup-description" placeholder="Описание backup (опционально)">
          <button class="btn btn-primary" onclick="window.createBackup()">Создать backup</button>
        </div>
        
        <div class="auto-backup-section">
          <h3>🔄 Автоматическое резервное копирование</h3>
          <div class="auto-backup-toggle">
            <input type="checkbox" id="auto-backup" onchange="window.toggleAutoBackup()">
            <label for="auto-backup">Включить автоматическое создание backup ежедневно</label>
          </div>
        </div>
        
        <div class="backup-list">
          ${backups.length === 0 ? `
            <div class="backup-empty">
              <div class="backup-empty-icon">📦</div>
              <p>Нет резервных копий</p>
            </div>
          ` : ''}
          ${backups.map(b => `
            <div class="backup-item" data-backup-id="${b.id}">
              <div class="backup-icon">💿</div>
              <div class="backup-info">
                <div class="backup-filename">${escapeHtml(b.filename)}</div>
                <div class="backup-meta">
                  <span>📅 ${formatDateTime(b.created_at)}</span>
                  <span>👤 ${escapeHtml(b.creator_username)}</span>
                  <span>💾 ${formatFileSize(b.file_size)}</span>
                  ${b.description ? `<span>📝 ${escapeHtml(b.description)}</span>` : ''}
                </div>
              </div>
              <div class="backup-actions">
                <button class="btn btn-primary btn-sm" onclick="window.downloadBackup(${b.id})" title="Скачать">⬇️</button>
                <button class="btn btn-success btn-sm" onclick="window.restoreBackup(${b.id})" title="Восстановить">↩️</button>
                <button class="btn btn-danger btn-sm" onclick="window.deleteBackup(${b.id})" title="Удалить">🗑️</button>
              </div>
            </div>
          `).join('')}
        </div>
      </div>
    `;
  } catch (error) {
    console.error(error);
    content.innerHTML = '<div class="empty-state">Ошибка загрузки backup</div>';
    showToast('Не удалось загрузить backup', 'error');
  }
}

export function closeBackupModal() {
  const modal = document.getElementById('backup-modal');
  if (modal) {
    modal.classList.remove('open');
  }
}

export async function createBackup() {
  const description = document.getElementById('backup-description')?.value.trim();
  
  try {
    await apiRequest('/api/backup', {
      method: 'POST',
      body: JSON.stringify({ description })
    });
    
    showToast('Backup создан', 'success');
    document.getElementById('backup-description').value = '';
    openBackupModal();
  } catch (error) {
    console.error(error);
    showToast(error.message || 'Ошибка создания backup', 'error');
  }
}

export async function downloadBackup(backupId) {
  try {
    const token = localStorage.getItem('token');
    const response = await fetch(`/api/backup/${backupId}`, {
      headers: {
        'Authorization': token ? `Bearer ${token}` : ''
      }
    });
    
    if (!response.ok) {
      throw new Error('Ошибка скачивания');
    }
    
    const blob = await response.blob();
    const url = window.URL.createObjectURL(blob);
    const a = document.createElement('a');
    a.href = url;
    a.download = response.headers.get('Content-Disposition')?.split('filename=')[1]?.replace(/"/g, '') || `backup_${backupId}.db`;
    document.body.appendChild(a);
    a.click();
    document.body.removeChild(a);
    window.URL.revokeObjectURL(url);
    
    showToast('Backup скачан', 'success');
  } catch (error) {
    console.error(error);
    showToast('Ошибка скачивания backup', 'error');
  }
}

export async function restoreBackup(backupId) {
  if (!confirm('⚠️ ВНИМАНИЕ! Восстановление backup заменит текущую базу данных.\n\nВы уверены?')) {
    return;
  }
  
  try {
    await apiRequest(`/api/backup/${backupId}/restore`, {
      method: 'POST'
    });
    
    showToast('Backup восстановлен! Перезагрузите страницу.', 'success');
    setTimeout(() => {
      window.location.reload();
    }, 2000);
  } catch (error) {
    console.error(error);
    showToast(error.message || 'Ошибка восстановления backup', 'error');
  }
}

export async function deleteBackup(backupId) {
  if (!confirm('Удалить этот backup?')) {
    return;
  }
  
  try {
    await apiRequest(`/api/backup/${backupId}`, {
      method: 'DELETE'
    });
    
    showToast('Backup удалён', 'success');
    openBackupModal();
  } catch (error) {
    console.error(error);
    showToast('Ошибка удаления backup', 'error');
  }
}

export async function toggleAutoBackup() {
  const enabled = document.getElementById('auto-backup')?.checked;
  
  try {
    await apiRequest('/api/backup/auto', {
      method: 'POST',
      body: JSON.stringify({ enabled })
    });
    
    showToast(enabled ? 'Автоматический backup включён' : 'Автоматический backup выключен', 'success');
  } catch (error) {
    console.error(error);
    showToast('Ошибка настройки auto backup', 'error');
  }
}

function escapeHtml(text) {
  const div = document.createElement('div');
  div.textContent = text;
  return div.innerHTML;
}

function formatFileSize(bytes) {
  if (bytes === 0) return '0 B';
  const k = 1024;
  const sizes = ['B', 'KB', 'MB', 'GB'];
  const i = Math.floor(Math.log(bytes) / Math.log(k));
  return Math.round(bytes / Math.pow(k, i) * 100) / 100 + ' ' + sizes[i];
}
