import { cmdError } from "../ui/outputs.js";

const FLAG_ALIASES = {
    'g': 'game', 'game': 'game',
    'n': 'name', 'name': 'name',
    'f': 'fig', 'fig': 'fig',
    'r': 'room', 'room': 'room',
    'm': 'mode', 'mode': 'mode',
};

const CMD_RULES = {
    'create': { required: [ 'game', 'name', 'fig', 'mode' ] },
    'join': { required: [ 'name', 'fig', 'room' ] },
};

export function cmdParser(shell, input, cmds) {
    const trimmed = input.trim();
    const tokens = trimmed.split(/\s+/);
    
    const cmd = tokens[0] || '';
    if (!cmds.includes(cmd)) { 
        shell.print(cmdError(cmd));
        return null; 
    }

    const rawFlags = {};
    const regex = /--?([a-zA-Z-]+)(?:\s+|=)(?:["']([^"']+)["']|([^\s]+))/g;
    let match;

    while ((match = regex.exec(trimmed)) !== null) {
        const flagName = match[1];
        const flagValue = match[2] !== undefined ? match[2] : match[3];
        const key = FLAG_ALIASES[flagName] || flagName;

        rawFlags[key] = flagValue;
    }

    const rule = CMD_RULES[cmd];
    if (rule && rule.required) {
        const missing = rule.required.filter(flag => !rawFlags[flag]);

        if (missing.length > 0) {
            const formattedMissing = missing.map(key => {
                const aliases = Object.entries(FLAG_ALIASES)
                    .filter(([alias, k]) => k === key)
                    .map(([alias]) => alias.length === 1 ? `-${alias}` : `--${alias}`);
                
                aliases.sort((a, b) => b.length - a.length);
                return aliases.join(' ou ');
            });
            
            shell.print(cmdError(cmd, null, formattedMissing.join(', ')));
            return null;
        }
    }

    return { cmd, flags: rawFlags, args: tokens.slice(1) };
}