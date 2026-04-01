// frontend/js/modules/calendar.js
// === Calendar Functions ===

import { apiRequest } from './api.js';
import { showToast } from './toast.js';
import { escapeHtml } from './utils.js';

let currentCalendarYear = new Date().getFullYear();
let currentCalendarMonth = new Date().getMonth() + 1;

export function initCalendar() {
  renderCalendar(currentCalendarYear, currentCalendarMonth);
}

export async function renderCalendar(year, month) {
  currentCalendarYear = year;
  currentCalendarMonth = month;
  
  const container = document.getElementById('calendar-grid');
  if (!container) return;
  
  try {
    const boardId = window.currentBoardId;
    const cards = boardId 
      ? await apiRequest(`/api/boards/${boardId}/calendar?year=${year}&month=${month}`)
      : [];
    
    const firstDay = new Date(year, month - 1, 1);
    const lastDay = new Date(year, month, 0);
    const startDay = firstDay.getDay() || 7; // Пн=1, Вс=7
    const totalDays = lastDay.getDate();
    
    let html = '';
    
    // Дни недели
    html += '<div class="calendar-weekday">Пн</div>';
    html += '<div class="calendar-weekday">Вт</div>';
    html += '<div class="calendar-weekday">Ср</div>';
    html += '<div class="calendar-weekday">Чт</div>';
    html += '<div class="calendar-weekday">Пт</div>';
    html += '<div class="calendar-weekday">Сб</div>';
    html += '<div class="calendar-weekday">Вс</div>';
    
    // Пустые ячейки до первого дня
    for (let i = 1; i < startDay; i++) {
      html += '<div class="calendar-day empty"></div>';
    }
    
    // Дни месяца
    for (let day = 1; day <= totalDays; day++) {
      const date = new Date(year, month - 1, day);
      const dateStr = date.toISOString().split('T')[0];
      const dayCards = cards.filter(c => {
        if (!c.due_date) return false;
        const cardDate = new Date(c.due_date * 1000).toISOString().split('T')[0];
        return cardDate === dateStr;
      });
      
      const isToday = new Date().toDateString() === date.toDateString();
      
      html += `
        <div class="calendar-day ${isToday ? 'today' : ''}" onclick="window.selectCalendarDay(${year}, ${month}, ${day})">
          <div class="calendar-day-number">${day}</div>
          <div class="calendar-day-cards">
            ${dayCards.slice(0, 3).map(c => `
              <div class="calendar-card ${c.done ? 'done' : ''}" style="background:${c.label_color || '#0079bf'}">
                ${escapeHtml(c.title)}
              </div>
            `).join('')}
            ${dayCards.length > 3 ? `<div class="calendar-more">+${dayCards.length - 3} ещё</div>` : ''}
          </div>
        </div>
      `;
    }
    
    container.innerHTML = html;
    
    // Обновляем заголовок
    const monthNames = ['Январь', 'Февраль', 'Март', 'Апрель', 'Май', 'Июнь', 
                       'Июль', 'Август', 'Сентябрь', 'Октябрь', 'Ноябрь', 'Декабрь'];
    const header = document.getElementById('calendar-month-year');
    if (header) {
      header.textContent = `${monthNames[month - 1]} ${year}`;
    }
  } catch (error) {
    console.error(error);
    showToast('Ошибка загрузки календаря', 'error');
  }
}

export function previousMonth() {
  let month = currentCalendarMonth - 1;
  let year = currentCalendarYear;
  
  if (month < 1) {
    month = 12;
    year--;
  }
  
  renderCalendar(year, month);
}

export function nextMonth() {
  let month = currentCalendarMonth + 1;
  let year = currentCalendarYear;
  
  if (month > 12) {
    month = 1;
    year++;
  }
  
  renderCalendar(year, month);
}

export function goToToday() {
  const today = new Date();
  renderCalendar(today.getFullYear(), today.getMonth() + 1);
}

export async function selectCalendarDay(year, month, day) {
  const date = new Date(year, month - 1, day);
  const boardId = window.currentBoardId;
  
  if (!boardId) {
    showToast('Выберите доску', 'error');
    return;
  }
  
  try {
    const cards = await apiRequest(`/api/boards/${boardId}/calendar/${year}/${month}/${day}`);
    
    if (cards.length === 0) {
      showToast(`На ${day}.${month}.${year} нет карточек`, 'info');
      return;
    }
    
    const cardList = cards.map(c => `• ${escapeHtml(c.title)}`).join('\n');
    alert(`Карточки на ${day}.${month}.${year}:\n\n${cardList}`);
  } catch (error) {
    console.error(error);
    showToast('Ошибка загрузки карточек', 'error');
  }
}

export async function openCalendarModal() {
  const modal = document.getElementById('calendar-modal');
  if (modal) {
    modal.classList.add('open');
    initCalendar();
  }
}

export function closeCalendarModal() {
  const modal = document.getElementById('calendar-modal');
  if (modal) {
    modal.classList.remove('open');
  }
}
