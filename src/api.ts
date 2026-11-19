import { invoke } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-dialog";

export type Operation = "preview" | "replace" | "restore";
export type EntryStatus = "previewed" | "written" | "restored" | "failed";
export type FileKind = "txt" | "docx" | "xlsx";
export type TaskPhase = "preview" | "apply" | "restore" | null;

export interface ReportEntry {
  path: string;
  relativePath: string;
  matches: number;
  status: EntryStatus;
  error: string | null;
}

export interface Report {
  operation: Operation;
  root: string;
  scanned: number;
  changedFiles: number;
  replacements: number;
  failures: number;
  entries: ReportEntry[];
}

export async function pickDirectory(): Promise<string | null> {
  const selected = await open({
    directory: true,
    multiple: false,
    title: "选择要处理的文件夹",
  });

  return typeof selected === "string" ? selected : null;
}

export function previewReplace(
  directory: string,
  needle: string,
  replacement: string,
  kinds: FileKind[],
): Promise<Report> {
  return invoke<Report>("preview_replace", { directory, needle, replacement, kinds });
}

export function applyReplace(
  directory: string,
  needle: string,
  replacement: string,
  kinds: FileKind[],
): Promise<Report> {
  return invoke<Report>("apply_replace", { directory, needle, replacement, kinds });
}

export function restoreBackups(directory: string): Promise<Report> {
  return invoke<Report>("restore_backups", { directory });
}

export function describeError(error: unknown): string {
  if (typeof error === "string") return error;
  if (error instanceof Error) return error.message;
  return String(error);
}
