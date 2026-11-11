import { themeCommand } from "../theme.js";
import { fetchKeys } from "../../api/api.js";
import { fetchAndPrint } from "../../ui/fetchAndPrint.js";
import { onExit, setOnExit } from "../kernel.js";

export const defaultRegistry = {};

defaultRegistry['clear'] = (_value, shell) => shell.clear();
defaultRegistry['theme'] = themeCommand;
defaultRegistry['keys'] = (_value, shell) => fetchAndPrint(shell, fetchKeys);
defaultRegistry['exit'] = (_value, shell) => onExit?.(shell);
