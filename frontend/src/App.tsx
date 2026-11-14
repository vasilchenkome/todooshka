import { useEffect, useState } from "react";

type HealthState = "loading" | "ok" | "unreachable";

/**
 * Заглушка этапа 0: проверяет, что Vite-прокси и health-эндпоинт связаны.
 * Экраны и дизайн появятся на этапе 4.
 */
export default function App() {
  const [health, setHealth] = useState<HealthState>("loading");

  useEffect(() => {
    let cancelled = false;

    async function probe() {
      try {
        const response = await fetch("/api/health");
        if (!cancelled) {
          setHealth(response.ok ? "ok" : "unreachable");
        }
      } catch {
        if (!cancelled) {
          setHealth("unreachable");
        }
      }
    }

    void probe();
    return () => {
      cancelled = true;
    };
  }, []);

  return (
    <main>
      <h1>Todooshka</h1>
      <p>бэкенд: {health === "loading" ? "проверяю…" : health === "ok" ? "ok" : "недоступен"}</p>
    </main>
  );
}
