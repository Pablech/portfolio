import { fetchGameAndPrint } from "../../ui/fetchAndPrint.js"
import { fetchCreateGame, fetchJoinGame } from "../../api/api.js";
import { generateReqId } from "../../core/utils.js";
import { startGame, requireGame } from "../../core/websockets.js";
import { nextReqId, clearSession, loadSession } from "../../core/sessionGame.js";

export const gamesRegistry = {};

export function resumeInput(shell, raw) {
    const saved = shell.state.resume;
    shell.state.resume = null;
    if (raw.toLowerCase() === 'y') {
        shell.state.myName = saved.name;
        startGame(shell, saved.room_id, saved.token);
    } else {
        shell.print('Sessão descartada.');
        clearSession();
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

gamesRegistry['create'] = async (parsed, shell) => {
    const { flags } = parsed;
    const vsCpu = flags.mode !== 'pvp';
    const body = {
        game: flags.game || 'connect4',
        name: flags.name,
        fig: flags.fig,
        vs_cpu: vsCpu,
    };

    await fetchGameAndPrint(shell, () => fetchCreateGame(body), {
        onSuccess: (data) => {
            shell.print(`Sala criada ${data.room_id}`);
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

    await fetchGameAndPrint(shell,
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

    sess.ws.send(JSON.stringify({
        action: 'rematch',
        data: { token: sess.token, accept: !parsed.flags.decline, req_id: nextReqId(shell) },
    }));
};

gamesRegistry['quit'] = async (_parsed, shell) => {
    const sess = requireGame(shell);
    if (!sess) return;
    clearSession();
    sess.ws.send(JSON.stringify({
        action: 'quit',
        data: { token: sess.token },
    }));
};

gamesRegistry['reconnect'] = async (_parsed, shell) => {
    if (shell.state.inGame) { shell.print('Já estas em jogo'); return; }

    const saved = loadSession();
    if (!saved?.room_id || !saved?.token) {
        shell.print('Sem sessão para reconectar');
        return;
    }

    shell.state.myName = saved.name;
    startGame(shell, saved.room_id, saved.token);
};