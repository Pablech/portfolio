export function getHtmlTerminal(prompt) {
    return `
        <div class="terminal">
            <div class="output">
                <div class="cmd-output"></div>
            </div>
            <div class="input-line">
                <span class="prompt prompt-label">${prompt}</span>
                <span class="typed-text"></span>
                <input type="text" class="cmd-input" autocomplete="off" spellcheck="false"
                       style="position:absolute; opacity:0; width:1px;">
            </div>
        </div>`;
}