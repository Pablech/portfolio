export function escapeHtml(s) {
    return s.replace(/&/g, '&amp;').replace(/</g, '&lt;')
            .replace(/>/g, '&gt;').replace(/"/g, '&quot;');
}

function levenshtein(a, b) {
    const lenA = a.length;
    const lenB = b.length;

    if (lenA === 0) return lenB;
    if (lenB === 0) return lenA;

    const dp = Array(lenA + 1);
    for (let i = 0; i <= lenA; i++) dp[i] = Array(lenB + 1);

    for (let i = 0; i <= lenA; i++) dp[i][0] = i;
    for (let j = 0; j <= lenB; j++) dp[0][j] = j;

    for (let i = 1; i <= lenA; i++) {
        for (let j = 1; j <= lenB; j++) {
            const cost = a[i - 1] === b[j - 1] ? 0 : 1;
            dp[i][j] = Math.min(
                dp[i - 1][j] + 1,
                dp[i][j - 1] + 1,
                dp[i - 1][j - 1] + cost
            );

            if (
                i > 1 && j > 1 &&
                a[i - 1] === b[j - 2] &&
                a[i - 2] === b[j - 1]
            ) {
                dp[i][j] = Math.min(dp[i][j], dp[i - 2][j - 2] + cost);
            }
        }
    }

    return dp[lenA][lenB];
}

function commonPrefixLen(a, b) {
    let i = 0;
    while (i < a.length && i < b.length && a[i] === b[i]) i++;
    return i;
}

export function suggestCmd(name, cmds) {
    const typed = name.toLowerCase();

    const exact = cmds
        .filter(c => c.startsWith(typed))
        .map(cmd => ({ cmd, lcp: typed.length }));
    if (exact.length) return exact;

    const candidates = cmds
        .map(cmd => ({
            cmd,
            lcp: commonPrefixLen(typed, cmd),
            dist: levenshtein(typed, cmd),
        }))
        .filter(({ dist }) => dist > 0 && dist <= 2)
        .sort((a, b) => b.lcp - a.lcp || a.dist - b.dist);

    return candidates.length ? [candidates[0]] : null;
}

export function commonPrefix(strings) {
    if (!strings.length) return '';
    return strings.reduce((acc, s) => {
        let i = 0;
        while (i < acc.length && i < s.length && acc[i] === s[i]) i++;
        return acc.slice(0, i);
    })
}

export function generateReqId() {
    return crypto.randomUUID();
}