import { showStatusPopup } from "../ui/statusPopup.js";
import { sendEmail } from "../api/api.js";

export function startWizard(shell) {
    shell.state.wizardStep = 0;
    shell.wizardData.from = shell.wizardData.subject = shell.wizardData.body = '';

    shell.print("<br>--- MODO DE COMPOSIÇÃO DE E-MAIL (Digita '-q' para cancelar) ---");
    shell.setPrompt(shell.wizardSteps[0].prompt);
}

export function handleWizardInput(value, shell) {
    const s = shell.state;
    const steps = shell.wizardSteps;

    if (value.toLowerCase() === '-q') {
        s.wizardStep = null;
        shell.setPrompt(shell.prompt);
        shell.print("<span style='color:red;'>[!] Envio de e-mail cancelado pelo utilizador.</span>");
        return;
    }

    shell.wizardData[steps[s.wizardStep].key] = value;
    shell.print(`${steps[s.wizardStep].prompt} ${value}`);

    s.wizardStep++;
    if (s.wizardStep < steps.length) {
        shell.setPrompt(steps[s.wizardStep].prompt);
    } else {
        finishWizard(shell);
    }
}

async function finishWizard(shell) {
    const s = shell.state;
    s.wizardStep = null;

    if (!shell.wizardData.body.trim()) {
        shell.print("<span style='color:red;'>[✗] A mensagem está vazia. Abortado.</span>");
        shell.setPrompt(shell.prompt);
        return;
    }

    shell.input.disabled = true;
    shell.setPrompt("ENVIANDO...");

    const progressLine = document.createElement('div');
    progressLine.className = 'cmd-output';
    shell.output.appendChild(progressLine);

    let step = 0;
    const totalBlocks = 12;
    const progressInterval = setInterval(() => {
        step = (step + 1) % (totalBlocks + 1);
        const filled = "=".repeat(step);
        const empty = " ".repeat(totalBlocks - step);
        progressLine.innerHTML = `<span style="color:var(--tty-color);">A enviar pacotes SMTP... [${filled}&gt;${empty}]</span>`;
        shell.output.scrollTop = shell.output.scrollHeight;
    }, 120);

    try {
        const { status, ok, data } = await sendEmail(shell.wizardData);

        clearInterval(progressInterval);
        progressLine.remove();

        if (status === 429) {
            shell.print("<span style='color:red;'>[!] Calma! Podes enviar outra mensagem daqui a alguns segundos.</span>");
            return;
        }

        const isError = !ok || (data?.output &&
            (data.output.includes("Erro") || data.output.includes("error")));

        showStatusPopup(
            isError ? "FALHA NO ENVIO" : "E-MAIL ENVIADO",
            data?.output || "Resposta recebida.",
            isError
        );
    } catch (_err) {
        clearInterval(progressInterval);
        progressLine.remove();
        showStatusPopup("ERRO DE REDE", "Não foi possível estabelecer ligação com o servidor.", true);
    } finally {
        shell.input.disabled = false;
        shell.setPrompt(shell.prompt);
        shell.focus();
    }
}