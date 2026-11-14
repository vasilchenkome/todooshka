import { useState } from "react";
import type { FormEvent } from "react";
import { Navigate } from "react-router-dom";

import { ApiError } from "../../api/client";
import { Brand } from "../../shared/ui/Brand";
import { Field } from "../../shared/ui/Field";
import { usePageTitle } from "../../shared/lib/usePageTitle";
import { useAuth } from "./AuthContext";

export default function LoginPage() {
  const { user, login } = useAuth();
  usePageTitle("Вход — todooshka");
  const [username, setUsername] = useState("");
  const [password, setPassword] = useState("");
  const [error, setError] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);

  if (user) {
    return <Navigate to="/" replace />;
  }

  async function submit(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    setBusy(true);
    setError(null);

    try {
      await login(username, password);
    } catch (caught) {
      setError(caught instanceof ApiError ? caught.message : "не удалось войти");
    } finally {
      setBusy(false);
    }
  }

  return (
    <div className="flex min-h-full flex-col">
      <header className="flex h-[60px] items-center border-b border-ink px-5 md:px-7">
        <Brand />
      </header>

      <main className="flex-1 px-5 py-10 md:px-12 md:py-16">
        <div className="w-full max-w-[360px]">
          <h1 className="font-display text-[32px] font-semibold tracking-[-0.02em]">Вход</h1>
          <p className="mt-1.5 text-[13px] text-steel">Одна учётная запись — своя доска.</p>

          <form className="mt-10 space-y-7" onSubmit={submit}>
            <Field
              label="Логин"
              name="username"
              autoComplete="username"
              autoFocus
              value={username}
              onChange={(event) => setUsername(event.target.value)}
            />
            <Field
              label="Пароль"
              name="password"
              type="password"
              autoComplete="current-password"
              value={password}
              onChange={(event) => setPassword(event.target.value)}
            />

            {error && (
              <p role="alert" className="border-l-2 border-signal pl-3 text-[13px]">
                {error}
              </p>
            )}

            <button
              type="submit"
              disabled={busy || username.trim() === "" || password === ""}
              className="w-full bg-ink px-4 py-3 text-[15px] text-paper transition-colors hover:bg-signal disabled:cursor-not-allowed disabled:opacity-60"
            >
              {busy ? "вхожу…" : "Войти"}
            </button>
          </form>
        </div>
      </main>
    </div>
  );
}
