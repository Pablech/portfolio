function makePrompt(dir) {
    return `[pablech@portfolio ${dir}]$`;
}

function makeHeader(dir) {
    return `Portfolio ${dir} v1.0.0`;
}

export function cmdError(value, suggestion = null, missing = null) {
    if (suggestion) {
        return `<span style='color:red;'>[Erro] Commando '${value}' não encontrado. Similar cmd: ${suggestion}.</span>`;
    } else if (missing) {
        return `<span style='color:red;'>[Erro] Commando '${value}' precisa de: ${missing}.</span>`;
    } else {
        return `<span style='color:red;'>[Erro] Comando desconhecido: '${value}'. Digita 'help'.</span>`;
    }
}

export function wsGameError(status, data) {
    const GAME_STATUS = {
        400: (data) => `<span style='color:red;'>${data?.message ?? "Pedido inválido."}</span>`,
        404: `<span style='color:red;'>[Erro] Sala inexistente (expirou?). Pede ao host um convite novo.</span>`,
        409: `<span style='color:red;'>[Erro] Sala cheia ou partida solo (vs CPU).</span>`,
        422: `<span style='color:red;'>[Erro] Pedido malformado — confere os parâmetros.</span>`,
    };
    return GAME_STATUS[status];
}

export function renderGameState(shell, st) {
    if (!st) return;

    const d = st.continue || st.win || st.draw || st.rematch_pending ||
              st.rematch_declined || st;

    if (d.output) {
        if (shell.state.boardsPrinted > 0) shell.clear();
        shell.state.boardsPrinted++;
        shell.print(d.output);
    }

    if (st.continue?.next_player) {
        shell.print(`Turno de: ${st.continue.next_player}`);
    } else if (st.win) {
        shell.print(`Vitória de: ${st.win.winner}`);
        shell.print('rematch para jogar outra vez');
    } else if (st.draw) {
        shell.print('O jogo acabou em empate');
        shell.print('rematch para jogar outra vez');
    } else if (st.rematch_pending) {
        if (st.rematch_pending.who === shell.state.myName) {
            shell.print('Revanche pedida, escreve rematch para aceitar.')
        } else {
            shell.print(`rematch de ${st.rematch_pending.who} pendente de aprovação`);
        }
    } else if (st.rematch_declined) {
        if (st.rematch_declined.by === shell.state.myName) {
            shell.print('Tempo esgotado, encerrando sala.')
        } else {
            shell.print(`${st.rematch_declined.by} não aceitou o pedido de revanche`);
        }
        shell.state.inGame = false;
    }
}

export const DEFAULT_PROMPT = makePrompt('~');
export const DEFAULT_OS_HEADER = makeHeader('OS');
export const DEFAULT_INIT_MSG = 'Escreve "help" para veres os comandos disponíveis.';
export const KIND_OS = 'os';

export const GAME_PROMPT = makePrompt('games');
export const DEFAULT_GAME_HEADER = makeHeader('GAMES');
export const DEFAULT_GAME_MSG = 'Escreve "help" para veres os jogos disponíveis.';
export const KIND_GAME = 'games';
