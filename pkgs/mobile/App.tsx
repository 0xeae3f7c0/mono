// mobile: CI status on phones (commit platform, pkgs/mobile)
import { StatusBar } from 'expo-status-bar';
import { useEffect, useState } from 'react';
import { StyleSheet, Text, View } from 'react-native';

type Run = { name: string; conclusion: string | null; status: string };

/** Base URL of the status worker; set EXPO_PUBLIC_STATUS_URL for real devices. */
const STATUS_URL = process.env.EXPO_PUBLIC_STATUS_URL ?? 'http://localhost:8787';

export default function App() {
  const [run, setRun] = useState<Run | null>(null);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    fetch(`${STATUS_URL}/status`)
      .then((r) => (r.ok ? r.json() : Promise.reject(new Error(`HTTP ${r.status}`))))
      .then((body) => setRun(body.latestRun))
      .catch((e: Error) => setError(e.message));
  }, []);

  return (
    <View style={styles.container}>
      <Text style={styles.title}>commit</Text>
      {error ? (
        <Text>Status unavailable: {error}</Text>
      ) : run ? (
        <Text>
          {run.name}: {run.conclusion ?? run.status}
        </Text>
      ) : (
        <Text>Loading CI status…</Text>
      )}
      <StatusBar style="auto" />
    </View>
  );
}

const styles = StyleSheet.create({
  container: { flex: 1, backgroundColor: '#fff', alignItems: 'center', justifyContent: 'center', gap: 8 },
  title: { fontSize: 24, fontWeight: '600' },
});
