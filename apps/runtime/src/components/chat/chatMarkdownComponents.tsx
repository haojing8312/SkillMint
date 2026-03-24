import { Prism as SyntaxHighlighter } from "react-syntax-highlighter";
import { oneDark } from "react-syntax-highlighter/dist/esm/styles/prism";

function extractNodeText(node: unknown): string {
  if (typeof node === "string" || typeof node === "number") return String(node);
  if (Array.isArray(node)) return node.map((item) => extractNodeText(item)).join("");
  if (node && typeof node === "object" && "props" in node) {
    return extractNodeText((node as { props?: { children?: unknown } }).props?.children);
  }
  return "";
}

export function createChatMarkdownComponents() {
  return {
    code({ className, children, ...props }: any) {
      const match = /language-(\w+)/.exec(className || "");
      const codeString = String(children).replace(/\n$/, "");
      return match ? (
        <SyntaxHighlighter
          style={oneDark}
          language={match[1]}
          PreTag="div"
          customStyle={{ margin: 0, borderRadius: "0.375rem", fontSize: "0.8125rem" }}
        >
          {codeString}
        </SyntaxHighlighter>
      ) : (
        <code className={"bg-gray-200/60 px-1.5 py-0.5 rounded text-sm text-gray-800 font-mono " + (className || "")} {...props}>
          {children}
        </code>
      );
    },
    h1: ({ children }: any) => (
      <h1 className="mt-7 mb-4 border-b border-slate-200 pb-3 text-[1.75rem] font-semibold tracking-[-0.02em] text-slate-950">
        {children}
      </h1>
    ),
    h2: ({ children }: any) => (
      <h2 className="mt-6 mb-3 border-b border-slate-150 pb-2 text-[1.35rem] font-semibold tracking-[-0.015em] text-slate-900">
        {children}
      </h2>
    ),
    h3: ({ children }: any) => <h3 className="mt-5 mb-2.5 text-[1.1rem] font-semibold text-slate-900">{children}</h3>,
    h4: ({ children }: any) => <h4 className="mt-4 mb-2 text-base font-semibold text-slate-800">{children}</h4>,
    h5: ({ children }: any) => <h5 className="mt-3 mb-1.5 text-sm font-semibold uppercase tracking-[0.01em] text-slate-700">{children}</h5>,
    h6: ({ children }: any) => <h6 className="mt-3 mb-1 text-sm font-medium text-slate-600">{children}</h6>,
    p: ({ children }: any) => {
      const text = extractNodeText(children).trim();
      const isSummaryBlock = /^(共计|总计|总结[:：]?|结论[:：]?)/.test(text);
      return isSummaryBlock ? (
        <p
          data-testid="assistant-result-summary"
          className="mb-4 rounded-2xl border border-slate-200/90 bg-slate-50/80 px-4 py-3 text-[15px] font-medium leading-7 text-slate-800"
        >
          {children}
        </p>
      ) : (
        <p className="mb-4 text-[15px] leading-7 text-slate-700">{children}</p>
      );
    },
    ul: ({ children }: any) => <ul className="mb-4 list-disc space-y-1.5 pl-5 text-[15px] text-slate-700">{children}</ul>,
    ol: ({ children }: any) => <ol className="mb-4 list-decimal space-y-1.5 pl-5 text-[15px] text-slate-700">{children}</ol>,
    li: ({ children }: any) => <li className="leading-7 text-slate-700">{children}</li>,
    a: ({ href, children }: any) => (
      <a
        href={href}
        className="text-blue-500 hover:text-blue-600 underline underline-offset-2 text-sm"
        target="_blank"
        rel="noopener noreferrer"
      >
        {children}
      </a>
    ),
    blockquote: ({ children }: any) => (
      <blockquote className="my-4 rounded-r-lg border-l-[3px] border-slate-300 bg-slate-50/70 py-1 pl-4">
        <div className="text-[15px] italic text-slate-600">{children}</div>
      </blockquote>
    ),
    table: ({ children }: any) => (
      <div className="my-4 overflow-x-auto rounded-2xl border border-slate-200/90 bg-white/90 shadow-[0_1px_2px_rgba(15,23,42,0.04)]">
        <table className="min-w-full text-sm">{children}</table>
      </div>
    ),
    thead: ({ children }: any) => <thead className="bg-slate-50/90">{children}</thead>,
    tbody: ({ children }: any) => <tbody className="divide-y divide-slate-200/80 bg-white">{children}</tbody>,
    tr: ({ children }: any) => <tr className="transition-colors hover:bg-slate-50/70">{children}</tr>,
    th: ({ children }: any) => (
      <th className="bg-slate-50/90 px-4 py-3 text-left text-xs font-semibold uppercase tracking-[0.08em] text-slate-500">
        {children}
      </th>
    ),
    td: ({ children }: any) => {
      const text = extractNodeText(children).trim();
      const isNumericLike = /^(?:[\d,]+(?:\.\d+)?(?:\s*(?:字节|KB|MB|GB|%))?|[\d]{4}\/[\d]{1,2}\/[\d]{1,2}.*)$/.test(text);
      return (
        <td className={"px-4 py-3 text-[15px] text-slate-700 " + (isNumericLike ? "whitespace-nowrap tabular-nums" : "")}>
          {children}
        </td>
      );
    },
    hr: () => <hr className="my-7 border-slate-200" />,
    strong: ({ children }: any) => <strong className="font-semibold text-slate-950">{children}</strong>,
    em: ({ children }: any) => <em className="italic text-slate-700">{children}</em>,
    input: ({ type, checked, disabled }: any) => {
      if (type === "checkbox" && disabled) {
        return (
          <span
            aria-hidden="true"
            className={
              "mr-2 inline-flex h-4 w-4 translate-y-[1px] items-center justify-center rounded border " +
              (checked
                ? "border-emerald-200 bg-emerald-50 text-emerald-600"
                : "border-slate-300 bg-white text-transparent")
            }
          >
            {checked ? (
              <svg className="h-3 w-3" fill="none" viewBox="0 0 24 24" stroke="currentColor" strokeWidth={2}>
                <path strokeLinecap="round" strokeLinejoin="round" d="M5 13l4 4L19 7" />
              </svg>
            ) : (
              <span className="h-3 w-3" />
            )}
          </span>
        );
      }
      return <input type={type} checked={checked} disabled={disabled} readOnly />;
    },
  };
}
