import { fetchHelp, fetchGitHub, sendEmail } from './api.js';
import { makeDraggable, toggleMaximize, toggleMinimize, showStatusPopup, closeAllModals } from './modals.js';

const cmdInput = document.getElementById('cmd-input');
const typedText = document.getElementById('typed-text');
const outputDiv = document.getElementById('output');
const promptLabel = document.getElementById('prompt-label');

const DEFAULT_PROMPT = 'pablech@portfolio:~\$';

let backendRepos = [];
let wizardStep = null;
const wizardData = { from: '', subject: '', body: '' };
const WIZARD_STEPS = [
    { key: 'from', prompt: 'De (teu e-mail):' },
    { key: 'subject', prompt: 'Assunto:' },
    { key: 'body', prompt: 'Mensagem:' }
];

let isPaginating = false;
let ghPage = 0;
let isSending = false;

cmdInput.addEventListener('input', () => {
    typedText.textContent = cmdInput.value;
});

makeDraggable('modal-detalhes', 'header-detalhes');
makeDraggable('modal-vscode', 'header-vscode');

window.toggleMinimize = toggleMinimize;
window.toggleMaximize = toggleMaximize;
window.closeAllModals = () => closeAllModals(cmdInput);
window.closeStatusModal = () => {
    document.getElementById('overlay').style.display = 'none';
    document.getElementById('modal-status').style.display = 'none';
    cmdInput.focus();
};
window.openDetalhes = openDetalhes;
window.openVSCode = openVSCode;
window.closeVSCode = closeVSCode;

function printLine(html) {
    const div = document.createElement('div');
    div.className = 'cmd-output';
    div.innerHTML = html;
    outputDiv.appendChild(div);
    window.scrollTo(0, document.body.scrollHeight);
}

function setPrompt(str) {
    promptLabel.textContent = str;
}

function startGitHubPagination() {
    isPaginating = true;
    ghPage = 0;
    printLine(`Encontrados ${backendRepos.length} repositórios públicos.<br>`);
    renderGitHubPage();
}

function renderGitHubPage() {
    const start = ghPage * 5;
    const end = start + 5;
    const pageRepos = backendRepos.slice(start, end);

    pageRepos.forEach(repo => {
        printLine(`* <b>${repo.name}</b> [${repo.lang}] - ${repo.desc} <span class="btn-tty" onclick="openDetalhes(${repo.id})">[Detalhes]</span>`);
    });

    if (end < backendRepos.length) {
        setPrompt("--MAIS--");
    } else {
        printLine("<br>Fim da lista.");
        isPaginating = false;
        setPrompt(DEFAULT_PROMPT);
    }
}

function startWizard() {
    wizardStep = 0;
    wizardData.from = wizardData.subject = wizardData.body = '';
    printLine("<br>--- MODO DE COMPOSIÇÃO DE E-MAIL ---");
    setPrompt(WIZARD_STEPS[0].prompt);
}

async function finishWizard() {
    wizardStep = null;

    if (!wizardData.body.trim()) {
        printLine("<span style='color:#f7768e;'>[✗] A mensagem está vazia. Abortado.</span>");
        setPrompt(DEFAULT_PROMPT);
        return;
    }

    isSending = true;
    cmdInput.disabled = true;
    setPrompt("ENVIANDO...");

    const progressLine = document.createElement('div');
    progressLine.className = 'cmd-output';
    outputDiv.appendChild(progressLine);

    let step = 0;
    const totalBlocks = 12;
    const progressInterval = setInterval(() => {
        step = (step + 1) % (totalBlocks + 1);
        const filled = "=".repeat(step);
        const empty = " ".repeat(totalBlocks - step);
        progressLine.innerHTML = `<span style="color:#e0af68;">A enviar pacotes SMTP... [${filled}&gt;${empty}]</span>`;
        window.scrollTo(0, document.body.scrollHeight);
    }, 120);

    try {
        const response = await sendEmail(wizardData);

        if (response.status === 429) {
            printLine("<span style='color:#e0af68;'>[!] Calma! Podes enviar outra mensagem daqui a alguns segundos.</span>");
            return;
        }
        if (!response.ok) {
            printLine(`<span style='color:#ff5f56;'>[Erro]: Servidor respondeu ${response.status}.</span>`);
            return;
        }
        
        const data = await response.json();
        clearInterval(progressInterval);
        progressLine.remove();

        const isError = !response.ok || (data.output && (data.output.includes("Erro") || data.output.includes("error")));
        showStatusPopup(isError ? "FALHA NO ENVIO" : "E-MAIL ENVIADO", data.output || "Resposta recebida.", isError);

    } catch (err) {
        clearInterval(progressInterval);
        progressLine.remove();
        showStatusPopup("ERRO DE REDE", "Não foi possível estabelecer ligação com o servidor.", true);
    } finally {
        isSending = false;
        cmdInput.disabled = false;
        setPrompt(DEFAULT_PROMPT);
        cmdInput.focus();
    }
}

function openDetalhes(repoId) {
    const repo = backendRepos.find(r => r.id === repoId);
    if (!repo) return;

    document.getElementById('overlay').style.display = 'block';
    document.getElementById('modal-detalhes').style.display = 'block';
    document.getElementById('detalhes-title').textContent = `[ ROOT / ${repo.name} ]`;
    
    let filesHtml = repo.files.map((f, idx) => 
        `|-- <span class="file-link" onclick="openVSCode(${repoId}, ${idx})">${f.name}</span>`
    ).join('<br>');

    document.getElementById('detalhes-body').innerHTML = `
        <p>> <b>Linguagem:</b> ${repo.lang}</p>
        <p>> <b>Descrição:</b> ${repo.desc}</p>
        <p>> <b>Link:</b> <a href="${repo.url}" target="_blank" style="color:#7aa2f7; text-decoration:underline; text-underline-offset:4px;">${repo.url}</a></p>
        <p>> <b>Árvore de Ficheiros:</b></p>
        ${filesHtml}
    `;
}

function openVSCode(repoId, fileIdx) {
    const repo = backendRepos.find(r => r.id === repoId);
    const file = repo.files[fileIdx];
    
    document.getElementById('modal-detalhes').style.display = 'none';
    document.getElementById('modal-vscode').style.display = 'flex';
    document.getElementById('vscode-filename').textContent = file.name;
    
    const coloredCode = file.code
        .replace(/fn |let |mut |pub |impl |struct |println!/g, match => `<span style="color:#569cd6">${match}</span>`)
        .replace(/".*?"/g, match => `<span style="color:#ce9178">${match}</span>`);

    document.getElementById('vscode-code').innerHTML = coloredCode;
}

function closeVSCode() {
    document.getElementById('modal-vscode').style.display = 'none';
    document.getElementById('modal-detalhes').style.display = 'block';
}

document.addEventListener('keydown', (e) => {
    if (e.key === 'Escape' || e.key === 'Enter') {
        const statusModal = document.getElementById('modal-status');
        if (statusModal && statusModal.style.display === 'block') {
            window.closeStatusModal();
            return;
        }
        closeAllModals(cmdInput);
    }
});

cmdInput.addEventListener('keydown', async (e) => {
    if (e.ctrlKey && e.key.toLowerCase() === 'l') { e.preventDefault(); outputDiv.innerHTML = ''; return; }
    if (e.key !== 'Enter') return;

    const value = cmdInput.value.trim();
    cmdInput.value = '';
    typedText.textContent = ''; 

    if (isPaginating) {
        if (value.toLowerCase() === 'q') {
            isPaginating = false;
            setPrompt(DEFAULT_PROMPT);
            printLine("Saída da paginação.");
            return;
        }
        ghPage++;
        renderGitHubPage();
        return;
    }

    if (wizardStep !== null) {
        if (value.toLowerCase() === ':q') {
            wizardStep = null;
            setPrompt(DEFAULT_PROMPT);
            printLine("Operação cancelada.");
            return;
        }
        
        wizardData[WIZARD_STEPS[wizardStep].key] = value;
        printLine(`${WIZARD_STEPS[wizardStep].prompt} ${value}`);
        
        wizardStep++;
        if (wizardStep < WIZARD_STEPS.length) {
            setPrompt(WIZARD_STEPS[wizardStep].prompt);
        } else {
            finishWizard();
        }
        return;
    }

    printLine(`<span class="prompt">${DEFAULT_PROMPT}</span> ${value}`);
    if (!value) return;

    if (value === 'clear') {
        outputDiv.innerHTML = '';
        return;
    }

    if (value === 'email') {
        startWizard();
        return;
    }

    try {
        if (value === 'help') {
            const data = await fetchHelp();
            if (data.output) printLine(data.output);
            return;
        }

        if (value === 'github') {
            printLine("<br>A ligar à API do GitHub... [AGUARDE]");
            
            cmdInput.disabled = true;
            setPrompt("A CARREGAR...");

            try {
                const data = await fetchGitHub();
                backendRepos = JSON.parse(data.output);
                startGitHubPagination();
            } catch (e) {
                printLine("<span style='color:#f7768e;'>[Erro] Formato de dados do GitHub inválido ou falha na rede.</span>");
                setPrompt(DEFAULT_PROMPT);
            } finally {
                cmdInput.disabled = false;
                cmdInput.focus();
            }
            return;
        }

        printLine(`<span style='color:#f7768e;'>Comando não encontrado: <${value}>. Digita 'help' para ajuda.</span>`);
    } catch (err) {
        printLine("<span style='color:#f7768e;'>[Erro] O núcleo de Rust não responde.</span>");
    }
});