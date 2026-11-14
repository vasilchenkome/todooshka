import type { ReactNode } from "react";

import { useAuth } from "../../features/auth/AuthContext";
import { useI18n } from "../../i18n";
import { Brand } from "./Brand";
import { LocaleSwitcher } from "./LocaleSwitcher";

type AppBarProps = {
  left?: ReactNode;
  right?: ReactNode;
};

/** App top bar: brand, screen context and the current user. */
export function AppBar({ left, right }: AppBarProps) {
  const { user, logout } = useAuth();
  const { t } = useI18n();

  return (
    <header className="sticky top-0 z-20 flex h-[60px] shrink-0 items-center gap-3 border-b border-ink bg-paper px-4 sm:gap-6 sm:px-5 md:px-7">
      <div className="flex h-6 shrink-0 items-center md:border-r md:border-rule md:pr-6">
        <Brand />
      </div>
      <div className="flex min-w-0 flex-1 items-center gap-3 sm:gap-6">{left}</div>
      <div className="flex shrink-0 items-center gap-3 text-[13px] text-steel">
        {right}
        <LocaleSwitcher />
        <span className="hidden text-ink sm:inline">{user?.username}</span>
        <button
          type="button"
          onClick={() => void logout()}
          className="border-b border-transparent pb-px transition-colors hover:border-rule hover:text-ink"
        >
          {t("common.logout")}
        </button>
      </div>
    </header>
  );
}
