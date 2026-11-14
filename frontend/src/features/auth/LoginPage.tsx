import { useState } from "react";
import type { FormEvent } from "react";
import { Navigate } from "react-router-dom";

import { ApiError } from "../../api/client";
import { useI18n } from "../../i18n";
import { Brand } from "../../shared/ui/Brand";
import { LocaleSwitcher } from "../../shared/ui/LocaleSwitcher";
import { Field } from "../../shared/ui/Field";
import { usePageTitle } from "../../shared/lib/usePageTitle";
import { useAuth } from "./AuthContext";

export default function LoginPage() {
  const { user, login } = useAuth();
  const { t } = useI18n();
  usePageTitle(t("page.login"));
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
      setError(caught instanceof ApiError ? caught.message : t("errors.login"));
    } finally {
      setBusy(false);
    }
  }

  return (
    <div className="flex min-h-full flex-col">
      <header className="flex h-[60px] items-center border-b border-ink px-5 md:px-7">
        <Brand />
        <LocaleSwitcher className="ml-auto" />
      </header>

      <main className="flex flex-1 justify-center px-5 py-10 md:items-center md:px-12 md:py-16">
        <div className="w-full max-w-[360px]">
          <h1 className="font-display text-[32px] font-semibold tracking-[-0.02em]">{t("auth.loginTitle")}</h1>
          <p className="mt-1.5 text-[13px] text-steel">{t("auth.loginSubtitle")}</p>

          <form className="mt-10 space-y-7" onSubmit={submit}>
            <Field
              label={t("auth.username")}
              name="username"
              autoComplete="username"
              autoFocus
              value={username}
              onChange={(event) => setUsername(event.target.value)}
            />
            <Field
              label={t("auth.password")}
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
              {busy ? t("auth.signingIn") : t("auth.signIn")}
            </button>
          </form>
        </div>
      </main>
    </div>
  );
}
