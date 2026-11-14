import { useEffect } from "react";

/** Tab title: the project name on the board, a screen-specific one elsewhere. */
export function usePageTitle(title: string) {
  useEffect(() => {
    document.title = title;
  }, [title]);
}
