import { fetchAbout, fetchGitHub, fetchHelp, fetchKeys } from "../api/api.js";
import { startWizard, handleWizardInput } from "./email.js";
import { startGitHubPagination, handlePaginationInput } from "./github.js";
import { themeCommand } from "./theme.js";

const registry = {};
let onNew = null;
let onExit = null;

export function setOnNew(fn) { onNew = fn; }
export function setOnExit(fn) { onExit = fn; }

async function fetchAndPrint(shell, fn) {
    shell.print("<br>Carregando... [AGUARDE]");
    shell.input.disabled = true;
    shell.setPrompt("A CARREGAR...");

    try {
        const { ok, status, data } = await fn();
        if (!ok) {
            shell.print(`<span style='color:red;'>[Erro] Servidor respondeu ${status}.</span>`);
            return;
        }
        if (data?.output) shell.print(data.output);
    } finally {
        shell.input.disabled = false;
        shell.setPrompt(shell.prompt);
        shell.focus();
    }
}

registry['clear'] = (_value, shell) => shell.clear();
registry['email'] = (_value, shell) => startWizard(shell);
registry['new']   = () => onNew?.();
registry['theme'] = themeCommand;
registry['exit'] = (_value, shell) => onExit?.(shell);
registry['help']  = (_value, shell) => fetchAndPrint(shell, fetchHelp);
registry['about'] = (_value, shell) => fetchAndPrint(shell, fetchAbout);
registry['keys']  = (_value, shell) => fetchAndPrint(shell, fetchKeys);

registry['github'] = async (_value, shell) => {
    const s = shell.state;
    shell.print("<br>A ligar à API do GitHub... [AGUARDE]");
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
    const s = shell.state;

    if (s.isPaginating)   { handlePaginationInput(value, shell); return; }
    if (s.wizardStep !== null) { handleWizardInput(value, shell); return; }

    shell.print(`<span class="prompt">${shell.prompt}</span>${value}`);
    if (!value) return;

    const name = value.split(/\s+/)[0].toLowerCase();
    const handler = registry[name];
    if (handler) return handler(value, shell);

    shell.print(`<span style='color:red;'>Comando não encontrado: ${value}. Digita 'help' para ajuda.</span>`);
}