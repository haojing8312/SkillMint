type ChatAskUserActionCardProps = {
  askUserQuestion: string;
  askUserOptions: string[];
  askUserAnswer: string;
  onAskUserAnswerChange: (value: string) => void;
  onAnswerUser: (answer: string) => void;
};

export function ChatAskUserActionCard({
  askUserQuestion,
  askUserOptions,
  askUserAnswer,
  onAskUserAnswerChange,
  onAnswerUser,
}: ChatAskUserActionCardProps) {
  return (
    <div className="sticky top-0 z-20 flex justify-start">
      <div
        data-testid="ask-user-action-card"
        className="max-w-[80%] rounded-2xl border border-amber-300 bg-amber-50 px-4 py-3 text-sm shadow-sm"
      >
        <div className="mb-1 font-semibold text-amber-800">需要你的确认</div>
        <div className="mb-2 font-medium text-amber-700">{askUserQuestion}</div>
        {askUserOptions.length > 0 && (
          <div className="mb-2 flex flex-wrap gap-2">
            {askUserOptions.map((option, index) => (
              <button
                key={index}
                type="button"
                onClick={() => onAnswerUser(option)}
                className="rounded border border-amber-300 bg-amber-100 px-3 py-1 text-xs text-amber-800 transition-colors hover:bg-amber-200"
              >
                {option}
              </button>
            ))}
          </div>
        )}
        <div className="flex gap-2">
          <input
            value={askUserAnswer}
            onChange={(event) => onAskUserAnswerChange(event.target.value)}
            onKeyDown={(event) => {
              if (event.key === "Enter") {
                event.preventDefault();
                onAnswerUser(askUserAnswer);
              }
            }}
            placeholder="输入回答..."
            className="flex-1 rounded border border-gray-200 bg-white px-2 py-1 text-xs focus:border-amber-500 focus:outline-none"
          />
          <button
            type="button"
            onClick={() => onAnswerUser(askUserAnswer)}
            disabled={!askUserAnswer.trim()}
            className="rounded bg-amber-500 px-3 py-1 text-xs transition-colors hover:bg-amber-600 disabled:bg-gray-200 disabled:text-gray-400"
          >
            回答
          </button>
        </div>
      </div>
    </div>
  );
}
