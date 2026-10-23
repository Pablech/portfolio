import { setRepoOwner } from "./repoViews.js";

export function startGitHubPagination(repos, shell) {
    setRepoOwner(shell);
    shell.state.backendRepos = repos;
    shell.state.isPaginating = true;
    shell.state.ghPage = 0;
    shell.print(`Encontrados ${repos.length} repositórios públicos.<br>`);
    renderGitHubPage(shell);
}

export function handlePaginationInput(value, shell) {
    if (value.toLowerCase() === 'q') {
        shell.state.isPaginating = false;
        shell.setPrompt(shell.prompt);
        shell.print("Saída da paginação.");
        return;
    }
    shell.state.ghPage++;
    renderGitHubPage(shell);
}

function renderGitHubPage(shell) {
    const s = shell.state;
    const repos = s.backendRepos;
    const start = s.ghPage * 5;
    const end = start + 5;

    repos.slice(start, end).forEach(repo => {
        shell.print(
            `* <b>${escapeRepo(repo)}</b> ` +
            `<span class="btn-tty" data-action="detalhes" data-repo-id="${repo.id}">[Detalhes]</span>`
        );
    });

    if (end < repos.length) {
        shell.setPrompt("--MAIS--");
    } else {
        shell.print("<br>Fim da lista.");
        s.isPaginating = false;
        shell.setPrompt(shell.prompt);
    }
}

function escapeRepo(repo) {
    const txt = document.createElement('textarea');
    txt.innerHTML = `${repo.name} [${repo.lang}] - ${repo.desc}`;
    return txt.value;
}