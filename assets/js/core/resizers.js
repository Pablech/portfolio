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

// function attachChatResizer(host) {
//     const rz = host.querySelector('.chat-resizer');
//     const panel = host.querySelector('.chat-panel');

//     rz.addEventListener('mousedown', (e) => {
//         e.preventDefault();
//         const startY = e.clientY;
//         const startH = panel.getBoundingClientRect().height;
//         const maxH = host.getBoundingClientRect().height - 160;

//         const move = (ev) => {
//             const h = Math.min(Math.max(startH - (ev.clientY - startY), 48), maxH);
//             panel.style.height = h + 'px';
//             panel.style.flexShrink = 0;
//         };
//         const up = () => {
//             document.removeEventListener('mousemove', move);
//             document.removeEventListener('mouseup', up);
//         };
//         document.addEventListener('mousemove', move);
//         document.addEventListener('mouseup', up);
//     });
// }