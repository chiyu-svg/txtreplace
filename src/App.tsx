import { useEffect, useMemo, useState } from "react";
import type { FormEvent } from "react";

import {
  applyReplace,
  describeError,
  pickDirectory,
  previewReplace,
  restoreBackups,
} from "./api";
import type { Report, TaskPhase } from "./api";
import ConfirmDialog from "./components/ConfirmDialog";
import type { ConfirmState } from "./components/ConfirmDialog";
import ResultPanel from "./components/ResultPanel";

export default function App() {
  const [directory, setDirectory] = useState("");
  const [needle, setNeedle] = useState("");
  const [replacement, setReplacement] = useState("");
  const [report, setReport] = useState<Report | null>(null);
  const [previewKey, setPreviewKey] = useState<string | null>(null);
  const [phase, setPhase] = useState<TaskPhase>(null);
  const [error, setError] = useState<string | null>(null);
  const [confirm, setConfirm] = useState<ConfirmState | null>(null);

  const currentKey = useMemo(
    () => `${directory}\u0000${needle}\u0000${replacement}`,
    [directory, needle, replacement],
  );

  const stale = report?.operation === "preview" && previewKey !== currentKey;
  const canPreview = directory.trim() !== "" && needle.length > 0 && phase === null;
  const canApply =
    report !== null &&
    report.operation === "preview" &&
    !stale &&
    report.changedFiles > 0 &&
    phase === null;

  useEffect(() => {
    if (!error) return;
    const timer = window.setTimeout(() => setError(null), 9000);
    return () => window.clearTimeout(timer);
  }, [error]);

  async function handlePick() {
    try {
      const selected = await pickDirectory();
      if (!selected) return;
      setDirectory(selected);
      setReport(null);
      setPreviewKey(null);
      setError(null);
    } catch (caught) {
      setError(describeError(caught));
    }
  }

  async function handlePreview(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    if (!canPreview) return;

    setPhase("preview");
    setError(null);
    try {
      const next = await previewReplace(directory, needle, replacement);
      setReport(next);
      setPreviewKey(currentKey);
    } catch (caught) {
      setError(describeError(caught));
    } finally {
      setPhase(null);
    }
  }

  function requestApply() {
    if (!canApply || !report) return;
    setConfirm({
      kind: "apply",
      directory,
      files: report.changedFiles,
      matches: report.replacements,
    });
  }

  function requestRestore(target: string) {
    if (!target.trim() || phase !== null) return;
    setConfirm({ kind: "restore", directory: target });
  }

  async function handleConfirm() {
    if (!confirm) return;
    const action = confirm;
    setConfirm(null);
    setPhase(action.kind === "apply" ? "apply" : "restore");
    setError(null);

    try {
      const next =
        action.kind === "apply"
          ? await applyReplace(action.directory, needle, replacement)
          : await restoreBackups(action.directory);
      setReport(next);
      setPreviewKey(null);
    } catch (caught) {
      setError(describeError(caught));
    } finally {
      setPhase(null);
    }
  }

  return (
    <div className="app">
      <div className="grain" aria-hidden="true" />

      <header className="masthead">
        <div className="brand">
          <span className="brand-mark" aria-hidden="true">
            <SwapGlyph />
          </span>
          <div>
            <h1>文本替换</h1>
            <p className="eyebrow">TXT · DOCX · XLSX — DESKTOP EDITION</p>
          </div>
        </div>
      </header>

      <main className="workbench">
        <section className="panel panel-form" aria-labelledby="form-title">
          <div className="panel-head">
            <p className="panel-kicker">TASK SETUP</p>
            <h2 id="form-title">设置替换任务</h2>
            <p className="panel-sub">先预览命中结果，确认无误后再写入；写入会自动备份。</p>
          </div>

          <form onSubmit={handlePreview}>
            <div className="field">
              <div className="field-label">
                <span className="field-index">01</span>
                <span>目标目录</span>
                <span className="field-hint">递归扫描</span>
              </div>
              <div className="dir-row">
                <input
                  type="text"
                  value={directory}
                  readOnly
                  placeholder="尚未选择文件夹"
                  aria-label="目标目录"
                  onClick={handlePick}
                />
                <button
                  type="button"
                  className="btn btn-ghost"
                  onClick={handlePick}
                  disabled={phase !== null}
                >
                  浏览…
                </button>
              </div>
            </div>

            <div className="field">
              <div className="field-label">
                <span className="field-index">02</span>
                <span>查找内容</span>
              </div>
              <input
                type="text"
                value={needle}
                onChange={(event) => setNeedle(event.target.value)}
                placeholder="例如：Java"
                autoFocus
              />
            </div>

            <div className="field">
              <div className="field-label">
                <span className="field-index">03</span>
                <span>替换为</span>
                <span className="field-hint">留空 = 删除</span>
              </div>
              <input
                type="text"
                value={replacement}
                onChange={(event) => setReplacement(event.target.value)}
                placeholder="例如：Rust"
              />
            </div>

            <div className="form-actions">
              <button type="submit" className="btn btn-primary" disabled={!canPreview}>
                {phase === "preview" && <Spinner />}
                预览命中
              </button>
              <button
                type="button"
                className="btn btn-accent"
                disabled={!canApply}
                onClick={requestApply}
              >
                {phase === "apply" && <Spinner />}
                执行替换{canApply && report ? ` · ${report.changedFiles} 个文件` : ""}
              </button>
            </div>

            <p className="form-note">
              {report?.operation === "replace"
                ? "写入完成并已生成 .bak 备份；如需还原，点击右侧「恢复 .bak 备份」。"
                : canApply && report
                  ? `即将写入 ${report.changedFiles} 个文件、共 ${report.replacements} 处，写入前会自动备份。`
                  : "预览不会修改任何文件；修改输入后需重新预览才能写入。"}
            </p>
          </form>
        </section>

        <section className="panel panel-results" aria-live="polite">
          <ResultPanel
            report={report}
            stale={stale}
            phase={phase}
            restoreTarget={report?.root ?? directory}
            onRestore={requestRestore}
          />
        </section>
      </main>

      <footer className="app-footer">
        <span className="footer-mono">
          {report ? `${report.entries.length} 条记录` : "就绪"}
        </span>
      </footer>

      {confirm && (
        <ConfirmDialog
          state={confirm}
          onCancel={() => setConfirm(null)}
          onConfirm={handleConfirm}
        />
      )}

      {error && (
        <div className="toast" role="alert">
          <span>{error}</span>
          <button
            type="button"
            className="toast-close"
            aria-label="关闭"
            onClick={() => setError(null)}
          >
            ×
          </button>
        </div>
      )}
    </div>
  );
}

function SwapGlyph() {
  return (
    <svg viewBox="0 0 24 24" width="24" height="24" fill="none" aria-hidden="true">
      <path
        d="M4 8h13l-3.2-3.2M20 16H7l3.2 3.2"
        stroke="currentColor"
        strokeWidth="1.7"
        strokeLinecap="round"
        strokeLinejoin="round"
      />
    </svg>
  );
}

function Spinner() {
  return <span className="spinner" aria-hidden="true" />;
}
