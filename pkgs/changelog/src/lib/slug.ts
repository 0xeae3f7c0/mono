/** Turns a post title into a URL slug: lowercase, ASCII, hyphen-separated. */
export function slugify(title: string): string {
	return title
		.normalize('NFKD')
		.replace(/[̀-ͯ]/g, '')
		.toLowerCase()
		.replace(/[^a-z0-9]+/g, '-')
		.replace(/^-+|-+$/g, '');
}

/** Estimated reading time in whole minutes (at least 1), at 200 words per minute. */
export function readingMinutes(body: string): number {
	const words = body.trim().split(/\s+/).filter(Boolean).length;
	return Math.max(1, Math.ceil(words / 200));
}
