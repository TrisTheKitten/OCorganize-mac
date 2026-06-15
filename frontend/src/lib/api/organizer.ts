import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";

export type Strategy = "type" | "size" | "date";

export interface PreviewCategory {
  category: string;
  folderName: string;
  fileCount: number;
  sampleFiles: string[];
}

export interface PreviewResult {
  directory: string;
  strategy: string;
  totalFiles: number;
  categoryCount: number;
  symlinkSkipCount: number;
  categories: PreviewCategory[];
}

export interface OrganizeSummary {
  organizedCount: number;
  categoryCount: number;
  symlinkSkipCount: number;
  previewOnly: boolean;
  undoFile: string | null;
}

export interface UndoSummary {
  restoredCount: number;
  failedCount: number;
  partial: boolean;
}

export interface OrganizeProgressEvent {
  status: string;
  progress: number;
  fileName: string;
  processedFiles: number;
  totalFiles: number;
}

export function validateDirectory(path: string): Promise<string> {
  return invoke<string>("validate_directory", { path });
}

export function previewOrganization(
  path: string,
  strategy: Strategy,
): Promise<PreviewResult> {
  return invoke<PreviewResult>("preview_organization", { path, strategy });
}

export function organizeFiles(
  path: string,
  strategy: Strategy,
  previewOnly: boolean,
  createUndo: boolean,
): Promise<OrganizeSummary> {
  return invoke<OrganizeSummary>("organize_files", {
    path,
    strategy,
    previewOnly,
    createUndo,
  });
}

export function undoLastOperation(): Promise<UndoSummary> {
  return invoke<UndoSummary>("undo_last_operation");
}

export function hasUndoOperations(): Promise<boolean> {
  return invoke<boolean>("has_undo_operations");
}

export function onOrganizeProgress(
  handler: (event: OrganizeProgressEvent) => void,
): Promise<UnlistenFn> {
  return listen<OrganizeProgressEvent>("organize-progress", (payload) => {
    handler(payload.payload);
  });
}
