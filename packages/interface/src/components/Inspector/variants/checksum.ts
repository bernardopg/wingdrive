/**
 * Pulls a SHA-256 digest out of what people paste: a bare hex string, a
 * `sha256:` prefixed one, a `sha256sum`/SHA256SUMS line (`<hash>  <name>`), a
 * BSD tag line (`SHA256 (name) = <hash>`) or `openssl`/`certutil` output.
 * Returns null when no 64-character hex digest is present, or when the text
 * holds two different ones and picking either would be a guess.
 */
export function parseExpectedSha256(input: string): string | null {
	// Hex runs are whole tokens, so a 65-character run or part of a longer word never counts.
	const digests = new Set(
		(input.match(/(?<![0-9a-z])[0-9a-f]{64}(?![0-9a-z])/gi) ?? []).map((digest) =>
			digest.toLowerCase()
		)
	);
	return digests.size === 1 ? [...digests][0]! : null;
}

/** 'match' or 'mismatch' against the pasted text, or 'invalid' when it holds no digest. */
export function compareSha256(actual: string, expected: string): 'match' | 'mismatch' | 'invalid' {
	const parsed = parseExpectedSha256(expected);
	if (!parsed) return 'invalid';
	return parsed === actual.toLowerCase() ? 'match' : 'mismatch';
}
