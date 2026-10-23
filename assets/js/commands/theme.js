const THEMES = { 'tty': 'tty', 'white': 'white', 'matrix': 'matrix', 'pink': 'pink' };

export function applyTheme(name) {
    if (!THEMES[name]) return false;
    document.body.className = `theme-${THEMES[name]}`;
    localStorage.setItem('portfolio_theme', THEMES[name]);
    return true;
}

export function restoreTheme() {
    const saved = localStorage.getItem('portfolio_theme');
    if (saved && THEMES[saved]) document.body.className = `theme-${saved}`;
}

export function themeCommand(value, shell) {
    const name = value.split(' ')[1];
    if (!applyTheme(name)) {
        shell.print(`<span style='color:red;'>Tema desconhecido. Utiliza: ${Object.keys(THEMES).join(', ')}</span>`);
    }
}