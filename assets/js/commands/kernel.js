import { fetchAbout, fetchGitHub, fetchHelp, fetchKeys } from "../api/api.js";
import { startWizard, handleWizardInput } from "./email.js";
import { startGitHubPagination, handlePaginationInput } from "./github.js";
import { themeCommand } from "./theme.js";
// import { GAME_PROMPT, DEFAULT_GAME_HEADER, DEFAULT_GAME_MSG, KIND_GAME } from "../ui/outputs.js";
import { fetchAndPrint } from "../ui/fetchAndPrint.js";
import { cmdError } from "../ui/outputs.js";
import { suggestCmd } from "../core/utils.js";

const registry = {};
let onNew = null;
let onExit = null;

export function setOnNew(fn) { onNew = fn; }
export function setOnExit(fn) { onExit = fn; }

registry['clear'] = (_value, shell) => shell.clear();
registry['email'] = (_value, shell) => startWizard(shell);
// registry['games'] = () => onNew?.({
//     prompt: GAME_PROMPT,
//     header: DEFAULT_GAME_HEADER,
//     welcome: DEFAULT_GAME_MSG,
//     kind: KIND_GAME,
// });
registry['theme'] = themeCommand;
registry['exit'] = (_value, shell) => onExit?.(shell);
registry['help']  = (_value, shell) => fetchAndPrint(shell, fetchHelp);
registry['about'] = (_value, shell) => fetchAndPrint(shell, fetchAbout);
registry['keys']  = (_value, shell) => fetchAndPrint(shell, fetchKeys);

registry['resume'] = (_value, shell) => {
    shell.print("A abrir o currículo... [PDF]");
    const a = document.createElement('a');
    a.href = '/api/resume';
    a.target = '_blank';
    a.rel = 'noopener';
    a.click();
};

registry['github'] = async (_value, shell) => {
    const s = shell.state;
    shell.print("A ligar à API do GitHub... [AGUARDE]");
    shell.input.disabled = true;
    shell.setPrompt("A CARREGAR...");

    const { ok, status, data } = await fetchGitHub();
    if (!ok) {
        shell.print(`<span style='color:red;'>[Erro] Servidor respondeu ${status}.</span>`);
    } else if (data?.output) {
        s.backendRepos = JSON.parse(data.output);
        startGitHubPagination(s.backendRepos, shell);
    }

    shell.input.disabled = false;
    shell.setPrompt(shell.prompt);
    shell.focus();
};


export async function routeCommand(value, shell) {
    const allCommands = Object.keys(registry);
    const s = shell.state;

    if (s.isPaginating)   { handlePaginationInput(value, shell); return; }
    if (s.wizardStep !== null) { handleWizardInput(value, shell); return; }

    
    shell.print(`<span class="prompt">${shell.prompt}</span>${value}`);
    if (!value) return;
    
    const name = value.split(/\s+/)[0];
    const handler = registry[name];
    if (handler) return handler(value, shell);

    let candidates = suggestCmd(name, allCommands);
    if (candidates?.length) {
        const suggestions = candidates.map(c => `\`${c.cmd}\``).join(', ');
        shell.print(cmdError(value, suggestions));
    } else {
        shell.print(cmdError(value));
    }
}