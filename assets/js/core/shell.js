export class Shell {
    constructor(root, prompt) {
        this.root = root;
        this.prompt = prompt;
        this.input = root.querySelector('.cmd-input');
        this.typed = root.querySelector('.typed-text');
        this.output = root.querySelector('.output');
        this.promptLabel = root.querySelector('.prompt-label');

        this.state = {
            backendRepos: [],
            isPaginating: false,
            ghPage: 0,
            wizardStep: null,
        };

        this.wizardData = { from: '', subject: '', body: '' };
        this.wizardSteps = [
            { key: 'from',    prompt: 'De (teu e-mail):' },
            { key: 'subject', prompt: 'Assunto:' },
            { key: 'body',    prompt: 'Mensagem:' },
        ];
    }

    print(html) {
        const div = document.createElement('div');
        div.className = 'cmd-output';
        div.innerHTML = html;
        this.output.appendChild(div);
        if (!window.getSelection().toString()) this.root.scrollTop = this.root.scrollHeight;
    }

    setPrompt(str) { this.promptLabel.textContent = str; }
    focus()        { this.input.focus({ preventScroll: true }); }
    clear()        { this.output.innerHTML = ''; }
}