import { useEffect, useRef } from "react";

export type ConfirmState =
  | { kind: "apply"; directory: string; files: number; matches: number }
  | { kind: "restore"; directory: string };

interface Props {
  state: ConfirmState;
  onCancel: () => void;
  onConfirm: () => void;
}

export default function ConfirmDialog({ state, onCancel, onConfirm }: Props) {
  const cancelRef = useRef<HTMLButtonElement>(null);
  const isApply = state.kind === "apply";

  useEffect(() => {
    cancelRef.current?.focus();

    function handleKeyDown(event: KeyboardEvent) {
      if (event.key === "Escape") onCancel();
    }

    window.addEventListener("keydown", handleKeyDown);
    return () => window.removeEventListener("keydown", handleKeyDown);
  }, [onCancel]);

  return (
    <div
      className="dialog-backdrop"
      onMouseDown={(event) => {
        if (event.target === event.currentTarget) onCancel();
      }}
    >
      <div
        className="dialog"
        role="dialog"
        aria-modal="true"
        aria-labelledby="dialog-title"
        aria-describedby="dialog-note"
      >
        <p className="dialog-kicker">{isApply ? "WRITE TO DISK" : "RESTORE FROM BACKUP"}</p>
        <h3 id="dialog-title">{isApply ? "确认写入这些文件？" : "确认从备份恢复？"}</h3>

        <dl className="dialog-facts">
          <div>
            <dt>目录</dt>
            <dd className="mono">{state.directory}</dd>
          </div>
          {isApply && (
            <div>
              <dt>范围</dt>
              <dd>
                {state.files} 个文件 · {state.matches} 处命中
              </dd>
            </div>
          )}
        </dl>

        <p className="dialog-note" id="dialog-note">
          {isApply
            ? "写入前会为每个被修改的文件生成 .bak 备份，之后可通过「恢复备份」回滚。"
            : "会用 .bak 覆盖当前文件并删除备份，此操作无法撤销。"}
        </p>

        <div className="dialog-actions">
          <button ref={cancelRef} type="button" className="btn btn-ghost" onClick={onCancel}>
            取消
          </button>
          <button type="button" className="btn btn-accent" onClick={onConfirm}>
            {isApply ? "确认写入" : "确认恢复"}
          </button>
        </div>
      </div>
    </div>
  );
}
