function makePrompt(dir) {
    return `[pablech@portfolio ${dir}]$`;
}

function makeHeader(dir) {
    return `Portfolio ${dir} v1.0.0`;
}

export function cmdError(value, suggestion = null) {
    if (!suggestion) {
        return `<span style='color:red;'>[!] Comando desconhecido: '${value}'. Digita 'help'.</span>`;
    } else {
        return `<span style='color:yellow;'>[?] Commando '${value}' não encontrado. Cmd mais similar: ${suggestion}.</span>`
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

export const CONN4_PROMPT = makePrompt('games/conn4');
export const DEFAULT_CONN4_HEADER = makeHeader('CONN4');
export const DEFAULT_CONN4_MSG = 'Escreve "help" para veres os comandos disponíveis.';
