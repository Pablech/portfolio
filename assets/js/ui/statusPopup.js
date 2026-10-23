import { getHtmlModal } from "../partials/modal.js";

function ensureModal() {
    let modal = document.getElementById('modal-status');
    if (modal) return modal;

    const host = document.createElement('div');
    host.innerHTML = getHtmlModal();
    modal = host.firstElementChild;
    document.body.appendChild(modal);
    return modal;
}

export function showStatusPopup(title, message, isError = false) {
    const popup = ensureModal();

    popup.style.display = 'block';
    document.getElementById('status-title').style.color = isError ? '#ff5555' : 'var(--tty-color)';
    document.getElementById('status-title').textContent = `[ ${title} ]`;
    document.getElementById('status-body').innerHTML = message;
}

export function closeStatusPopup() {
    const popup = document.getElementById('modal-status');
    if (popup) popup.style.display = 'none';

    const activeInput = document.querySelector('.terminal:last-child .cmd-input')
        || document.querySelector('.cmd-input');
    activeInput?.focus();
}