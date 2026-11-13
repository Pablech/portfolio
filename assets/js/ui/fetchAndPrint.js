// assets/js/ui.js
export async function fetchAndPrint(shell, fn, {
        loading = "Carregando... [AGUARDE]",
        onSuccess,
        onError,
    } = {}) {

    const lastPrompt = shell.prompt;

    shell.setPrompt(loading);
    shell.input.disabled = true;

    try {
        const { ok, status, data } = await fn();

        if (ok) {
            if (data?.output) shell.print(data.output);
            await onSuccess?.(data);
        } else {
            const custom = onError?.(status, data);
            shell.print(custom ??
                `<span style='color:red;'>[Erro] Servidor respondeu ${status}.</span>`);
        }

        return { ok, status, data };

    } finally {
        shell.input.disabled = false;
        shell.setPrompt(lastPrompt);
        shell.focus(true);
    }
}