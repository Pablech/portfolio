import { themeCommand } from "../theme.js";
import { fetchHelp, fetchKeys } from "../../api/api.js";
import { fetchAndPrint } from "../../ui/fetchAndPrint.js";
import { onExit } from "../kernel.js";


export const defaultRegistry = {};

defaultRegistry['clear'] = (_value, shell) => shell.clear();
defaultRegistry['theme'] = themeCommand;
defaultRegistry['keys'] = (_value, shell) => fetchAndPrint(shell, fetchKeys);
defaultRegistry['close'] = (_value, shell) => onExit?.(shell);
defaultRegistry['help'] = (_value, shell) => fetchAndPrint(
    shell, () => fetchHelp(shell.kind === 'games' ? 'games' : 'os'));
