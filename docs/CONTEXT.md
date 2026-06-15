# OCorganize

Desktop file organizer that sorts loose, Finder-visible files sitting directly in the user's chosen folder into category subfolders at that same root.

## Language

**Organization root**:
The directory the user selects to organize.
_Avoid_: target folder, base path

**LooseRootScan**:
Shallow scan of only direct, Finder-visible files in the organization root. Subfolders, hidden files, and existing category folders are skipped.
_Avoid_: recursive scan, deep walk

**Category folder**:
A subfolder created under the organization root for a grouping bucket, such as `PDF_FILES` or `no_extension`.
_Avoid_: bucket, group directory

**Organization strategy**:
The rule used to group files: extension, size, or date.
_Avoid_: sort mode, method

**Undo log**:
A JSON file written in the organization root listing file moves that can be reversed.
_Avoid_: history file, rollback manifest

**Skipped symlink**:
A symlinked file or directory excluded from scanning and moves.
_Avoid_: ignored link

## Relationships

- An **Organization root** receives many **Category folders** during LooseRootScan
- Each **Undo log** belongs to one **Organization root**
- **Organization strategy** determines how files are assigned to **Category folders**

## Example dialogue

> **Dev:** "Do we organize files inside subfolders?"
> **Domain expert:** "No — LooseRootScan only touches loose files already sitting in the organization root."

## Flagged ambiguities

- "Preview" can mean dry-run organize or the pre-run analysis dialog — both are preview flows, but only the checkbox dry-run skips moves.
