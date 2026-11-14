import { GAME_PROMPT, DEFAULT_GAME_HEADER, DEFAULT_GAME_MSG, KIND_GAME } from "../../ui/outputs.js";
import { startWizard } from "../email.js";
import { fetchAndPrint } from "../../ui/fetchAndPrint.js";
import { fetchAbout, fetchGitHub, fetchLinks } from "../../api/api.js";
import { startGitHubPagination } from "../github.js";
import { onNew } from "../kernel.js";

export const siteRegistry = {};

siteRegistry['email'] = (_value, shell) => startWizard(shell);
siteRegistry['about'] = (_value, shell) => fetchAndPrint(shell, fetchAbout);

siteRegistry['github'] = async (_value, shell) => {
    const s = shell.state;

    shell.print("A ligar a API do GitHub... [AGUARDE]");
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

siteRegistry['resume'] = (_value, shell) => {
    shell.print("A abrir o currículo... [PDF]");
    const a = document.createElement('a');
    a.href = '/api/resume';
    a.target = '_blank';
    a.rel = 'noopener';
    a.click();
};

siteRegistry['games'] = () => onNew?.({
    prompt: GAME_PROMPT,
    header: DEFAULT_GAME_HEADER,
    welcome: DEFAULT_GAME_MSG,
    kind: KIND_GAME,
});

siteRegistry['links'] = async (_value, shell) => {    
    const lastPrompt = shell.prompt;
    shell.setPrompt("Carregando... [AGUARDE]");
    shell.input.disabled = true;
    
    try {
        const { ok, data } = await fetchLinks();
        if (!ok || !Array.isArray(data) || data.length === 0) {
        shell.print('<span style="color:red;">[Erro] Sem links disponíveis.</span>');
            return; 
        } 

        const pad = Math.max(...data.map(l => l.label.length));
        const html = data.map(({ label, mask, url }) =>
            `${label.padEnd(pad)}: <a href="${url}" target="_blank" ` +
            `rel="noopener noreferrer" style="color:var(--tty-color);">${mask}</a>`
        ).join('<br>');
        shell.print(html);
    } finally {
        shell.input.disabled = false;
        shell.setPrompt(lastPrompt);
        shell.focus(true);
    }
}