import { openSidePanel } from "../ui/sidePanel.js";

let ownerShell = null;
export function setRepoOwner(shell) { ownerShell = shell; }

export function openVSCode(shell, repoId, fileIdx) {
    const repo = shell.state.backendRepos.find(r => r.id === repoId);
    if (!repo?.files?.[fileIdx]) return;
    const file = repo.files[fileIdx];

    const coloredCode = file.code
        .replace(/fn |let |mut |pub |impl |struct |println!/g, m => `<span style="font-weight:bold;">${m}</span>`)
        .replace(/".*?"/g, m => `<span>${m}</span>`);

    openSidePanel(
        file.name,
        `<pre style="margin:0; font-family:monospace; color:var(--tty-color);">${coloredCode}</pre>`,
        () => openDetalhes(shell, repoId)
    );
}

export function openDetalhes(shell, repoId) {
    const repo = shell.state.backendRepos.find(r => r.id === repoId);
    if (!repo) return;

    const filesHtml = (repo.files ?? []).map((f, idx) =>
        `|-- <span class="file-link" data-action="vscode" data-repo-id="${repoId}" data-file-idx="${idx}">${f.name}</span>`
    ).join('<br>');

    const content = `
        <p>> <b>Linguagem:</b> ${repo.lang}</p>
        <p>> <b>Descrição:</b> ${repo.desc}</p>
        <p>> <b>Link:</b> <a href="${repo.url}" target="_blank" style="color:var(--tty-color); text-decoration:underline;">${repo.url}</a></p>
        <p>> <b>Árvore de Ficheiros:</b></p>
        ${filesHtml}
    `;

    openSidePanel(`[ ROOT / ${repo.name} ]`, content, null);
}

export function handleSidePanelClick(_e, shell) {
    const el = _e.target.closest('[data-action]');
    if (!el) return;

    const target = ownerShell ?? shell;
    const repoId = Number(el.dataset.repoId);
    const action = el.dataset.action;

    if (action === 'detalhes') openDetalhes(target, repoId);
    else if (action === 'vscode') openVSCode(target, repoId, Number(el.dataset.fileIdx));
}