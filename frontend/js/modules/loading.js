// frontend/js/modules/loading.js
// === Loading State ===

export function showLoading() {
  const indicator = document.getElementById('loading-indicator');
  if (indicator) {
    indicator.style.display = 'block';
  }
}

export function hideLoading() {
  const indicator = document.getElementById('loading-indicator');
  if (indicator) {
    indicator.style.display = 'none';
  }
}
