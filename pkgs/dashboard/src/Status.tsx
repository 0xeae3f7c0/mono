import { useQuery } from "@tanstack/react-query";

export type StatusResponse = {
  repository: string;
  latestRun: { name: string; status: string; conclusion: string | null; url: string } | null;
};

/** Base URL of web/status-worker; override with VITE_STATUS_URL. */
const STATUS_URL = import.meta.env.VITE_STATUS_URL ?? "http://localhost:8787";

async function fetchStatus(): Promise<StatusResponse> {
  const res = await fetch(`${STATUS_URL}/status`);
  if (!res.ok) throw new Error(`status worker returned ${res.status}`);
  return res.json();
}

/** Shows the latest CI run reported by the status worker. */
export function Status() {
  const { data, error, isPending } = useQuery({ queryKey: ["status"], queryFn: fetchStatus, refetchInterval: 60_000 });

  if (isPending) return <p role="status">Loading CI status…</p>;
  if (error) return <p role="alert">CI status unavailable: {error.message}</p>;
  if (!data.latestRun) return <p role="status">{data.repository}: no runs yet</p>;
  const { name, conclusion, status, url } = data.latestRun;
  return (
    <p role="status">
      {data.repository}: <a href={url}>{name}</a> is {conclusion ?? status}
    </p>
  );
}
