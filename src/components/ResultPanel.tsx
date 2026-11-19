import type { CSSProperties } from "react";

import type { EntryStatus, Report, TaskPhase } from "../api";

interface Props {
  report: Report | null;
  stale: boolean;
  phase: TaskPhase;
  restoreTarget: string;
  onRestore: (directory: string) => void;
}

const OPERATION_COPY: Record<Report["operation"], { title: string; note: string }> = {
  preview: { title: "预览结果", note: "尚未修改任何文件" },
  replace: { title: "写入完成", note: "已生成 .bak 备份，可随时回滚" },
  restore: { title: "备份已恢复", note: ".bak 备份已删除" },
};

export default function ResultPanel({ report, stale, phase, restoreTarget, onRestore }: Props) {
  const copy = report ? OPERATION_COPY[report.operation] : null;

  return (
    <>
      <div className="results-head">
        <div>
          <p className="panel-kicker">OUTPUT LEDGER</p>
          <h2>{copy ? copy.title : "结果"}</h2>
          <p className="results-sub">
            {copy ? copy.note : "预览后在这里查看每个文件的命中情况。"}
          </p>
        </div>
        <button
          type="button"
          className="btn btn-link"
          disabled={restoreTarget.trim() === "" || phase !== null}
          onClick={() => onRestore(restoreTarget)}
        >
          恢复 .bak 备份
        </button>
      </div>

      {stale && <p className="stale-note">输入已修改，当前结果已过期，请重新预览。</p>}

      {report ? <ReportBody report={report} /> : <EmptyState />}
    </>
  );
}

function ReportBody({ report }: { report: Report }) {
  const isRestore = report.operation === "restore";

  return (
    <div className="report">
      <div className="stats">
        <Stat label={isRestore ? "扫描备份" : "扫描文件"} value={report.scanned} />
        <Stat
          label={isRestore ? "已恢复" : "命中文件"}
          value={report.changedFiles}
          tone="accent"
        />
        {!isRestore && <Stat label="替换处数" value={report.replacements} />}
        <Stat
          label="失败"
          value={report.failures}
          tone={report.failures > 0 ? "danger" : undefined}
        />
      </div>

      {report.entries.length === 0 ? (
        <div className="empty empty--compact">
          <p className="empty-title">
            {isRestore ? "没有找到 .bak 备份" : "没有命中任何文件"}
          </p>
          <p className="empty-hint">
            {isRestore
              ? "执行一次替换后才会生成备份。"
              : "换个关键词，或确认目录是否选对。"}
          </p>
        </div>
      ) : (
        <ul className="ledger">
          {report.entries.map((entry, index) => (
            <li
              key={`${entry.path}-${index}`}
              className={`ledger-row status-${entry.status}`}
              style={{ "--i": index } as CSSProperties}
            >
              <span className="row-index">{String(index + 1).padStart(2, "0")}</span>
              <div className="row-main">
                <span className="row-path">{entry.relativePath}</span>
                {entry.error && <span className="row-error">{entry.error}</span>}
              </div>
              <span className="row-metric">{metricLabel(entry.matches, entry.status)}</span>
            </li>
          ))}
        </ul>
      )}
    </div>
  );
}

function EmptyState() {
  return (
    <div className="empty">
      <span className="empty-mark" aria-hidden="true">
        Aa
      </span>
      <p className="empty-title">还没有运行任务</p>
      <ol className="empty-steps">
        <li>选择目标文件夹</li>
        <li>填写查找与替换内容</li>
        <li>先预览，再写入</li>
      </ol>
    </div>
  );
}

function Stat({
  label,
  value,
  tone,
}: {
  label: string;
  value: number;
  tone?: "accent" | "danger";
}) {
  return (
    <div className={`stat${tone ? ` stat--${tone}` : ""}`}>
      <span className="stat-value">{value}</span>
      <span className="stat-label">{label}</span>
    </div>
  );
}

function metricLabel(matches: number, status: EntryStatus): string {
  if (status === "failed") return "失败";
  if (status === "restored") return "已恢复";
  return `${matches} 处`;
}
