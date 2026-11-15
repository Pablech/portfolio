import { fetchAndPrint } from "../../ui/fetchAndPrint.js";
import { fetchCreateGame, fetchJoinGame } from "../../api/api.js";
import { generateReqId } from "../../core/utils.js";
import { wsGameError, renderGameState } from "../../ui/outputs.js";
import { closeChat, openChat, printChat } from "../../core/chat.js";
import { escapeHtml, time } from "../../core/utils.js";

export const gamesRegistry = {};
const wsProto = () => (window.location.protocol === 'https:' ? 'wss:' : 'ws:');

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

export function resumeInput(shell, raw) {
    const saved = shell.state.resume;
    shell.state.resume = null;
    if (raw.toLowerCase() === 'y') {
        shell.state.myName = saved.name;
        startGame(shell, saved.room_id, saved.token);
    } else {
        shell.print('Sessão descartada.')
        sessionStorage.removeItem('game.session');
    }
}

export function routInput(shell, value) {
    const raw = value.trim();

    if (raw.startsWith('/')) {
        if (raw.length === 1) {
            shell.print('use /mensagem para conversar na sala');
            return;
        }
        if (!shell.state.activeWs || shell.state.activeWs.readyState !== WebSocket.OPEN) {
            shell.print('<span style="color:red">[chat] sem conexão com a sala</span>');
            return;
        }
        shell.state.activeWs.send(JSON.stringify({
            action: 'chat',
            data: { token: shell.state.gameToken, text: raw.slice(1).trim() },
        }));
        return;
    }

    if (!shell.state.activeWs || shell.state.activeWs.readyState !== WebSocket.OPEN) {
        shell.print('<span style="color:red">sem conexão com a sala</span>');
        return;
    }
    if (!shell.state.gameToken) { shell.print('sem sessão ativa'); return; }

    shell.state.activeWs.send(JSON.stringify({
        action: 'play',
        data: {
            token: shell.state.gameToken,
            input: raw,
            req_id: generateReqId(),
        }
    }));
}

function requireGame(shell) {
    const { activeWs, gameToken, inGame } = shell.state;

    if (!inGame || !activeWs || !gameToken) {
        shell.print('Não estas em jogo.');
        return null;
    }
    if (activeWs.readyState !== WebSocket.OPEN) {
        shell.print('Ligação fechada, reconnect/join para reconectar.');
        return null;
    }
    return { ws: activeWs, token: gameToken };
}

function nextReqId(shell) {
    const reqId = generateReqId();
    shell.state.lastReqId = reqId;
    return reqId;
}

function startGame(shell, roomId, token) {
    const ws = new WebSocket(`${wsProto()}//${window.location.host}/games/ws/${roomId}?token=${token}`);

    shell.state.activeWs = ws;
    shell.state.gameToken = token;
    shell.state.inGame = false;
    shell.state.lastReqId = null;
    shell.state.gameRoom = roomId;


    ws.onmessage = (event) => {
        let msg;
        try {
            msg = JSON.parse(event.data);
        } catch (err) {
            shell.print(`<span style='color:red;'>[Erro WS]:</span> ${err.message}`);
            return;
        }

        if (msg.status === 'joined') {
            shell.state.inGame = true;
            if (msg.chat) openChat(shell, `Chat da sala: ${msg.room_id}`);
            if (shell.state.myName == null) shell.state.myName = msg.name;
            if (msg.name === shell.state.myName) return;
            printChat(shell, `<span style="opacity:.50">${time()}</span> <span style="opacity:.6">[Server]</span> ${(escapeHtml(msg.name))} entrou na sala`)
        } else if (msg.status === 'error') {
            shell.print(`<span style='color:red;'>[Error] ${msg.message}</span>`)
        } else if (msg.status === 'state') {
            renderGameState(shell, msg.state);
        } else if (msg.status === 'chat') {
            printChat(shell, `<span style="opacity:.50">${time()}</span> <span style="opacity:.6">[${escapeHtml(msg.author)}]</span> ${escapeHtml(msg.text)}`)
        } else if (msg.status === 'left') {
            printChat(shell, `<span style="opacity:.50">${time()}</span> <span style="opacity:.6">[Server]</span> ${escapeHtml(msg.name)} saiu da sala`)
        } else if (msg.status === 'quit') {
            shell.state.gameToken = null;
            shell.state.gameRoom = null;
            shell.state.inGame = false;
            sessionStorage.removeItem('game.session');
        }
    };

    ws.onerror = () => {
        shell.print(`<span style='color:red;'>[Erro] Falha na conexão WebSocket.</span>`);
    };

    ws.onclose = () => {
        if (shell.state.inGame) {
            shell.print(`Desconectado da sala: ${roomId}`);
            sessionStorage.setItem('game.session', JSON.stringify({
                room_id: roomId, token: shell.state.gameToken, name: shell.state.myName,
            }));
        } else if (shell.state.gameToken) {
            shell.print('Expirado - sem acesso à sala');
            sessionStorage.removeItem('game.session');
        }
        shell.state.inGame = false;
        shell.state.activeWs = null;
        shell.state.gameToken = null;
        shell.state.boardPrinter = 0;
        closeChat(shell);
    };
}

gamesRegistry['create'] = async (parsed, shell) => {
    const { flags } = parsed;
    const vsCpu = flags.mode !== 'pvp';
    const body = {
        game: flags.game || 'connect4',
        name: flags.name,
        fig: flags.fig,
        vs_cpu: vsCpu,
    };

    const { data } = await fetchGameAndPrint(shell, () => fetchCreateGame(body), {
        onSuccess: (data) => {
            shell.print(`Sala criada ${data.room_id}`)
            if (vsCpu) {
                shell.print('A jogar contra a CPU.');
            } else {
                shell.print(`Aguardando oponente — partilha o room_id: ${data.room_id}`);
            }
            shell.state.myName = flags.name;
            startGame(shell, data.room_id, data.token);
        },
    });
};

gamesRegistry['join'] = async (parsed, shell) => {
    const { flags } = parsed;

    const { data } = await fetchGameAndPrint(shell,
        () => fetchJoinGame(flags.room, { name: flags.name, fig: flags.fig }),
        {
            onSuccess: (data) => {
                shell.state.myName = data.joined;
                shell.print(`Entraste como ${data.joined}`);
                startGame(shell, data.room_id, data.token);
            },
        });
};


gamesRegistry['rematch'] = async (parsed, shell) => {
    const sess = requireGame(shell);
    if (!sess) return;

    const accept = !parsed.flags.decline;
    sess.ws.send(JSON.stringify({
        action: 'rematch',
        data: { token: sess.token, accept, req_id: nextReqId(shell) },
    }));
};

gamesRegistry['quit'] = async (_parsed, shell) => {
    const sess = requireGame(shell);
    if (!sess) return;
    sessionStorage.removeItem('game.session');
    sess.ws.send(JSON.stringify({
        action: 'quit',
        data: { token: sess.token, },
    }));
};

gamesRegistry['reconnect'] = async (_parsed, shell) => {
    if (shell.state.inGame) {shell.print('Já estas em jogo'); return;}

    let saved;
    try {
        saved = JSON.parse(sessionStorage.getItem('game.session'));
    } catch {}

    if (!saved?.room_id || !saved?.token) {
        shell.print('Sem sessão para reconectar');
        return;
    }

    shell.state.myName = saved.name;
    startGame(shell, saved.room_id, saved.token);
}