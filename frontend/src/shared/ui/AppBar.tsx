import type { ReactNode } from "react";

import { useAuth } from "../../features/auth/AuthContext";
import { Brand } from "./Brand";

type AppBarProps = {
  left?: ReactNode;
};

/** Верхняя полоса приложения: марка, контекст экрана и текущий пользователь. */
export function AppBar({ left }: AppBarProps) {
  const { user, logout } = useAuth();

  return (
    <header className="flex h-[60px] items-center gap-3 border-b border-ink px-4 sm:gap-6 sm:px-5 md:px-7">
      <div className="flex h-6 shrink-0 items-center md:border-r md:border-rule md:pr-6">
        <Brand />
      </div>
      <div className="flex min-w-0 flex-1 items-center gap-3 sm:gap-6">{left}</div>
      <div className="flex shrink-0 items-center gap-3 text-[13px] text-steel">
        <span className="hidden text-ink sm:inline">{user?.username}</span>
        <button
          type="button"
          onClick={() => void logout()}
          className="border-b border-transparent pb-px transition-colors hover:border-rule hover:text-ink"
        >
          выйти
        </button>
      </div>
    </header>
  );
}
