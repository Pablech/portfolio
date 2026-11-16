import { cmdError, errNote } from "../ui/outputs.js";

const FLAG_ALIASES = {
    'g': 'game', 'game': 'game',
    'n': 'name', 'name': 'name',
    'f': 'fig', 'fig': 'fig',
    'r': 'room', 'room': 'room',
    'm': 'mode', 'mode': 'mode',
};

const CMD_RULES = {
    'create': {
        required: ['game', 'name', 'fig', 'mode'],
        usage: 'create -gnfm game name fig modo',
    },
    'join': {
        required: ['name', 'fig', 'room'],
        usage: 'join -nfr name fig room',
    },
};

const PLACEHOLDERS = {
    game: 'game', name: 'name', fig: 'fig',
    room: 'room', mode: 'modo',
};

function shortAlias(key) {
    const entries = Object.entries(FLAG_ALIASES).filter(([, k]) => k === key);
    const short = entries.find(([a]) => a.length === 1);
    return short ? short[0] : entries[0][0];
}

function dynUsage(cmd, providedOrder, missing) {
    if (providedOrder.length === 0) return CMD_RULES[cmd].usage;
    const ordered = [
        ...providedOrder,
        ...missing.filter(k => !providedOrder.includes(k)),
    ];
    const letters = ordered.map(shortAlias).join('');
    const fields = ordered.map(k => `${PLACEHOLDERS[k] || k}`).join(' ');
    return `${cmd} -${letters} ${fields}`;
}

function tokenize(raw) {
    const tokens = [];
    const re = /"([^"]*)"|'([^']*)'|(\S+)/g;
    let m;
    while ((m = re.exec(raw)) !== null) {
        tokens.push(m[1] ?? m[2] ?? m[3]);
    }
    return tokens;
}

function flagKeys(body) {
    if (FLAG_ALIASES[body]) return [FLAG_ALIASES[body]];
    return [...body].map(ch => FLAG_ALIASES[ch] || null);
}

export function cmdParser(shell, input, cmds) {
    const tokens = tokenize(input.trim());
    const cmd = tokens[0] || '';

    if (!cmds.includes(cmd)) {
        shell.print(cmdError(cmd));
        return null;
    }

    const flags = {};
    const providedOrder = [];

    let i = 1;
    while (i < tokens.length) {
        const tok = tokens[i];

        if (!tok.startsWith('-')) {
            i++;
            continue;
        }

        const body = tok.replace(/^-{1,2}/, '');
        const keys = flagKeys(body);

        if (keys.length === 0 || keys.some(k => k === null)) {
            shell.print(errNote(`flag inválida: ${tok} (uso: ${CMD_RULES[cmd]?.usage || '--flag valor'})`));
            return null;
        }

        const values = [];
        let j = i + 1;
        while (values.length < keys.length && j < tokens.length && !tokens[j].startsWith('-')) {
            values.push(tokens[j++]);
        }

        if (values.length < keys.length) {
            shell.print(errNote(`faltam ${keys.length - values.length} valor(es) para ${tok}`));
            return null;
        }

        keys.forEach((key, idx) => {
            flags[key] = values[idx];
            if (!providedOrder.includes(key)) providedOrder.push(key);
        });
        i = j;
    }

    const rule = CMD_RULES[cmd];
    if (rule?.required) {
        const missing = rule.required.filter(f => !flags[f]);
        if (missing.length > 0) {
            shell.print(cmdError(cmd, null,
                `precisa de: ${missing.join(', ')} — uso: ${dynUsage(cmd, providedOrder, missing)}`));
            return null;
        }
    }

    return { cmd, flags, args: tokens.slice(1) };
}