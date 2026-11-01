export async function fetchAndPrint(shell, fn) {
    shell.setPrompt("Carregando... [AGUARDE]");
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