/**
 * Tiny word-level diff for the polish preview's "Resaltar cambios" toggle.
 * Splits both strings on whitespace boundaries (preserving the whitespace as
 * separate tokens), runs a longest-common-subsequence pass, and emits
 * `add`/`rem`/`equal` parts the UI can render with the .diff-add / .diff-rem
 * styles from the design.
 *
 * Good enough for short transcriptions (<1000 words). For longer inputs the
 * LCS table is O(n*m) memory; we'd swap to Myers diff if it ever becomes a
 * problem in practice.
 */

export type DiffPart = {
	kind: 'equal' | 'add' | 'rem';
	text: string;
};

/** Split a string into word-and-whitespace tokens. Each token is either a
 *  run of word chars or a run of non-word chars; preserving the original
 *  spacing means the diff renders without losing layout. */
function tokenize(s: string): string[] {
	if (!s) return [];
	const tokens: string[] = [];
	for (const m of s.matchAll(/(\s+)|([^\s]+)/g)) {
		tokens.push(m[0]);
	}
	return tokens;
}

/**
 * Compute a word-level diff between `before` and `after`.
 * Adjacent parts of the same kind are collapsed into one for cleaner render.
 */
export function wordDiff(before: string, after: string): DiffPart[] {
	const a = tokenize(before);
	const b = tokenize(after);

	// Standard LCS DP table.
	const n = a.length;
	const m = b.length;
	const dp: number[][] = Array.from({ length: n + 1 }, () => new Array(m + 1).fill(0));
	for (let i = 1; i <= n; i++) {
		for (let j = 1; j <= m; j++) {
			if (a[i - 1] === b[j - 1]) {
				dp[i][j] = dp[i - 1][j - 1] + 1;
			} else {
				dp[i][j] = Math.max(dp[i - 1][j], dp[i][j - 1]);
			}
		}
	}

	// Walk back from (n, m) to assemble the diff in order.
	const out: DiffPart[] = [];
	let i = n;
	let j = m;
	while (i > 0 && j > 0) {
		if (a[i - 1] === b[j - 1]) {
			out.push({ kind: 'equal', text: a[i - 1] });
			i--;
			j--;
		} else if (dp[i - 1][j] >= dp[i][j - 1]) {
			out.push({ kind: 'rem', text: a[i - 1] });
			i--;
		} else {
			out.push({ kind: 'add', text: b[j - 1] });
			j--;
		}
	}
	while (i > 0) {
		out.push({ kind: 'rem', text: a[i - 1] });
		i--;
	}
	while (j > 0) {
		out.push({ kind: 'add', text: b[j - 1] });
		j--;
	}
	out.reverse();

	// Collapse adjacent same-kind parts so the renderer doesn't ship one
	// span per word.
	const merged: DiffPart[] = [];
	for (const part of out) {
		const last = merged[merged.length - 1];
		if (last && last.kind === part.kind) {
			last.text += part.text;
		} else {
			merged.push({ ...part });
		}
	}
	return merged;
}
