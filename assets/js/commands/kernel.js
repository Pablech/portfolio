import { handleWizardInput } from "./email.js";
import { handlePaginationInput } from "./github.js";
import { suggestCmd } from "../core/utils.js";
import { defaultRegistry } from "./libs/default.js";
import { siteRegistry } from "./libs/site.js";
import { gamesRegistry, resumeInput, routInput } from "./libs/games.js";
import { cmdParser } from "../core/parser.js";
import { GAME_PROMPT, DEFAULT_GAME_HEADER, DEFAULT_GAME_MSG, KIND_GAME, KIND_OS, cmdError } from "../ui/outputs.js";

const registriesByKind = { 
    [KIND_OS]: { ...defaultRegistry, ...siteRegistry },
    [KIND_GAME]: { ...defaultRegistry, ...gamesRegistry },
};

export let onNew = null;
export let onExit = null;
export let spawner = null;

export function setSpawner(fn) { spawner = fn; }
export function setOnNew(fn) { onNew = fn; }
export function setOnExit(fn) { onExit = fn; }

export function getCommandNames(shell) { 
    const reg = registriesByKind[shell?.kind] || registriesByKind[KIND_OS];
    return Object.keys(reg); 
}

export function openGamesShell() {
    return spawner({
        prompt: GAME_PROMPT,
        header: DEFAULT_GAME_HEADER,
        welcome: DEFAULT_GAME_MSG,
        kind: KIND_GAME,
    });
}

export async function routeCommand(value, shell) {
    const registry = registriesByKind[shell.kind] || registriesByKind[KIND_OS];
    const allCommands = Object.keys(registry);
    const s = shell.state;

    if (s.isPaginating)   { handlePaginationInput(value, shell); return; }
    if (s.wizardStep !== null) { handleWizardInput(value, shell); return; }
    if (s.resume) { resumeInput(shell, value); return; }
    
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
        routInput(shell, value);
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