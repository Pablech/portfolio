export function makeDraggable(modalId, headerId) {
    const modal = document.getElementById(modalId);
    const header = document.getElementById(headerId);
    let pos1 = 0, pos2 = 0, pos3 = 0, pos4 = 0;

    header.onmousedown = dragMouseDown;

    function dragMouseDown(e) {
        if (e.target.classList.contains('win-btn') || e.target.classList.contains('close-btn')) return;
        e.preventDefault();
        pos3 = e.clientX;
        pos4 = e.clientY;
        document.onmouseup = closeDragElement;
        document.onmousemove = elementDrag;
    }

    function elementDrag(e) {
        e.preventDefault();
        pos1 = pos3 - e.clientX;
        pos2 = pos4 - e.clientY;
        pos3 = e.clientX;
        pos4 = e.clientY;
        
        let newTop = modal.offsetTop - pos2;
        if (newTop < 0) newTop = 0;

        modal.style.top = newTop + "px";
        modal.style.left = (modal.offsetLeft - pos1) + "px";
    }

    function closeDragElement() {
        document.onmouseup = null;
        document.onmousemove = null;
    }
}

export function toggleMaximize(modalId) {
    document.getElementById(modalId).classList.toggle('modal-maximized');
}

export function toggleMinimize(modalId, bodyId) {
    const body = document.getElementById(bodyId);
    const modal = document.getElementById(modalId);
    
    if (body.style.display === 'none') {
        body.style.display = '';
        modal.style.height = modalId === 'modal-vscode' ? '500px' : 'auto'; 
    } else {
        body.style.display = 'none';
        modal.style.height = 'auto';
    }
}

export function showStatusPopup(title, message, isError = false) {
    document.getElementById('overlay').style.display = 'block';
    const modal = document.getElementById('modal-status');
    modal.style.display = 'block';

    const titleEl = document.getElementById('status-title');
    const bodyEl = document.getElementById('status-body');

    titleEl.style.color = isError ? '#f7768e' : '#9ece6a';
    titleEl.textContent = `[ ${title} ]`;
    bodyEl.innerHTML = message;
}

export function closeAllModals(cmdInput) {
    document.getElementById('overlay').style.display = 'none';
    document.getElementById('modal-detalhes').style.display = 'none';
    document.getElementById('modal-vscode').style.display = 'none';
    if (cmdInput) cmdInput.focus();
}