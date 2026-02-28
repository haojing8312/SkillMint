interface FileTreeProps {
  files: string[];
}

export function FileTree({ files }: FileTreeProps) {
  if (files.length === 0) {
    return <div className="text-gray-400 text-sm p-4 italic">选择目录后显示文件列表</div>;
  }

  return (
    <div className="text-sm font-mono space-y-0.5 p-3">
      {files.map((f) => (
        <div key={f} className="flex items-center gap-2 text-gray-700 py-0.5 hover:text-gray-900">
          <span className="text-gray-400 text-xs">[file]</span>
          <span className="truncate">{f}</span>
        </div>
      ))}
    </div>
  );
}
