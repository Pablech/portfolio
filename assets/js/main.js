import { attachShellEvents, initGlobalEvents, initTerminalHotkeys } from "./core/events.js";
import { getHtmlTerminal } from "./partials/terminal.js";
import { DEFAULT_PROMPT, DEFAULT_OS_HEADER } from "./ui/outputs.js";
import { makeColResizer } from "./core/resizers.js";
import { setOnNew, setOnExit } from "./commands/kernel.js";
import { restoreTheme } from "./commands/theme.js";
import { Shell } from "./core/shell.js";

const desktop = document.getElementById('desktop');
const shells = [];
let ttyCounter = 0;
let activeShell = null;

export function getActiveShell() { return activeShell ?? shells[0]; }
function setActive(shell) {
    activeShell = shell;
    document.querySelectorAll('.terminal.focused')
    .forEach(t => t.classList.remove('focused'));
    shell?.root.classList.add('focused');
}

restoreTheme();
initGlobalEvents(getActiveShell);

initTerminalHotkeys({
    getShells: () => shells,
    setActive,
    closeTerminal
});

export function closeTerminal(shell) {
    if (shells.length <= 1) {
        shell.print("<span style='color:red;'>[!] Não podes fechar o último terminal.</span>");
        return;
    }

    const { host } = shell;

    const leftSep = host.previousElementSibling;
    const sep = leftSep?.classList.contains('terminal-separator')
        ? leftSep
        : host.nextElementSibling;

    host.remove();
    sep?.remove();

    shells.splice(shells.indexOf(shell), 1);

    const lastHost = desktop.lastElementChild;
    if (lastHost) {
        lastHost.style.flex = '1';
        lastHost.style.width = '';
    }

    if (activeShell === shell) setActive(shells[shells.length - 1] ?? null);
    activeShell?.focus();
}

export function spawnTerminal() {
    ttyCounter++;

    const isFirst = shells.length === 0;

    const wrapper = document.createElement('div');
    wrapper.style.display = 'flex';
    wrapper.style.flex = '1';
    wrapper.style.minWidth = '0';
    wrapper.innerHTML = getHtmlTerminal(DEFAULT_PROMPT);

    const host = document.createElement('div');
    host.className = 'terminal-host';
    host.appendChild(wrapper);

    if (isFirst) {
        desktop.appendChild(host);
    } else {
        const sep = document.createElement('div');
        sep.className = 'terminal-separator';
        desktop.appendChild(sep);
        desktop.appendChild(host);

        makeColResizer(sep, {
            onMove: (x) => {
                const left = sep.previousElementSibling;
                if (!left || !left.classList.contains('terminal-host')) return;
                const w = x - left.getBoundingClientRect().left;
                if (w <= 150) return;
                left.style.flex = 'none';
                left.style.width = `${w}px`;
            },
        });
    }

    const shell = new Shell(wrapper.querySelector('.terminal'), DEFAULT_PROMPT);
    shell.ttyId = ttyCounter;
    shell.host = host;
    attachShellEvents(shell, { setActive });
    shells.push(shell);

    shell.print(`${DEFAULT_OS_HEADER} (tty${ttyCounter})`);
    shell.print("Escreve 'help' para veres os comandos disponíveis.");
    shell.focus();
    setActive(shell);
    return shell;
}

setOnNew(spawnTerminal);
setOnExit(closeTerminal);
spawnTerminal();
