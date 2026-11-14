import { useQuery } from "@tanstack/react-query";
import { Link } from "react-router-dom";

import { api } from "../../api/client";
import type { AgendaTask } from "../../api/types";
import { useI18n } from "../../i18n";
import { formatLongDate } from "../../i18n/format";
import { usePageTitle } from "../../shared/lib/usePageTitle";
import { AppBar } from "../../shared/ui/AppBar";
import { dueLabel, priorityLabel } from "../board/boardModel";
import { parseDateKey, todayLocalDate } from "../../shared/lib/dueCalendar";

function Section({ title, entries }: { title: string; entries: AgendaTask[] }) {
  const { locale } = useI18n();
  if (entries.length === 0) return null;

  return (
    <section className="mt-8 first:mt-1">
      <div className="flex items-end justify-between border-b border-ink pb-2">
        <h2 className="font-display text-[15px] font-semibold tracking-[-0.01em]">{title}</h2>
        <span className="font-display text-[22px] leading-none font-semibold tabular-nums">
          {entries.length}
        </span>
      </div>
      <ul>
        {entries.map((entry) => {
          const due = dueLabel(entry.due_on, locale);
          const priority = entry.priority !== "normal" ? priorityLabel(entry.priority, locale) : null;
          return (
            <li key={entry.task_id} className="border-b border-rule">
              <Link
                to={`/projects/${entry.project_id}?task=${entry.task_id}`}
                className="grid min-h-11 grid-cols-[minmax(0,1fr)_auto] gap-4 py-3 transition-colors hover:bg-hover"
              >
                <div className="min-w-0">
                  <p className="truncate text-[11px] tracking-[0.01em] text-steel">
                    {entry.project_name}
                  </p>
                  <p className="mt-0.5 text-[15px] font-medium leading-[1.3]">{entry.title}</p>
                </div>
                <div className="flex shrink-0 flex-col items-end justify-center gap-0.5 text-[12px] text-steel">
                  <span className={due.tone === "over" ? "text-signal" : due.tone === "today" ? "text-ink" : ""}>
                    {due.text}
                  </span>
                  {priority && <span className="text-[11px]">{priority}</span>}
                </div>
              </Link>
            </li>
          );
        })}
      </ul>
    </section>
  );
}

/** Read-only daily view: overdue, today and the next seven days. */
export default function TodayPage() {
  const { t, locale } = useI18n();
  const today = todayLocalDate();
  const date = parseDateKey(today) ?? new Date();
  usePageTitle(t("page.today"));

  const agenda = useQuery({
    queryKey: ["agenda", today],
    queryFn: () => api.agenda(today),
    staleTime: 0,
    refetchOnWindowFocus: true,
  });

  const data = agenda.data;
  const total = data ? data.overdue.length + data.today.length + data.week.length : 0;

  return (
    <div className="flex min-h-full flex-col">
      <AppBar
        left={
          <>
            <Link
              to="/"
              aria-label={t("today.toProjects")}
              className="shrink-0 border-b border-transparent pb-px text-[13px] text-steel transition-colors hover:border-rule hover:text-ink"
            >
              ←<span className="hidden sm:inline"> {t("board.back")}</span>
            </Link>
          </>
        }
      />

      <main className="flex-1 px-4 py-7 md:px-12 md:py-10">
        <div className="w-full max-w-[920px]">
          <h2 className="font-display text-[28px] font-semibold tracking-[-0.02em] md:text-[32px]">
            {t("today.title")}
          </h2>
          <p className="mt-1 text-[14px] text-steel">{formatLongDate(date, locale)}</p>
          {data && (
            <p className="mt-3 text-[13px] text-steel">
              {t("today.summary", {
                overdue: data.overdue.length,
                today: data.today.length,
                week: data.week.length,
              })}
            </p>
          )}

          {agenda.isPending && (
            <p className="py-10 text-[14px] text-steel">{t("common.loading")}</p>
          )}

          {agenda.isError && (
            <div className="py-8">
              <p role="alert" className="border-l-2 border-signal pl-3 text-[13px]">
                {t("errors.load")}
              </p>
              <button
                type="button"
                onClick={() => void agenda.refetch()}
                className="mt-3 text-[13px] text-steel hover:text-ink"
              >
                {t("common.retry")}
              </button>
            </div>
          )}

          {data && total === 0 && (
            <div className="mt-10 border-t border-rule py-8 text-[14px] text-steel">
              <p>{t("today.empty")}</p>
              <Link
                to="/"
                className="mt-2 inline-block border-b border-rule pb-px text-[13px] text-steel hover:border-ink hover:text-ink"
              >
                {t("today.toProjects")}
              </Link>
            </div>
          )}

          {data && total > 0 && (
            <div className="mt-6">
              <Section title={t("today.overdue")} entries={data.overdue} />
              <Section title={t("today.today")} entries={data.today} />
              <Section title={t("today.week")} entries={data.week} />
            </div>
          )}
        </div>
      </main>
    </div>
  );
}
