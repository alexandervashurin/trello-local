// frontend/js/modules/attachments.js
// === Attachments Management ===

import { apiRequest } from './api.js';
import { showToast } from './toast.js';
import { escapeHtml, formatDateTime } from './utils.js';

export async function loadAttachments(cardId) {
  try {
    const attachments = await apiRequest(`/api/cards/${cardId}/attachments`);
    return attachments;
  } catch (error) {
    console.error(error);
    return [];
  }
}

export async function openAttachmentsModal(cardId) {
  const modal = document.getElementById('attachments-modal');
  const content = document.getElementById('attachments-list');
  
  if (!modal || !content) return;
  
  modal.classList.add('open');
  content.innerHTML = '<div class="loading">Загрузка вложений...</div>';
  
  try {
    const attachments = await loadAttachments(cardId);
    
    content.innerHTML = `
      <div class="attachments-list">
        ${attachments.length === 0 ? '<p class="empty">Нет вложений</p>' : ''}
        ${attachments.map(a => `
          <div class="attachment-item">
            <div class="attachment-icon">📎</div>
            <div class="attachment-info">
              <a href="/api/attachments/${a.id}" target="_blank">${escapeHtml(a.filename)}</a>
              <small>${formatDateTime(a.created_at)} • ${formatFileSize(a.file_size)}</small>
            </div>
            <button class="btn btn-sm btn-danger" onclick="window.deleteAttachment(${cardId}, ${a.id})">🗑️</button>
          </div>
        `).join('')}
      </div>
      
      <div class="attachment-upload">
        <h4>Загрузить вложение</h4>
        <input type="file" id="attachment-file" style="margin-bottom:8px;">
        <button class="btn btn-primary" onclick="window.uploadAttachment(${cardId})">Загрузить</button>
      </div>
    `;
  } catch (error) {
    console.error(error);
    content.innerHTML = '<div class="empty-state">Ошибка загрузки вложений</div>';
    showToast('Не удалось загрузить вложения', 'error');
  }
}

export function closeAttachmentsModal() {
  const modal = document.getElementById('attachments-modal');
  if (modal) {
    modal.classList.remove('open');
  }
}

export function closeImagePreview() {
  const modal = document.getElementById('image-preview-modal');
  if (modal) {
    modal.classList.remove('open');
  }
}

export function openImagePreview(url) {
  const modal = document.getElementById('image-preview-modal');
  const img = document.getElementById('image-preview-img');
  
  if (modal && img) {
    img.src = url;
    modal.classList.add('open');
  }
}

export async function uploadAttachment(cardId) {
  const fileInput = document.getElementById('attachment-file');
  const file = fileInput?.files[0];
  
  if (!file) {
    showToast('Выберите файл', 'error');
    return;
  }
  
  const formData = new FormData();
  formData.append('file', file);
  
  try {
    const token = localStorage.getItem('token');
    const response = await fetch(`/api/cards/${cardId}/boards/${window.currentBoardId}/attachments`, {
      method: 'POST',
      headers: {
        'Authorization': token ? `Bearer ${token}` : ''
      },
      body: formData
    });
    
    if (!response.ok) {
      const error = await response.json().catch(() => ({ error: 'Ошибка сервера' }));
      throw new Error(error.error || 'Ошибка загрузки');
    }
    
    showToast('Вложение загружено', 'success');
    openAttachmentsModal(cardId);
  } catch (error) {
    console.error(error);
    showToast(error.message || 'Ошибка загрузки вложения', 'error');
  }
}

export async function deleteAttachment(cardId, attachmentId) {
  if (!confirm('Удалить это вложение?')) return;
  
  try {
    await apiRequest(`/api/cards/${cardId}/attachments/${attachmentId}`, { method: 'DELETE' });
    showToast('Вложение удалено', 'success');
    openAttachmentsModal(cardId);
  } catch (error) {
    console.error(error);
    showToast('Ошибка удаления вложения', 'error');
  }
}

function formatFileSize(bytes) {
  if (bytes === 0) return '0 B';
  const k = 1024;
  const sizes = ['B', 'KB', 'MB', 'GB'];
  const i = Math.floor(Math.log(bytes) / Math.log(k));
  return Math.round(bytes / Math.pow(k, i) * 100) / 100 + ' ' + sizes[i];
}

export function renderAttachments(attachments) {
  if (!attachments || attachments.length === 0) return '<p class="empty">Нет вложений</p>';
  
  return attachments.map(a => `
    <div class="attachment-item">
      <div class="attachment-icon">📎</div>
      <div class="attachment-info">
        <a href="/api/attachments/${a.id}" target="_blank">${escapeHtml(a.filename)}</a>
        <small>${formatDateTime(a.created_at)} • ${formatFileSize(a.file_size)}</small>
      </div>
    </div>
  `).join('');
}
