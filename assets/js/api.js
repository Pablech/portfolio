export async function fetchHelp() {
    const response = await fetch('/api/help', {cache: 'no-store'});
    return response.json();
}

export async function fetchGitHub() {
    const response = await fetch('/api/github', {cache: 'no-store'});
    return response.json();
}

export async function sendEmail(wizardData) {
    return await fetch('/api/send-email', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(wizardData)
    });
}