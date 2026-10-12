export function initSplitScreen() {
    const resizer = document.getElementById('resizer');
    const terminal = document.getElementById('terminal');
    const sidePanel = document.getElementById('side-panel');
    
    if (!resizer || !terminal || !sidePanel) {
        console.error('[Modal] Elementos do layout não encontrados.');
        return;
    }

    let isResizing = false;

    resizer.addEventListener('mousedown', (e) => {
        isResizing = true;
        document.body.style.cursor = 'col-resize';
    });

    document.addEventListener('mousemove', (e) => {
        if (!isResizing) return;
        const containerWidth = window.innerWidth;
        const leftWidth = e.clientX;
        const rightWidth = containerWidth - leftWidth;

        if (leftWidth > 200 && rightWidth > 200) {
            terminal.style.flex = 'none';
            terminal.style.width = `${leftWidth}px`;
            sidePanel.style.flex = 'none';
            sidePanel.style.width = `${rightWidth}px`;
        }
    });

    document.addEventListener('mouseup', () => {
        if (isResizing) {
            isResizing = false;
            document.body.style.cursor = 'default';
        }
    });
}

export function openSidePanel(title, htmlContent, backCallback = null) {
    const sidePanel = document.getElementById('side-panel');
    const resizer = document.getElementById('resizer');
    const terminal = document.getElementById('terminal');
    const backBtn = document.getElementById('side-back-btn');

    sidePanel.style.display = 'flex';
    resizer.style.display = 'block';

    terminal.style.width = '';
    terminal.style.flex = '1';
    sidePanel.style.width = '';
    sidePanel.style.flex = '1';

    document.getElementById('side-title').textContent = title;
    document.getElementById('side-content').innerHTML = htmlContent;

    if (backCallback) {
        backBtn.style.display = 'inline-block';
        backBtn.onclick = backCallback;
    } else {
        backBtn.style.display = 'none';
        backBtn.onclick = null;
    }
}

export function closeSidePanel() {
    const sidePanel = document.getElementById('side-panel');
    const resizer = document.getElementById('resizer');
    const terminal = document.getElementById('terminal');

    sidePanel.style.display = 'none';
    resizer.style.display = 'none';

    terminal.style.width = '';
    terminal.style.flex = '1';

    const cmdInput = document.getElementById('cmd-input');
    if (cmdInput) cmdInput.focus();
}

export function showStatusPopup(title, message, isError = false) {
    document.getElementById('modal-status').style.display = 'block';
    const titleEl = document.getElementById('status-title');
    const bodyEl = document.getElementById('status-body');

    titleEl.style.color = isError ? '#ff5555' : 'var(--tty-color)';
    titleEl.textContent = `[ ${title} ]`;
    bodyEl.innerHTML = message;
}

