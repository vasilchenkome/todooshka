import { Link } from "react-router-dom";

/** Знак и имя приложения: марка есть на всех экранах. */
export function Brand() {
  return (
    <Link
      to="/"
      className="flex items-center font-display text-base font-semibold tracking-[-0.01em]"
    >
      <span aria-hidden="true" className="mr-0 h-2 w-2 bg-signal sm:mr-2" />
      <span className="hidden sm:inline">todooshka</span>
    </Link>
  );
}
