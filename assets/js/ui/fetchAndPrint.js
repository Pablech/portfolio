import { apiErrorLine } from "./outputs.js";

export async function fetchAndPrint(shell, fn, {
        loading = "Carregando... [AGUARDE]",
        onSuccess,
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
            shell.print(apiErrorLine(
                data?.status ?? status,
                data?.message ?? 'erro sem corpo no servidor',
            ));
        }

        return { ok, status, data };

    } finally {
        shell.input.disabled = false;
        shell.setPrompt(lastPrompt);
        shell.focus(true);
    }
}

export async function fetchGameAndPrint(shell, fn, { onSuccess } = {}) {
    return fetchAndPrint(shell, fn, {
        loading: "A comunicar com o servidor...",
        onSuccess,
    });
}