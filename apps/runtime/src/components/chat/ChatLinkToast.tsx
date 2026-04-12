interface ChatLinkToastState {
  variant: "success" | "error";
  message: string;
  url: string;
}

interface ChatLinkToastProps {
  toast: ChatLinkToastState | null;
  onRetry: (url: string) => void;
  onCopy: (url: string) => void;
  onClose: () => void;
}

export function ChatLinkToast({ toast, onRetry, onCopy, onClose }: ChatLinkToastProps) {
  if (!toast) {
    return null;
  }

  return (
    <div className="pointer-events-none absolute inset-x-0 bottom-5 z-20 flex justify-center px-4">
      <div
        data-testid="chat-link-toast"
        className={
          "pointer-events-auto flex max-w-[36rem] items-center gap-3 rounded-2xl border px-4 py-3 text-sm shadow-lg backdrop-blur-sm " +
          (toast.variant === "success"
            ? "border-emerald-200 bg-white/95 text-emerald-700"
            : "border-rose-200 bg-white/95 text-rose-700")
        }
      >
        <span className="font-medium">{toast.message}</span>
        {toast.variant === "error" && (
          <>
            <button
              type="button"
              className="rounded-lg border border-rose-200 bg-rose-50 px-3 py-1.5 text-xs font-medium text-rose-700 transition-colors hover:bg-rose-100"
              onClick={() => onRetry(toast.url)}
            >
              重试
            </button>
            <button
              type="button"
              className="rounded-lg border border-slate-200 bg-slate-50 px-3 py-1.5 text-xs font-medium text-slate-700 transition-colors hover:bg-slate-100"
              onClick={() => onCopy(toast.url)}
            >
              复制链接
            </button>
          </>
        )}
        <button
          type="button"
          aria-label="关闭链接提示"
          className="rounded-lg px-2 py-1 text-xs text-slate-500 transition-colors hover:bg-slate-100 hover:text-slate-700"
          onClick={onClose}
        >
          关闭
        </button>
      </div>
    </div>
  );
}
