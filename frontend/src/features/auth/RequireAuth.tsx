import type { ReactNode } from "react";
import { Navigate } from "react-router-dom";

import { useI18n } from "../../i18n";
import { useAuth } from "./AuthContext";

/** While the session is checked, a quiet placeholder; then either the screen or the login. */
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
