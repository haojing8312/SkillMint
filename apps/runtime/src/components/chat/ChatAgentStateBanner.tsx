import type { ReactNode } from "react";

interface ChatAgentStateBannerProps {
  visible: boolean;
  state?: string | null;
  label: string;
  indicator: ReactNode;
  secondary: ReactNode;
}

export function ChatAgentStateBanner({
  visible,
  state,
  label,
  indicator,
  secondary,
}: ChatAgentStateBannerProps) {
  if (!visible) {
    return null;
  }

  const toneClass =
    state === "stopped"
      ? "text-amber-800 border-amber-200"
      : state === "error"
      ? "text-red-700 border-red-200"
      : state === "retrying"
      ? "text-blue-700 border-blue-200"
      : "text-gray-600 border-gray-200";
  const labelClass =
    state === "error" ? "text-red-500" : state === "retrying" ? "text-blue-700" : undefined;

  return (
    <div
      className={`sticky top-0 z-10 flex items-center gap-2 rounded-xl border bg-white/80 px-4 py-2 text-xs shadow-sm backdrop-blur-lg mx-4 mt-2 ${toneClass}`}
    >
      {indicator}
      <div className="flex min-w-0 flex-col">
        <span className={labelClass}>{label}</span>
        {secondary}
      </div>
    </div>
  );
}
