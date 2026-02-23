import { useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { save } from "@tauri-apps/plugin-dialog";

interface FrontMatter {
  name?: string;
  description?: string;
  version?: string;
  model?: string;
}

interface PackFormProps {
  dirPath: string;
  frontMatter: FrontMatter;
  fileCount: number;
}

export function PackForm({ dirPath, frontMatter, fileCount }: PackFormProps) {
  const [name, setName] = useState(frontMatter.name ?? "");
  const [description, setDescription] = useState(frontMatter.description ?? "");
  const [version, setVersion] = useState(frontMatter.version ?? "1.0.0");
  const [author, setAuthor] = useState("");
  const [username, setUsername] = useState("");
  const [recommendedModel, setRecommendedModel] = useState(
    frontMatter.model ?? "claude-3-5-sonnet-20241022"
  );
  const [status, setStatus] = useState<"idle" | "packing" | "done" | "error">("idle");
  const [errorMsg, setErrorMsg] = useState("");

  async function handlePack() {
    if (!username.trim()) {
      setErrorMsg("请填写客户用户名");
      setStatus("error");
      return;
    }
    if (!name.trim()) {
      setErrorMsg("请填写 Skill 名称");
      setStatus("error");
      return;
    }
    if (!/^\d+\.\d+\.\d+/.test(version.trim())) {
      setErrorMsg("版本号格式不正确，请使用 semver 格式（如 1.0.0）");
      setStatus("error");
      return;
    }
    const outputPath = await save({
      defaultPath: `${name.trim().replace(/\s+/g, "-")}.skillpack`,
      filters: [{ name: "SkillPack", extensions: ["skillpack"] }],
    });
    if (!outputPath) return;

    setStatus("packing");
    setErrorMsg("");
    try {
      await invoke("pack_skill", {
        dirPath,
        name,
        description,
        version,
        author,
        username,
        recommendedModel,
        outputPath,
      });
      setStatus("done");
    } catch (e: unknown) {
      setStatus("error");
      setErrorMsg(String(e));
    }
  }

  const inputCls =
    "w-full bg-slate-800 border border-slate-600 rounded-md px-3 py-2 text-sm text-slate-100 focus:outline-none focus:border-blue-500 focus:ring-1 focus:ring-blue-500/30 transition-colors";
  const labelCls = "block text-xs font-medium text-slate-400 mb-1.5";

  return (
    <div className="space-y-4">
      <div>
        <label className={labelCls}>Skill 名称 *</label>
        <input
          className={inputCls}
          value={name}
          onChange={(e) => setName(e.target.value)}
          placeholder="例如：合同审查助手"
        />
      </div>
      <div>
        <label className={labelCls}>描述</label>
        <input
          className={inputCls}
          value={description}
          onChange={(e) => setDescription(e.target.value)}
          placeholder="一句话描述此 Skill 的用途"
        />
      </div>
      <div>
        <label className={labelCls}>版本号</label>
        <input
          className={inputCls}
          value={version}
          onChange={(e) => setVersion(e.target.value)}
        />
      </div>
      <div>
        <label className={labelCls}>作者</label>
        <input
          className={inputCls}
          value={author}
          onChange={(e) => setAuthor(e.target.value)}
          placeholder="你的名字或组织"
        />
      </div>
      <div>
        <label className={labelCls}>推荐模型</label>
        <input
          className={inputCls}
          value={recommendedModel}
          onChange={(e) => setRecommendedModel(e.target.value)}
        />
      </div>
      <div>
        <label className={labelCls}>客户用户名（解密密鑰）*</label>
        <input
          className={inputCls}
          value={username}
          onChange={(e) => setUsername(e.target.value)}
          placeholder="例如：alice"
        />
        <p className="text-xs text-slate-500 mt-1.5 leading-relaxed">
          客户需在 Runtime 中输入此用户名才能解锁 Skill。请妥善保管，勿公开。
        </p>
      </div>

      {status === "error" && errorMsg && (
        <div className="text-red-400 text-sm bg-red-950/50 border border-red-800/50 rounded-md p-3">
          {errorMsg}
        </div>
      )}
      {status === "done" && (
        <div className="text-green-400 text-sm bg-green-950/50 border border-green-800/50 rounded-md p-3 space-y-1">
          <div className="font-medium">打包成功！</div>
          <div className="text-xs text-green-300/80 space-y-0.5">
            <div>Skill 名称：{name}</div>
            <div>版本：{version}</div>
            <div>文件数：{fileCount}</div>
          </div>
        </div>
      )}

      <button
        onClick={handlePack}
        disabled={status === "packing"}
        className="w-full bg-blue-600 hover:bg-blue-500 disabled:bg-slate-700 disabled:text-slate-400 text-white font-medium py-2.5 rounded-md transition-colors text-sm"
      >
        {status === "packing" ? "打包中..." : "一键打包"}
      </button>
    </div>
  );
}
