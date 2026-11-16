import { wsGameError } from "./outputs.js";

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

export async function fetchGameAndPrint(shell, fn, { onSuccess } = {}) {
    return fetchAndPrint(shell, fn, {
        loading: "A comunicar com o servidor...",
        onSuccess,
        onError: (status, data) => {
            const t = wsGameError(status, data);
            return typeof t === 'function' ? t(data) : t;
        },
    });
}