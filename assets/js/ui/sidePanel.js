import { getHtmlSidePanel } from "../partials/sidePanel.js";
import { makeColResizer } from "../core/resizers.js";

const MIN_LEFT  = 250;
const MIN_PANEL = 150;


function ensureSidePanel() {
    let panel = document.getElementById('side-panel');
    if (panel) return panel;

    const host = document.createElement('div');
    host.innerHTML = getHtmlSidePanel('');
    panel = host.firstElementChild;

    const resizer = document.getElementById('resizer');
    resizer.insertAdjacentElement('afterend', panel);

    panel.querySelector('#side-close-btn')
        .addEventListener('click', closeSidePanel);

    const desktop = document.getElementById('desktop');

    makeColResizer(resizer, {
        onMove: (x) => {
            const panelW = window.innerWidth - x - resizer.offsetWidth;
            if (x < MIN_LEFT || panelW < MIN_PANEL) return;
            desktop.style.flex = 'none';
            desktop.style.width = `${x}px`;
            panel.style.flex = 'none';
            panel.style.width = `${panelW}px`;
        },
        onEnd: () => localStorage.setItem('sidepanel_width', panel.offsetWidth),
    });

    const saved = parseInt(localStorage.getItem('sidepanel_width'), 10);
    if (Number.isFinite(saved) && saved >= MIN_PANEL) {
        const app = document.querySelector('.app-container');
        const cs = getComputedStyle(app);
        const avail = app.clientWidth
            - parseFloat(cs.paddingLeft) - parseFloat(cs.paddingRight)
            - parseFloat(cs.gap) * 2 - resizer.offsetWidth;

        const panelW = Math.min(saved, avail - MIN_LEFT);
        desktop.style.flex = 'none';
        desktop.style.width = `${avail - panelW}px`;
        panel.style.flex = 'none';
        panel.style.width = `${panelW}px`;
    }
    return panel;
}

export function openSidePanel(title, htmlContent, backCallback) {
    const panel = ensureSidePanel();

    document.getElementById('side-title').textContent = title;
    document.getElementById('side-content').innerHTML = htmlContent;

    const backBtn = document.getElementById('side-back-btn');
    if (backCallback) {
        backBtn.style.display = 'inline-flex';
        backBtn.onclick = backCallback;
    } else {
        backBtn.style.display = 'none';
    }

    panel.style.display = 'flex';
    const resizer = document.getElementById('resizer');
    if (resizer) resizer.style.display = 'block';
}

export function closeSidePanel() {
    const panel = document.getElementById('side-panel');
    const resizer = document.getElementById('resizer');
    const desktop = document.getElementById('desktop');

    if (panel)   panel.style.display = 'none';
    if (resizer) resizer.style.display = 'none';

    if (desktop) {
        desktop.style.width = '';
        desktop.style.flex = '1';
    }

    const activeInput = document.querySelector('.terminal:last-child .cmd-input')
        || document.querySelector('.cmd-input');
    activeInput?.focus();
}