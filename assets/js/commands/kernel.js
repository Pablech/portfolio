import { handleWizardInput } from "./email.js";
import { handlePaginationInput } from "./github.js";
import { KIND_GAME, KIND_OS } from "../ui/outputs.js";
import { cmdError } from "../ui/outputs.js";
import { suggestCmd, generateReqId } from "../core/utils.js";
import { defaultRegistry } from "./libs/default.js";
import { siteRegistry } from "./libs/site.js";
import { gamesRegistry } from "./libs/games.js";
import { cmdParser } from "../core/parser.js";

const registriesByKind = { 
    [KIND_OS]: { ...defaultRegistry, ...siteRegistry },
    [KIND_GAME]: { ...defaultRegistry, ...gamesRegistry },
};

export let onNew = null;
export let onExit = null;

export function setOnNew(fn) { onNew = fn; }
export function setOnExit(fn) { onExit = fn; }

export function getCommandNames(shell) { 
    const reg = registriesByKind[shell?.kind] || registriesByKind[KIND_OS];
    return Object.keys(reg); 
}

export async function routeCommand(value, shell) {
    const registry = registriesByKind[shell.kind] || registriesByKind[KIND_OS];
    const allCommands = Object.keys(registry);
    const s = shell.state;

    if (s.isPaginating)   { handlePaginationInput(value, shell); return; }
    if (s.wizardStep !== null) { handleWizardInput(value, shell); return; }
    
    if (!value) return;
    
    const name = value.split(/\s+/)[0];
    const handler = registry[name];

    if (!shell.state.inGame || allCommands.includes(name)) {
        shell.print(`<span class="prompt">${shell.prompt}</span>${value}`);
    };

    if (handler) {
        const parsed = cmdParser(shell, value, allCommands);
        if (!parsed) return;
        return handler(parsed, shell);
    } else if (s.inGame) {
        if (!value) return;

        if (shell.state.activeWs && shell.state.activeWs.readyState === WebSocket.OPEN) {
            shell.state.activeWs.send(JSON.stringify({
                action: "play",
                data: {
                    token: shell.state.gameToken,
                    input: value.trim(),
                    req_id: generateReqId(),
                }
            }));
        }
        return;
    }

    let candidates = suggestCmd(name, allCommands);
    if (candidates?.length) {
        const suggestions = candidates.map(c => `\`${c.cmd}\``).join(', ');
        shell.print(cmdError(value, suggestions));
    } else {
        shell.print(cmdError(value));
    }
}