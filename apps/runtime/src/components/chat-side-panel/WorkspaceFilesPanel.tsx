import { useEffect, useMemo, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { FilePreviewPane, WorkspaceFilePreview } from "./FilePreviewPane";
import {
  ancestorDirectories,
  buildTree,
  joinWorkspacePath,
  type WorkspaceFileItem,
  WorkspaceFilesSidebar,
} from "./workspaceFilesTree";

interface WorkspaceFilesPanelProps {
  workspace: string;
  touchedFiles: string[];
  active: boolean;
}

export function WorkspaceFilesPanel({ workspace, touchedFiles, active }: WorkspaceFilesPanelProps) {
  const [files, setFiles] = useState<WorkspaceFileItem[]>([]);
  const [selectedPath, setSelectedPath] = useState("");
  const [preview, setPreview] = useState<WorkspaceFilePreview | null>(null);
  const [search, setSearch] = useState("");
  const [previewMode, setPreviewMode] = useState<"rendered" | "source">("rendered");
  const [expandedDirs, setExpandedDirs] = useState<Set<string>>(new Set());

  useEffect(() => {
    if (!active || !workspace) return;
    let cancelled = false;
    invoke<WorkspaceFileItem[]>("list_workspace_files", { workspace })
      .then((result) => {
        if (cancelled) return;
        const nextFiles = result || [];
        setFiles(nextFiles);
        setExpandedDirs(new Set(touchedFiles.flatMap(ancestorDirectories)));
        setSelectedPath((current) => current || touchedFiles[0] || nextFiles.find((item) => item.kind !== "directory")?.path || "");
      })
      .catch(() => {
        if (!cancelled) {
          setFiles([]);
        }
      });
    return () => {
      cancelled = true;
    };
  }, [active, workspace, touchedFiles]);

  useEffect(() => {
    if (!selectedPath || !workspace) {
      setPreview(null);
      return;
    }
    let cancelled = false;
    invoke<WorkspaceFilePreview>("read_workspace_file_preview", {
      workspace,
      relativePath: selectedPath,
    })
      .then((result) => {
        if (!cancelled) {
          setPreview(result);
          setPreviewMode(result?.kind === "html" ? "rendered" : result?.kind === "markdown" ? "rendered" : "source");
        }
      })
      .catch(() => {
        if (!cancelled) {
          setPreview(null);
        }
      });
    return () => {
      cancelled = true;
    };
  }, [selectedPath, workspace]);

  const filteredFiles = useMemo(() => {
    const query = search.trim().toLowerCase();
    return files.filter((file) => !query || file.path.toLowerCase().includes(query) || file.name.toLowerCase().includes(query));
  }, [files, search]);

  const tree = useMemo(() => buildTree(filteredFiles), [filteredFiles]);

  useEffect(() => {
    if (!selectedPath) return;
    setExpandedDirs((prev) => {
      const next = new Set(prev);
      ancestorDirectories(selectedPath).forEach((path) => next.add(path));
      return next;
    });
  }, [selectedPath]);

  return (
    <div className="flex h-full min-h-[640px] overflow-hidden rounded-2xl border border-gray-200 bg-white">
      <WorkspaceFilesSidebar
        workspace={workspace}
        search={search}
        onSearchChange={setSearch}
        tree={tree}
        expandedDirs={expandedDirs}
        selectedPath={selectedPath}
        touchedFiles={touchedFiles}
        onToggleDirectory={(path) =>
          setExpandedDirs((prev) => {
            const next = new Set(prev);
            if (next.has(path)) {
              next.delete(path);
            } else {
              next.add(path);
            }
            return next;
          })
        }
        onSelectFile={setSelectedPath}
        onOpenWorkspace={() => {
          if (workspace) {
            void invoke("open_external_url", { url: workspace });
          }
        }}
      />
      <div className="min-w-0 flex-1">
        <FilePreviewPane
          preview={preview}
          workspace={workspace}
          mode={previewMode}
          onModeChange={setPreviewMode}
          onCopyPath={(path) => {
            void globalThis.navigator?.clipboard?.writeText?.(path);
          }}
          onOpenFile={(path) => {
            if (workspace) {
              void invoke("open_external_url", { url: joinWorkspacePath(workspace, path) });
            }
          }}
        />
      </div>
    </div>
  );
}
