export function initMouse(shell) {
    const root = shell.root;
    let dragging = false;
    let rafId = null;

    const sel = () => window.getSelection().toString();

    root.addEventListener('mousedown', () => {
        dragging = true;
        stopAutoScroll();
    });

    window.addEventListener('mousemove', (e) => {
        if (!dragging || e.buttons !== 1) return;

        const rect = root.getBoundingClientRect();
        const EDGE = 28;
        let dy = 0;
        if (e.clientY < rect.top + EDGE)    dy = -14;
        if (e.clientY > rect.bottom - EDGE) dy = 14;

        autoScroll(dy);
    });

    window.addEventListener('mouseup', () => {
        if (!dragging) return;
        dragging = false;
        stopAutoScroll();

        const text = sel();
        if (text) {
            try { navigator.clipboard?.writeText(text); } catch (_) {}
            shell._skipNextFocus = true;
            return;
        }
        shell.focus();
        shell.onActivate?.(shell);
    });

    function autoScroll(dy) {
        stopAutoScroll();
        if (!dy) return;
        const step = () => { root.scrollTop += dy; rafId = requestAnimationFrame(step); };
        rafId = requestAnimationFrame(step);
    }

    function stopAutoScroll() {
        if (rafId) cancelAnimationFrame(rafId), rafId = null;
    }
}