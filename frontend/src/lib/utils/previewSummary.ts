import type { PreviewResult } from "../api/organizer";

const KNOWN_FILE_TYPES = new Set([
  "Image",
  "Video",
  "Audio",
  "Document",
  "Spreadsheet",
  "Presentation",
  "Archive",
  "Code",
  "Font",
  "Disk Image",
  "Application",
]);

export function previewSummaryLines(preview: PreviewResult): string[] {
  const lines: string[] = [];
  const maxGroups = 3;
  for (const [index, category] of preview.categories.entries()) {
    if (index >= maxGroups) {
      break;
    }
    const count = category.fileCount;
    if (preview.strategy === "type") {
      if (category.category === "no_extension") {
        lines.push(`${count} ${count === 1 ? "file" : "files"} without extension`);
      } else if (KNOWN_FILE_TYPES.has(category.category)) {
        lines.push(`${count} ${category.category} ${count === 1 ? "file" : "files"}`);
      } else {
        lines.push(`${count} .${category.category} ${count === 1 ? "file" : "files"}`);
      }
    } else if (preview.strategy === "size") {
      lines.push(`${count} ${count === 1 ? "file" : "files"} in ${category.category}`);
    } else {
      lines.push(`${count} ${count === 1 ? "file" : "files"} from ${category.category}`);
    }
  }
  const remaining = preview.categories.length - lines.length;
  if (remaining > 0) {
    lines.push(`+ ${remaining} more groups`);
  }
  return lines;
}
