import { attachShellEvents, initGlobalEvents, initTerminalHotkeys } from "./core/events.js";
import { getHtmlTerminal } from "./partials/terminal.js";
import { DEFAULT_PROMPT, DEFAULT_OS_HEADER, DEFAULT_INIT_MSG, KIND_OS } from "./ui/outputs.js";
import { makeColResizer } from "./core/resizers.js";
import { setOnNew, setOnExit } from "./commands/kernel.js";
import { restoreTheme } from "./commands/theme.js";
import { Shell } from "./core/shell.js";

const desktop = document.getElementById('desktop');
const shells = [];
let activeShell = null;

function getActiveShell() { return activeShell ?? shells[0]; }
function setActive(shell) {
    activeShell = shell;
    document.querySelectorAll('.terminal.focused')
    .forEach(t => t.classList.remove('focused'));
    shell?.root.classList.add('focused');
}

function nextTtyId() {
    const used = new Set(shells.map(s => s.ttyId));
    let id = 1;
    while (used.has(id)) id++;
    return id;
}

restoreTheme();
initGlobalEvents(getActiveShell);

initTerminalHotkeys({
    getShells: () => shells,
    setActive,
    closeTerminal
});

export function closeTerminal(shell) {
    if (shell.kind === KIND_OS) {
        shell.print("<span style='color:red;'>[!] Não podes fechar este terminal.</span>");
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

export function spawnTerminal(opts = {}) {
    const {
        prompt = DEFAULT_PROMPT,
        header = DEFAULT_OS_HEADER,
        welcome = DEFAULT_INIT_MSG,
        kind = KIND_OS,
        onSpawn,
    } = opts;

    const existing = shells.find(s => s.kind === kind);
    if (existing) {
        const os = shells.find(s => s.kind === KIND_OS);
        os.print(`<span style='color:red;'>[!] Já tens um terminal "${kind}" aberto.</span>`);
        os.focus();
        setActive(existing);
        return existing;
    }

    const ttyId = nextTtyId();

    const wrapper = document.createElement('div');
    wrapper.style.display = 'flex';
    wrapper.style.flex = '1';
    wrapper.style.minWidth = '0';
    wrapper.innerHTML = getHtmlTerminal(prompt);

    const host = document.createElement('div');
    host.className = 'terminal-host';
    host.appendChild(wrapper);

    if (kind === KIND_OS) {
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

    const shell = new Shell(wrapper.querySelector('.terminal'), prompt, kind);
    shell.ttyId = ttyId;
    shell.kind = kind;
    shell.host = host;
    attachShellEvents(shell, { setActive });
    shells.push(shell);

    shell.print(`${header} (tty${ttyId})`);
    shell.print(welcome);
    shell.focus();
    setActive(shell);
    onSpawn?.(shell);
    return shell;
}

setOnNew(spawnTerminal);
setOnExit(closeTerminal);
spawnTerminal();
