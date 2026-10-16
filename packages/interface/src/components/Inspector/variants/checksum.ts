/**
 * Pulls a SHA-256 digest out of what people paste: a bare hex string, a
 * `sha256:` prefixed one, or a `sha256sum`/SHA256SUMS line (`<hash>  <name>`).
 * Returns null when no 64-character hex digest is present.
 */
export function parseExpectedSha256(input: string): string | null {
	const match = input.trim().match(/^(?:sha256[:=]\s*)?([0-9a-f]{64})(?:\s|$)/i);
	return match ? match[1]!.toLowerCase() : null;
}

/** 'match' or 'mismatch' against the pasted text, or 'invalid' when it holds no digest. */
export function compareSha256(actual: string, expected: string): 'match' | 'mismatch' | 'invalid' {
	const parsed = parseExpectedSha256(expected);
	if (!parsed) return 'invalid';
	return parsed === actual.toLowerCase() ? 'match' : 'mismatch';
}
