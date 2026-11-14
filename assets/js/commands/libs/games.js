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
        shell.print('Ligação fechada, reconecta com join.');
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
    shell.state.inGame = true;
    shell.state.lastReqId = null;

    ws.onmessage = (event) => {
        let msg;
        try {
            msg = JSON.parse(event.data);
        } catch (err) {
            shell.print(`<span style='color:red;'>[Erro WS]:</span> ${err.message}`);
            return;
        }

        if (msg.chat) openChat(shell);

        if (msg.status === 'joined') {
            if (msg.chat) openChat(shell, `Chat da sala: ${msg.room_id}`);
            if (msg.name === shell.state.myName) return;
            printChat(shell, `<span style="opacity:.50">${time()}</span> <span style="opacity:.6">[Server]</span> ${msg.name} entrou na sala`)
        } else if (msg.status === 'error') {
            shell.print(`<span style='color:red;'>[Error] ${msg.message}</span>`)
        } else if (msg.status === 'state') {
            renderGameState(shell, msg.state);
        } else if (msg.status === 'chat') {
            printChat(shell, `<span style="opacity:.50">${time()}</span> <span style="opacity:.6">[${escapeHtml(msg.author)}]</span> ${escapeHtml(msg.text)}`)
        }
    };

    ws.onerror = () => {
        shell.print(`<span style='color:red;'>[Erro] Falha na conexão WebSocket.</span>`);
    };

    ws.onclose = () => {
        shell.print(`Desconectado da sala: ${roomId}`);
        shell.state.inGame = false;
        shell.state.activeWs = null;
        shell.state.gameToken = null;
    }
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
                shell.state.myName = data.joined;;
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
    closeChat(shell);
    sess.ws.close();
};