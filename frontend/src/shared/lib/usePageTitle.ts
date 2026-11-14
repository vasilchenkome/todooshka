import { useEffect } from "react";

/** Заголовок вкладки: у доски — имя проекта, у служебных экранов — своё. */
export function usePageTitle(title: string) {
  useEffect(() => {
    document.title = title;
  }, [title]);
}
