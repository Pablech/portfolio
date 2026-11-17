import { getHtmlChat } from "./../partials/chat.js";
import { escapeHtml, time } from "./utils.js";

const MIN_PANEL_H = 48;
const TERM_MIN_H = 160;
const MAX_LINES = 200; 

const stamp = () => `<span style="opacity:.50">${time()}</span>`;

export function chatLine(author, text) {
    return `${stamp()} <span style="opacity:.6">[${escapeHtml(author)}]</span> ${escapeHtml(text)}`;
}

export function chatServer(text) {
    return `${stamp()} <span style="opacity:.6">[Server]</span> ${escapeHtml(text)}`;
}

function makeRowResizer(handle, getPanel, getHost) {
    handle.addEventListener('mousedown', (e) => {
        e.preventDefault();
        const startY = e.clientY;
        const startH = getPanel().getBoundingClientRect().height;
        const maxH = getHost().getBoundingClientRect().height - TERM_MIN_H;

        const move = (ev) => {
            const h = Math.min(
                Math.max(startH - (ev.clientY - startY), MIN_PANEL_H),
                Math.max(maxH, MIN_PANEL_H),
            );
            getPanel().style.height = `${h}px`;
        };
        const up = () => {
            document.removeEventListener('mousemove', move);
            document.removeEventListener('mouseup', up);
        };
        document.addEventListener('mousemove', move);
        document.addEventListener('mouseup', up);
    });
}

export function mountChat(shell) {
    const host = shell.host;
    if (host.querySelector('.chat-panel')) return;
    host.insertAdjacentHTML('beforeend', getHtmlChat());
    makeRowResizer(
        host.querySelector('.chat-resizer'),
        () => host.querySelector('.chat-panel'),
        () => host,
    );
}

export function openChat(shell, title) {
    shell.host.classList.add('chat-open');
    if (title) {
        const h = shell.host.querySelector('.chat-header');
        if (h) h.textContent = title;
    }
}

export function closeChat(shell) {
    shell.host.classList.remove('chat-open');
    shell.host.querySelector('.chat-strip')?.replaceChildren();
}

export function printChat(shell, html) {
    const host = shell.host;
    if (!host.classList.contains('chat-open')) return;
    const strip = host.querySelector('.chat-strip');
    if (!strip) return;

    const line = document.createElement('div');
    line.innerHTML = html;
    strip.appendChild(line);

    while (strip.children.length > MAX_LINES) strip.firstChild.remove();
    strip.scrollTop = strip.scrollHeight;
}
