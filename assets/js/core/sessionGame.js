import { generateReqId } from "./utils.js";

const SESSION_KEY = 'game.session';

export function setTicket(shell, { roomId, token }) {
    shell.state.gameToken = token;
    shell.state.gameRoom = roomId;
    shell.state.lastReqId = null;
}

export function clearTicket(shell) {
    shell.state.gameToken = null;
    shell.state.gameRoom = null;
    shell.state.inGame = false;
    shell.state.boardPrinter = 0;
}

export function saveSession(shell) {
    const { gameRoom: room_id, gameToken: token, myName: name } = shell.state;
    if (!room_id || !token) return;
    sessionStorage.setItem(SESSION_KEY, JSON.stringify({ room_id, token, name }));
}

export function loadSession() {
    try {
        return JSON.parse(sessionStorage.getItem(SESSION_KEY));
    } catch {
        return null;
    }
}

export function clearSession() {
    sessionStorage.removeItem(SESSION_KEY);
}

export function nextReqId(shell) {
    const reqId = generateReqId();
    shell.state.lastReqId = reqId;
    return reqId;
}