// frontend/js/modules/profile.js
// === Profile Management ===

import { apiRequest } from './api.js';
import { showToast } from './toast.js';
import { escapeHtml } from './utils.js';

export async function openProfileModal() {
  const modal = document.getElementById('profile-modal');
  const content = document.getElementById('profile-content');

  modal.classList.add('open');
  content.innerHTML = '<div class="loading">Загрузка профиля...</div>';

  try {
    const user = await apiRequest('/api/profile');
    const twoFAStatus = await apiRequest('/api/2fa/status').catch(() => ({ enabled: false }));

    const twoFAHtml = twoFAStatus.enabled
      ? `
        <div class="profile-field" style="background:#e3fcef;padding:16px;border-radius:8px;border-left:4px solid #61bd4f;">
          <label>🔐 Двухфакторная аутентификация</label>
          <p style="color:#006644;font-size:14px;margin:8px 0;">✅ Двухфакторная аутентификация включена</p>
          <button class="btn btn-danger" onclick="window.disable2FA()" style="margin-top:8px;">🔓 Отключить 2FA</button>
        </div>
      `
      : `
        <div class="profile-field" style="background:#fff0b3;padding:16px;border-radius:8px;border-left:4px solid #f5a623;">
          <label>🔐 Двухфакторная аутентификация</label>
          <p style="color:#856404;font-size:14px;margin:8px 0;">⚠️ Двухфакторная аутентификация не включена</p>
          <button class="btn btn-primary" onclick="window.setup2FA()" style="margin-top:8px;">🔑 Настроить 2FA</button>
        </div>
      `;

    content.innerHTML = `
      <div class="profile-form">
        <div class="profile-avatar" style="width:80px; height:80px; border-radius:50%; background:${user.avatar_color || '#0079bf'}; display:flex; align-items:center; justify-content:center; font-size:32px; font-weight:bold; color:white; margin:0 auto 16px;">
          ${user.username.charAt(0).toUpperCase()}
        </div>

        <div class="profile-info">
          <div class="profile-field">
            <label>👤 Имя пользователя</label>
            <input type="text" id="profile-username" value="${escapeHtml(user.username)}" disabled style="background:#f4f5f7;">
          </div>

          <div class="profile-field">
            <label>📧 Email</label>
            <input type="email" id="profile-email" value="${escapeHtml(user.email || '')}" placeholder="Не указан">
          </div>

          <div class="profile-field">
            <label>🎨 Цвет аватара</label>
            <input type="color" id="profile-avatar-color" value="${user.avatar_color || '#0079bf'}" style="width:100%; height:40px; cursor:pointer;">
          </div>

          <div class="profile-field">
            <label>📝 О себе</label>
            <textarea id="profile-bio" rows="3" placeholder="Расскажите о себе...">${escapeHtml(user.bio || '')}</textarea>
          </div>

          <div class="profile-field">
            <label>📅 Зарегистрирован</label>
            <input type="text" value="${new Date(user.created_at * 1000).toLocaleDateString('ru-RU')}" disabled style="background:#f4f5f7;">
          </div>

          <div class="profile-field">
            <label>🕐 Последний вход</label>
            <input type="text" value="${user.last_login ? new Date(user.last_login * 1000).toLocaleString('ru-RU') : '—'}" disabled style="background:#f4f5f7;">
          </div>

          ${twoFAHtml}
        </div>

        <div class="profile-actions" style="display:flex; gap:10px; margin-top:20px; flex-wrap:wrap;">
          <button class="btn btn-primary" onclick="window.saveProfile()" style="flex:1;">💾 Сохранить</button>
          <button class="btn btn-secondary" onclick="window.openChangePassword()" style="flex:1;">🔑 Сменить пароль</button>
          <button class="btn btn-danger" onclick="window.openDeleteAccount()" style="flex:1;">🗑️ Удалить аккаунт</button>
        </div>
      </div>
    `;
  } catch (error) {
    console.error(error);
    content.innerHTML = '<div class="empty-state">Ошибка загрузки профиля</div>';
    showToast('Не удалось загрузить профиль', 'error');
  }
}

export function closeProfileModal() {
  const modal = document.getElementById('profile-modal');
  if (modal) {
    modal.classList.remove('open');
  }
}

export async function saveProfile() {
  const email = document.getElementById('profile-email').value.trim();
  const avatarColor = document.getElementById('profile-avatar-color').value;
  const bio = document.getElementById('profile-bio').value.trim();

  try {
    const user = await apiRequest('/api/profile', {
      method: 'PATCH',
      body: JSON.stringify({ email, avatar_color: avatarColor, bio: bio || null })
    });

    const avatarEl = document.querySelector('.profile-avatar');
    if (avatarEl) {
      avatarEl.style.backgroundColor = avatarColor;
    }

    showToast('Профиль обновлён', 'success');
  } catch (error) {
    console.error(error);
    showToast(error.message || 'Не удалось сохранить профиль', 'error');
  }
}

export async function openChangePassword() {
  const currentPassword = prompt('Введите текущий пароль:');
  if (!currentPassword) return;

  const newPassword = prompt('Введите новый пароль (минимум 8 символов, заглавные, строчные, цифры):');
  if (!newPassword) return;

  if (newPassword.length < 8) {
    showToast('Пароль должен быть не менее 8 символов', 'error');
    return;
  }

  const hasUpper = /[A-Z]/.test(newPassword);
  const hasLower = /[a-z]/.test(newPassword);
  const hasDigit = /\d/.test(newPassword);

  if (!hasUpper || !hasLower || !hasDigit) {
    showToast('Пароль должен содержать заглавные и строчные буквы, а также цифры', 'error');
    return;
  }

  try {
    await apiRequest('/api/profile/change-password', {
      method: 'POST',
      body: JSON.stringify({ current_password: currentPassword, new_password: newPassword })
    });

    showToast('Пароль успешно изменён', 'success');
    closeProfileModal();
  } catch (error) {
    console.error(error);
    showToast(error.message || 'Не удалось сменить пароль', 'error');
  }
}

export async function openDeleteAccount() {
  const password = prompt('⚠️ ВНИМАНИЕ! Это действие необратимо.\n\nВведите ваш пароль для подтверждения удаления аккаунта:');
  if (!password) return;

  if (!confirm('Вы уверены, что хотите удалить аккаунт? Все ваши доски, карточки и данные будут безвозвратно удалены.')) {
    return;
  }

  try {
    await apiRequest('/api/profile/delete', {
      method: 'POST',
      body: JSON.stringify({ password })
    });

    localStorage.removeItem('token');
    localStorage.removeItem('user');
    window.location.href = '/login.html';
  } catch (error) {
    console.error(error);
    showToast(error.message || 'Не удалось удалить аккаунт', 'error');
  }
}

// === 2FA Functions ===

export async function setup2FA() {
  try {
    const setupData = await apiRequest('/api/2fa/setup', {
      method: 'POST',
      body: JSON.stringify({ code: 'setup' })
    });

    const modal = document.getElementById('profile-modal');
    const content = document.getElementById('profile-content');

    content.innerHTML = `
      <div class="profile-form">
        <h3 style="text-align:center;margin-bottom:20px;">🔐 Настройка двухфакторной аутентификации</h3>
        
        <div style="text-align:center;margin:20px 0;">
          <img src="${setupData.qr_code}" alt="QR Code" style="max-width:256px;border:1px solid #dfe1e6;border-radius:8px;padding:8px;background:white;">
        </div>
        
        <div style="background:#f4f5f7;padding:16px;border-radius:8px;margin:16px 0;">
          <h4 style="margin:0 0 8px;">📱 Инструкция:</h4>
          <ol style="margin:0;padding-left:20px;line-height:1.8;">
            <li>Установите приложение аутентификации (Google Authenticator, Authy, Microsoft Authenticator)</li>
            <li>Отсканируйте QR-код выше</li>
            <li>Введите 6-значный код из приложения для подтверждения</li>
          </ol>
        </div>
        
        <div style="background:#fff0b3;padding:12px;border-radius:8px;margin:16px 0;">
          <p style="margin:0;color:#856404;font-size:14px;">
            <strong>⚠️ Секретный ключ:</strong> <code style="background:#fff;padding:4px 8px;border-radius:4px;font-size:12px;">${setupData.secret}</code>
          </p>
          <p style="margin:8px 0 0;color:#856404;font-size:12px;">Сохраните этот ключ в безопасном месте для восстановления доступа!</p>
        </div>
        
        <div style="margin-top:20px;">
          <label>🔢 Код из приложения:</label>
          <input type="text" id="2fa-setup-code" placeholder="000000" maxlength="6" style="text-align:center;font-size:20px;letter-spacing:4px;margin-top:8px;">
        </div>
        
        <div class="profile-actions" style="display:flex;gap:10px;margin-top:20px;">
          <button class="btn btn-primary" onclick="window.confirm2FAEnable()" style="flex:1;">✅ Подтвердить и включить 2FA</button>
          <button class="btn btn-secondary" onclick="window.openProfileModal()" style="flex:1;">✕ Отмена</button>
        </div>
      </div>
    `;

    modal.classList.add('open');
  } catch (error) {
    console.error(error);
    showToast('Ошибка настройки 2FA', 'error');
  }
}

export async function confirm2FAEnable() {
  const code = document.getElementById('2fa-setup-code').value.trim();

  if (!code || code.length !== 6) {
    showToast('Введите 6-значный код', 'error');
    return;
  }

  try {
    await apiRequest('/api/2fa/enable', {
      method: 'POST',
      body: JSON.stringify({ code, enable: true })
    });

    showToast('Двухфакторная аутентификация включена!', 'success');
    openProfileModal();
  } catch (error) {
    console.error(error);
    showToast(error.message || 'Неверный код 2FA', 'error');
  }
}

export async function disable2FA() {
  const code = prompt('Введите текущий код из приложения аутентификации для отключения 2FA:');
  
  if (!code || code.length !== 6) {
    showToast('Введите 6-значный код', 'error');
    return;
  }

  if (!confirm('Вы уверены, что хотите отключить двухфакторную аутентификацию?')) {
    return;
  }

  try {
    await apiRequest('/api/2fa/enable', {
      method: 'POST',
      body: JSON.stringify({ code, enable: false })
    });

    showToast('Двухфакторная аутентификация отключена', 'success');
    openProfileModal();
  } catch (error) {
    console.error(error);
    showToast(error.message || 'Неверный код 2FA', 'error');
  }
}
