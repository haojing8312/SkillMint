interface FileTreeProps {
  files: string[];
}

export function FileTree({ files }: FileTreeProps) {
  if (files.length === 0) {
    return (
      <div className="text-slate-400 text-sm p-4 italic">
        选择 Skill 目录后显示文件树
      </div>
    );
  }

  return (
    <div className="text-sm font-mono space-y-0.5 p-3">
      {files.map((f) => (
        <div key={f} className="flex items-center gap-2 text-slate-300 py-0.5 hover:text-slate-100">
          <span className="text-slate-500 text-xs">[file]</span>
          <span className="truncate">{f}</span>
        </div>
      ))}
    </div>
  );
}
