export function getHtmlSidePanel(title) {
    return `
        <div id="side-panel" style="display: none;">
            <div class="side-header header-drag" id="header-side">
                <div class="side-tabs">
                    <span id="side-back-btn" style="display:none;" class="btn-tty">[VOLTAR]</span>
                    <span id="side-title">${title}</span>
                </div>
                <div class="win-controls">
                    <span class="win-btn close-btn" id="side-close-btn">[X]</span>
                </div>
            </div>
            <div class="side-body" id="side-content"></div>
        </div>`;
}