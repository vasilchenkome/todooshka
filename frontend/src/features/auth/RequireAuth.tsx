import type { ReactNode } from "react";
import { Navigate } from "react-router-dom";

import { useI18n } from "../../i18n";
import { useAuth } from "./AuthContext";

/** Пока сессия проверяется — тихая заглушка, потом либо экран, либо вход. */
export function RequireAuth({ children }: { children: ReactNode }) {
  const { user, loading } = useAuth();
  const { t } = useI18n();

  if (loading) {
    return (
      <div className="flex min-h-full items-center justify-center text-[13px] text-steel">
        {t("auth.loading")}
      </div>
    );
  }

  if (!user) {
    return <Navigate to="/login" replace />;
  }

  return <>{children}</>;
}
