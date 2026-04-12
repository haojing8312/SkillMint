import type { ClawhubInstallCandidate } from "./chatViewHelpers";

interface ChatInstallCandidatesPanelProps {
  candidates: ClawhubInstallCandidate[];
  installError: string | null;
  installedSkillSet: Set<string>;
  installingSlug: string | null;
  renderCandidateText: (text: string) => string;
  onInstallRequest: (candidate: ClawhubInstallCandidate) => void;
}

export function ChatInstallCandidatesPanel({
  candidates,
  installError,
  installedSkillSet,
  installingSlug,
  renderCandidateText,
  onInstallRequest,
}: ChatInstallCandidatesPanelProps) {
  if (candidates.length === 0) return null;

  return (
    <div className="mt-3 border border-blue-100 bg-blue-50/40 rounded-xl p-3">
      <div className="text-xs font-medium text-blue-700 mb-2">可安装技能</div>
      <div className="space-y-2">
        {candidates.map((candidate) => {
          const installed = installedSkillSet.has(`clawhub-${candidate.slug}`);
          const isInstalling = installingSlug === candidate.slug;
          return (
            <div key={`${candidate.slug}:${candidate.githubUrl ?? ""}`} className="rounded-lg border border-blue-100 bg-white p-2.5">
              <div className="flex items-start justify-between gap-3">
                <div className="min-w-0">
                  <div className="text-sm font-medium text-gray-800 truncate">{renderCandidateText(candidate.name)}</div>
                  <div className="text-[11px] text-gray-400">slug: {candidate.slug}</div>
                </div>
                <button
                  onClick={() => {
                    if (installed || isInstalling) return;
                    onInstallRequest(candidate);
                  }}
                  disabled={installed || isInstalling}
                  className={`h-7 px-2.5 rounded text-xs font-medium transition-colors ${
                    installed
                      ? "bg-gray-100 text-gray-400 cursor-not-allowed"
                      : isInstalling
                        ? "bg-blue-100 text-blue-400 cursor-not-allowed"
                        : "bg-blue-500 hover:bg-blue-600 text-white"
                  }`}
                >
                  {installed ? "已安装" : isInstalling ? "安装中..." : "立即安装"}
                </button>
              </div>
              {candidate.description && (
                <div className="mt-1.5 text-xs text-gray-600 line-clamp-2">
                  {renderCandidateText(candidate.description)}
                </div>
              )}
              <div className="mt-1.5 text-[11px] text-gray-400">stars: {candidate.stars ?? 0}</div>
            </div>
          );
        })}
      </div>
      {installError && <div className="mt-2 text-xs text-red-500">{installError}</div>}
    </div>
  );
}
