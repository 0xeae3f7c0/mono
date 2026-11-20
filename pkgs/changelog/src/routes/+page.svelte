<!-- changelog: Changelog (commit platform, pkgs/changelog) -->
<script lang="ts">
	import { readingMinutes, slugify } from '../lib/slug';

	const posts = [
		{ title: 'Per-version dependency PRs', body: 'Each dependency update now targets one exact version, and older pending updates merge first.' },
		{ title: 'Badges for every repository', body: 'The badges service renders commit and CI badges you can drop into any README.' }
	];
</script>

<h1>commit changelog</h1>
{#each posts as post (post.title)}
	<article id={slugify(post.title)}>
		<h2><a href="#{slugify(post.title)}">{post.title}</a></h2>
		<small>{readingMinutes(post.body)} min read</small>
		<p>{post.body}</p>
	</article>
{/each}
