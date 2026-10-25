import { routeCommand } from "../commands/kernel.js";
import { initMouse } from "./mouse.js";
import { handleSidePanelClick } from "../commands/repoViews.js";
import { closeStatusPopup } from "../ui/statusPopup.js";
import { escapeHtml } from "./utils.js";

function renderTypedLine(shell) {
    const { input, typed } = shell;
    const value = input.value;
    const start = input.selectionStart ?? value.length;
    const end = input.selectionEnd ?? value.length;

    if (start !== end) {
        const min = Math.min(start, end);
        const max = Math.max(start, end);

        const before = escapeHtml(value.slice(0, min));
        const selected = escapeHtml(value.slice(min, max));
        const after = escapeHtml(value.slice(max));

        typed.innerHTML =
            before +
            `<span class="cursor-selected">${selected}</span>` +
            after;
    } else {
        const before = escapeHtml(value.slice(0, start));
        const at = escapeHtml(value.slice(start, start + 1));
        const after = escapeHtml(value.slice(start + 1));

        typed.innerHTML =
            before +
            `<span class="cursor-block">${at || '&nbsp;'}</span>` +
            after;
    }
}

export function attachShellEvents(shell, { setActive }) {
    const render = () => renderTypedLine(shell);

    shell.input.addEventListener('input', render);
    shell.input.addEventListener('keyup', render);
    shell.input.addEventListener('click', render);
    shell.input.addEventListener('select', render);

    shell.root.addEventListener('click', () => {
        if (shell._skipNextFocus) { shell._skipNextFocus = false; return; }
        shell.focus();
        setActive(shell);
    });

    shell.input.addEventListener('keydown', async (e) => {
        if (window.getSelection().toString()) window.getSelection().removeAllRanges();

        if (e.ctrlKey && e.key.toLowerCase() === 'l') {
            e.preventDefault();
            shell.clear();
            return;
        }
        if (e.key !== 'Enter') return;

        const value = shell.input.value.trim();
        shell.input.value = '';
        renderTypedLine(shell);
        await routeCommand(value, shell);
    });

    initMouse(shell);
}

export function initGlobalEvents(getActiveShell) {
    document.addEventListener('click', (e) => {
        if (e.target && e.target.id === 'modal-close-btn') closeStatusPopup();
    });

    document.addEventListener('keydown', (e) => {
        const popup = document.getElementById('modal-status');
        if (popup && popup.style.display === 'block' && e.key === 'Enter') {
            e.preventDefault();
            closeStatusPopup();
        }
    });

    document.addEventListener('click', (e) => {
        const el = e.target.closest('[data-action]');
        if (!el) return;
        handleSidePanelClick(e, getActiveShell());
    });
}

export function initTerminalHotkeys({ getShells, setActive, closeTerminal }) {
    document.addEventListener('keydown', (e) => {
        if (!e.altKey || e.ctrlKey || e.metaKey || e.shiftKey) return;

        const shells = getShells();

        if (e.key === 'ArrowUp' || e.key === 'ArrowDown') {
            e.preventDefault();
            if (shells.length < 2) return;

            const current = shells.findIndex(sh => sh.root.classList.contains('focused'));
            const dir = e.key === 'ArrowDown' ? 1 : -1;
            const next = (current + dir + shells.length) % shells.length;
            setActive(shells[next]);
            shells[next].focus();

        } else if (e.key === 'PageUp' || e.key === 'PageDown') {
            e.preventDefault();
            const target = e.key === 'PageUp' ? shells[0] : shells[shells.length - 1];
            if (target) { setActive(target); target.focus(); }

        } else if (e.key.toLowerCase() === 'w') {
            e.preventDefault();
            const focused = shells.find(sh => sh.root.classList.contains('focused'));
            if (focused) closeTerminal(focused);
        }
    });
}