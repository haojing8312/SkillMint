import type { TaskJourneyViewModel } from "../chat-side-panel/view-model";

interface TaskJourneySummaryProps {
  model: TaskJourneyViewModel;
  onViewFiles?: () => void;
}

export function TaskJourneySummary({
  model,
  onViewFiles,
}: TaskJourneySummaryProps) {
  if (model.deliverables.length === 0 || !onViewFiles) {
    return null;
  }

  return (
    <div className="mr-auto mt-3 max-w-[80%]">
      <button
        type="button"
        onClick={onViewFiles}
        className="flex w-full items-center gap-3 rounded-3xl border border-gray-200 bg-white px-6 py-6 text-left text-gray-700 shadow-sm transition-colors hover:bg-gray-50"
      >
        <svg className="h-7 w-7 flex-none text-gray-500" fill="none" viewBox="0 0 24 24" stroke="currentColor" strokeWidth={1.8}>
          <path
            strokeLinecap="round"
            strokeLinejoin="round"
            d="M3 7.5A2.25 2.25 0 015.25 5.25H9l2.25 2.25h7.5A2.25 2.25 0 0121 9.75v8.5A2.25 2.25 0 0118.75 20.5H5.25A2.25 2.25 0 013 18.25V7.5z"
          />
        </svg>
        <span className="text-2xl font-medium tracking-tight">查看此任务中的所有文件</span>
      </button>
    </div>
  );
}
