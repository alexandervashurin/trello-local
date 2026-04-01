// frontend/js/modules/labels.js
// === Labels Management ===

import { apiRequest } from './api.js';
import { showToast } from './toast.js';
import { escapeHtml } from './utils.js';

const COLORS = [
  '#eb5a46', '#f5a623', '#61bd4f', '#0079bf', '#89609e', '#c377e0',
  '#ff78cb', '#ffd700', '#00e6cc', '#51e898', '#73b3e6', '#d93025'
];

export async function openLabelsModal(cardId) {
  const modal = document.getElementById('labels-modal');
  const content = document.getElementById('labels-list');
  
  if (!modal || !content) return;
  
  modal.classList.add('open');
  content.innerHTML = '<div class="loading">Загрузка меток...</div>';
  
  try {
    const labels = await apiRequest(`/api/cards/${cardId}/labels`);
    
    content.innerHTML = `
      <div class="labels-list">
        ${labels.length === 0 ? '<p class="empty">Нет меток</p>' : ''}
        ${labels.map(l => `
          <div class="label-item" style="border-left:4px solid ${l.color}">
            <span class="label-color" style="background:${l.color}"></span>
            <span>${escapeHtml(l.name)}</span>
            <button class="btn btn-sm btn-danger" onclick="window.deleteLabel(${cardId}, ${l.id})">Удалить</button>
          </div>
        `).join('')}
      </div>
      
      <div class="label-add">
        <h4>Добавить метку</h4>
        <input type="text" id="new-label-name" placeholder="Название метки" style="width:100%;padding:8px;margin-bottom:8px;">
        <div class="label-colors" style="display:flex;gap:4px;flex-wrap:wrap;margin-bottom:8px;">
          ${COLORS.map(c => `<span class="color-swatch" style="width:24px;height:24px;background:${c};border-radius:4px;cursor:pointer;" onclick="window.selectLabelColor('${c}')"></span>`).join('')}
        </div>
        <input type="hidden" id="selected-label-color" value="${COLORS[0]}">
        <button class="btn btn-primary" onclick="window.addLabel(${cardId})">Добавить</button>
      </div>
    `;
  } catch (error) {
    console.error(error);
    content.innerHTML = '<div class="empty-state">Ошибка загрузки меток</div>';
    showToast('Не удалось загрузить метки', 'error');
  }
}

export function closeLabelsModal() {
  const modal = document.getElementById('labels-modal');
  if (modal) {
    modal.classList.remove('open');
  }
}

export function selectLabelColor(color) {
  document.getElementById('selected-label-color').value = color;
  document.querySelectorAll('.color-swatch').forEach(swatch => {
    swatch.style.border = swatch.style.background === color ? '2px solid #172b4d' : 'none';
  });
}

export async function addLabel(cardId) {
  const name = document.getElementById('new-label-name')?.value.trim();
  const color = document.getElementById('selected-label-color')?.value;
  
  if (!name) {
    showToast('Введите название метки', 'error');
    return;
  }
  
  try {
    await apiRequest(`/api/cards/${cardId}/labels`, {
      method: 'POST',
      body: JSON.stringify({ name, color })
    });
    
    showToast('Метка добавлена', 'success');
    openLabelsModal(cardId);
  } catch (error) {
    console.error(error);
    showToast('Ошибка добавления метки', 'error');
  }
}

export async function deleteLabel(cardId, labelId) {
  try {
    await apiRequest(`/api/cards/${cardId}/labels/${labelId}`, { method: 'DELETE' });
    showToast('Метка удалена', 'success');
    openLabelsModal(cardId);
  } catch (error) {
    console.error(error);
    showToast('Ошибка удаления метки', 'error');
  }
}

export async function openLabelFilter(boardId) {
  const modal = document.getElementById('label-filter-modal');
  const content = document.getElementById('label-filter-content');
  
  if (!modal || !content) return;
  
  modal.classList.add('open');
  content.innerHTML = '<div class="loading">Загрузка меток...</div>';
  
  try {
    const labels = await apiRequest(`/api/boards/${boardId}/labels`);
    
    content.innerHTML = `
      <div class="label-filter-list">
        <label><input type="checkbox" value="all" checked> Все метки</label>
        ${labels.map(l => `
          <label style="display:flex;align-items:center;gap:8px;padding:4px 0;">
            <input type="checkbox" value="${l.id}" class="label-filter-checkbox">
            <span class="label-color" style="width:16px;height:16px;background:${l.color};border-radius:4px;"></span>
            <span>${escapeHtml(l.name)}</span>
          </label>
        `).join('')}
      </div>
      <button class="btn btn-primary" onclick="window.applyLabelFilter(${boardId})" style="margin-top:16px;">Применить</button>
    `;
  } catch (error) {
    console.error(error);
    content.innerHTML = '<div class="empty-state">Ошибка загрузки меток</div>';
    showToast('Не удалось загрузить метки', 'error');
  }
}

export function closeLabelFilter() {
  const modal = document.getElementById('label-filter-modal');
  if (modal) {
    modal.classList.remove('open');
  }
}

export async function applyLabelFilter(boardId) {
  const checkboxes = document.querySelectorAll('.label-filter-checkbox:checked');
  const selectedLabels = Array.from(checkboxes).map(cb => cb.value);
  
  showToast(`Фильтр по меткам: ${selectedLabels.length} выбрано`, 'info');
  closeLabelFilter();
  // Здесь можно добавить логику фильтрации досок
}
