import { renderGameState, apiErrorLine, offlineError } from "../ui/outputs.js";
import { openChat, printChat, closeChat, chatLine, chatServer } from "./chat.js";
import { setTicket, clearTicket, saveSession, clearSession } from "./sessionGame.js";

const wsProto = () => (window.location.protocol === 'https:' ? 'wss:' : 'ws:');

export function requireGame(shell) {
    const { activeWs, gameToken, inGame } = shell.state;

    if (!inGame || !activeWs || !gameToken) {
        shell.print(offlineError('sem conexão com a sala'));
        return null;
    }
    if (activeWs.readyState !== WebSocket.OPEN) {
        shell.print(offlineError('sem sessão ativa'));
        return null;
    }
    return { ws: activeWs, token: gameToken };
}

export function startGame(shell, roomId, token) {
    const ws = new WebSocket(`${wsProto()}//${window.location.host}/games/ws/${roomId}?token=${token}`);

    shell.state.activeWs = ws;
    shell.state.inGame = false;
    setTicket(shell, { roomId, token });

    ws.onmessage = (event) => {
        let msg;
        try {
            msg = JSON.parse(event.data);
        } catch (err) {
            shell.print(apiErrorLine('ws-parse', err.message));
            return;
        }

        if (msg.status === 'joined') {
            shell.state.inGame = true;
            if (msg.chat) openChat(shell, `Chat da sala: ${msg.room_id}`);
            if (shell.state.myName == null) shell.state.myName = msg.name;
            if (msg.name === shell.state.myName) return;
            printChat(shell, chatServer(`${msg.name} entrou na sala`));
        } else if (msg.status === 'error') {
            shell.print(apiErrorLine(msg.code, msg.message));
        } else if (msg.status === 'state') {
            renderGameState(shell, msg.state);
        } else if (msg.status === 'chat') {
            printChat(shell, chatLine(msg.author, msg.text));
        } else if (msg.status === 'left') {
            shell.state.lastTurnLine = null;
            shell.state.lastOutputState = null;
            printChat(shell, chatServer(`${msg.name} saiu da sala`));
        } else if (msg.status === 'quit') {
            shell.state.inGame = false; 
            shell.state.lastOutputState = null;
            shell.state.lastTurnLine = null;
            clearSession();
            clearTicket(shell);
            ws.close();
        }
    };

    ws.onerror = () => {
        shell.print(`<span style='color:red;'>[Erro] Falha na conexão WebSocket.</span>`);
    };

    ws.onclose = () => {
        if (shell.state.inGame) {
            shell.print(`Desconectado da sala: ${roomId}`);
            saveSession(shell);
        } else if (shell.state.gameToken) {
            shell.print('Expirado - sem acesso à sala');
            clearSession();
        }
        clearTicket(shell);
        shell.state.activeWs = null;
        closeChat(shell);
    };
}