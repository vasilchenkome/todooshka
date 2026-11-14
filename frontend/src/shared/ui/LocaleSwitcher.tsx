import { useI18n } from "../../i18n";
import { LOCALES } from "../../i18n/types";

type LocaleSwitcherProps = {
  className?: string;
};

export function LocaleSwitcher({ className = "" }: LocaleSwitcherProps) {
  const { locale, setLocale, t } = useI18n();

  return (
    <div
      role="group"
      aria-label={t("common.language")}
      className={`flex items-center gap-1 text-[11px] tracking-[0.08em] ${className}`}
    >
      {LOCALES.map((value, index) => (
        <span key={value} className="flex items-center gap-1">
          {index > 0 && <span className="text-rule">/</span>}
          <button
            type="button"
            onClick={() => setLocale(value)}
            aria-pressed={locale === value}
            className={`transition-colors ${
              locale === value
                ? "text-ink underline decoration-signal underline-offset-2"
                : "text-steel hover:text-ink"
            }`}
          >
            {value.toUpperCase()}
          </button>
        </span>
      ))}
    </div>
  );
}
