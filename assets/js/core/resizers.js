export function makeColResizer(handle, { onMove, onEnd }) {
    let active = false;

    handle.addEventListener('mousedown', (e) => {
        active = true;
        e.preventDefault();
        document.body.style.cursor = 'col-resize';
        document.body.style.userSelect = 'none';
    });

    window.addEventListener('mousemove', (e) => {
        if (!active) return;
        onMove(e.clientX);
    });

    window.addEventListener('mouseup', () => {
        if (!active) return;
        active = false;
        document.body.style.cursor = '';
        document.body.style.userSelect = '';
        onEnd?.();
    });
}