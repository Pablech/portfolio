export function getHtmlModal() {
    return `
    <div id="modal-status" class="modal">
        <div class="modal-box">
            <div id="status-title" class="modal-title"></div>
            <div id="status-body"></div>
            <div class="modal-footer">
                <span class="btn-tty" id="modal-close-btn">[ FECHAR ]</span>
            </div>
        </div>
    </div>`;
}