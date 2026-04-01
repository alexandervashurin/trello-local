// frontend/js/modules/comments.js
// === Comments Management ===

import { apiRequest } from './api.js';
import { showToast } from './toast.js';
import { escapeHtml, formatDateTime, getInitials } from './utils.js';

export async function openCommentsModal(cardId) {
  const modal = document.getElementById('comments-modal');
  const content = document.getElementById('comments-list');
  
  if (!modal || !content) return;
  
  modal.classList.add('open');
  content.innerHTML = '<div class="loading">Загрузка комментариев...</div>';
  
  try {
    const comments = await apiRequest(`/api/cards/${cardId}/comments`);
    
    content.innerHTML = `
      <div class="comments-list">
        ${comments.length === 0 ? '<p class="empty">Нет комментариев</p>' : ''}
        ${comments.map(c => `
          <div class="comment-item" data-comment-id="${c.id}">
            <div class="comment-header">
              <span class="avatar" style="background:${c.avatar_color || '#0079bf'}">${getInitials(c.username)}</span>
              <div>
                <strong>${escapeHtml(c.username)}</strong>
                <small>${formatDateTime(c.created_at)}</small>
              </div>
              ${window.getUser()?.id === c.user_id ? `
                <div class="comment-actions">
                  <button class="btn btn-sm" onclick="window.editComment(${c.id})">✏️</button>
                  <button class="btn btn-sm btn-danger" onclick="window.deleteComment(${c.id})">🗑️</button>
                </div>
              ` : ''}
            </div>
            <div class="comment-content">${escapeHtml(c.content)}</div>
          </div>
        `).join('')}
      </div>
      
      <div class="comment-add">
        <textarea id="new-comment" placeholder="Напишите комментарий..." rows="3" style="width:100%;padding:8px;border:1px solid #dfe1e6;border-radius:4px;"></textarea>
        <button class="btn btn-primary" onclick="window.addComment(${cardId})" style="margin-top:8px;">Отправить</button>
      </div>
    `;
  } catch (error) {
    console.error(error);
    content.innerHTML = '<div class="empty-state">Ошибка загрузки комментариев</div>';
    showToast('Не удалось загрузить комментарии', 'error');
  }
}

export function closeCommentsModal() {
  const modal = document.getElementById('comments-modal');
  if (modal) {
    modal.classList.remove('open');
  }
}

export async function addComment(cardId) {
  const content = document.getElementById('new-comment')?.value.trim();
  
  if (!content) {
    showToast('Введите текст комментария', 'error');
    return;
  }
  
  try {
    await apiRequest(`/api/cards/${cardId}/comments`, {
      method: 'POST',
      body: JSON.stringify({ content })
    });
    
    showToast('Комментарий добавлен', 'success');
    openCommentsModal(cardId);
  } catch (error) {
    console.error(error);
    showToast(error.message || 'Ошибка добавления комментария', 'error');
  }
}

export async function deleteComment(commentId) {
  if (!confirm('Удалить этот комментарий?')) return;
  
  try {
    await apiRequest(`/api/comments/${commentId}`, { method: 'DELETE' });
    showToast('Комментарий удалён', 'success');
    const cardId = window.currentCardId;
    if (cardId) {
      openCommentsModal(cardId);
    }
  } catch (error) {
    console.error(error);
    showToast('Ошибка удаления комментария', 'error');
  }
}

export async function editComment(commentId) {
  const commentEl = document.querySelector(`.comment-item[data-comment-id="${commentId}"]`);
  const contentEl = commentEl?.querySelector('.comment-content');
  const currentContent = contentEl?.textContent;
  
  const newContent = prompt('Редактировать комментарий:', currentContent);
  if (!newContent || newContent === currentContent) return;
  
  try {
    await apiRequest(`/api/comments/${commentId}`, {
      method: 'PATCH',
      body: JSON.stringify({ content: newContent })
    });
    
    showToast('Комментарий обновлён', 'success');
    const cardId = window.currentCardId;
    if (cardId) {
      openCommentsModal(cardId);
    }
  } catch (error) {
    console.error(error);
    showToast('Ошибка редактирования комментария', 'error');
  }
}
