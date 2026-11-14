async function request(route, options = {}) {
    try {
        const response = await fetch(route, { cache: 'no-store', ...options });
        const data = await response.json().catch(() => null);
        return { status: response.status, ok: response.ok, data };
    } catch (err) {
        return { status: 0, ok: false, data: null, error: err };
    }
}

function jsonBody(body) {
    return {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(body),
    };
}

export const fetchHelp = (kind) => request(`/api/help/${kind}`);
export const fetchGitHub = () => request('/api/github');
export const fetchAbout = () => request('/api/about');
export const fetchKeys = () => request('/api/keys');
export const sendEmail = (wizardData) => request('/api/send-email', jsonBody(wizardData));
export const fetchCreateGame = (body) => request('/games/rooms', jsonBody(body));
export const fetchJoinGame = (room, body) => request(`/games/rooms/${room}/join`, jsonBody(body));
export const fetchLinks = () => request('/api/links');