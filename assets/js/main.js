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
        shell.print("<span style='color:red;'>Não podes fechar este terminal.</span>");
        return;
    }

    const host = shell.root.closest('.terminal-host');
    const nextHost = host.nextElementSibling;
    if (nextHost) {
        nextHost.style.flex = '1';
        nextHost.style.width = '';
    }
    host?.remove();

    const indx = shells.indexOf(shell);
    if (indx !== -1) shells.splice(indx, 1);

    if (activeShell === shell) {
        setActive(shells[shells.length - 1] ?? null);
    }
    activeShell?.focus();
}

export function spawnTerminal() {
    ttyCounter++;

    const host = document.createElement('div');
    host.className = 'terminal-host';

    if (shells.length > 0) {
        const termResizer = document.createElement('div');
        termResizer.className = 'terminal-resizer';
        makeColResizer(termResizer, {
            onMove: (x) => {
                const prev = host.previousElementSibling;
                if (!prev) return;
                const w = x - prev.getBoundingClientRect().left;
                if (w <= 150) return;
                prev.style.flex = 'none';
                prev.style.width = `${w}px`;
            },
        });
        host.appendChild(termResizer);
    }

    const wrapper = document.createElement('div');
    wrapper.style.display = 'flex';
    wrapper.style.flex = '1';
    wrapper.style.minWidth = '0';
    wrapper.innerHTML = getHtmlTerminal(DEFAULT_PROMPT);
    host.appendChild(wrapper);

    desktop.appendChild(host);

    const shell = new Shell(wrapper.querySelector('.terminal'), DEFAULT_PROMPT);
    shell.ttyId = ttyCounter;
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
