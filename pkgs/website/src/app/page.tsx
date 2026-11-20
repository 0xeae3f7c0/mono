import styles from "./page.module.css";

const parts = [
  ["Collect", "webhooks verifies GitHub events and feeds events-api; gitstat and crawl fill in history and changelogs."],
  ["Analyze", "sql, summarize, search and forecast turn raw events into trends, rankings and early warnings."],
  ["Show", "dashboard, admin, the desktop and mobile apps, badges and the embeddable widget put numbers where teams look."],
  ["Report", "pipelines builds the weekly report, reports-pdf renders it and archive keeps every edition."],
];

export default function Home() {
  return (
    <main className={styles.main}>
      <h1>commit</h1>
      <p>A self-hosted platform for engineering activity: what changed, who shipped it and whether CI is healthy.</p>
      <ul>
        {parts.map(([title, text]) => (
          <li key={title}>
            <strong>{title}.</strong> {text}
          </li>
        ))}
      </ul>
      <p>
        <a href="https://github.com/0xeae3f7c0/mono">Source on GitHub</a>
      </p>
    </main>
  );
}
