# Rust + Iced Local Markdown Reader/Writer — Product & Technical Specification

## 1. Purpose

Build a small cross-platform desktop application in **Rust** using **Iced** that acts as a **local Markdown reader and writer**.

The app must allow a user to:

- Open either a **single Markdown file** or a **root folder**.
- Browse Markdown content in a **sidebar tree**.
- Treat subfolders as **categories**.
- Expand/collapse categories in the sidebar.
- Toggle the main content area between:
  - **Viewer mode** for rendered Markdown
  - **Editor mode** for direct text editing
- Edit the currently selected Markdown file directly in the app.
- **Auto-save on every change**.
- Detect when files are modified externally by another program and update the UI accordingly.
- Ignore any folder named **`asset`** and everything inside it.
- Create a **new Markdown file** or **new folder/category** inside a selected category.

This document is written so that an implementation agent can start building the app end-to-end without needing additional product clarification.

---

## 2. Core Product Goals

### Primary goal
Provide a lightweight local documentation editor/browser with a tree-based navigation model that maps directly to a folder structure.

### Secondary goals
- Be simple and fast.
- Work fully offline.
- Preserve filesystem structure instead of introducing a database.
- Make editing frictionless by auto-saving continuously.
- Reflect external filesystem changes in near real time.

### Non-goals
- No multi-pane live preview.
- No remote sync.
- No Git integration in the first version.
- No WYSIWYG editor.
- No image/file asset manager in the first version.
- No support for editing non-Markdown text files in v1.

---

## 3. Target Platform

The app should be designed for:

- **Windows**
- **macOS**
- **Linux**

The implementation should prefer crates and system integrations that are known to be cross-platform.

---

## 4. Recommended Stack

## UI
- **iced**

## File/folder picker
- **rfd**

## Filesystem watching
- **notify**

## Directory traversal
- **walkdir**

## Error handling
- **anyhow**
- optionally **thiserror** for domain errors

## Path utilities / serialization
- standard library is sufficient for most needs
- `serde` only if state persistence is added

---

## 5. High-Level UX

### 5.1 Open behavior
At startup, the app should show a minimal shell with:

- an **Open File** action
- an **Open Folder** action
- an empty sidebar or placeholder state
- an empty content area with guidance text

### 5.2 If the user opens a single file
The app should:

- Load that `.md` file.
- Set it as the current document.
- Show a minimal sidebar containing only that file, or a “single file mode” placeholder.
- Allow edit/view toggle.
- Auto-save edits.
- Watch the file for external changes.

### 5.3 If the user opens a folder
The app should:

- Recursively scan the folder.
- Build a sidebar tree from the folder structure.
- Treat subfolders as **categories**.
- Treat Markdown files as leaf nodes.
- Ignore any folder named exactly `asset` and skip its full subtree.
- Open a reasonable default document, preferably:
  1. root `README.md` if present
  2. otherwise the first Markdown file in sorted order
  3. otherwise no selection

### 5.4 Sidebar behavior
The sidebar must:

- Show categories and Markdown files.
- Support **expand/collapse** for categories.
- Support nested categories.
- Make the initial selected folder the top-level content root.
- Never show anything inside ignored `asset` directories.
- Allow selecting:
  - a category
  - a file

### 5.5 Main content behavior
The main content area has two modes:

#### Viewer mode
- Render the current Markdown file as Markdown.
- Scrollable.
- Links may be shown but do not need advanced navigation in v1.

#### Editor mode
- Show raw Markdown in a multiline text editor.
- Every change updates in-memory state immediately.
- Every change triggers save behavior automatically.

### 5.6 Create actions
When a **category/folder** is selected, the UI should expose actions to:

- **New Markdown File**
- **New Folder**

Creation should occur inside the selected category.

If a file is selected, creation actions should target the parent folder of that file.

---

## 6. Information Architecture

The app represents the filesystem as a tree rooted at the selected path.

### 6.1 Terminology
- **Root**: the user-selected folder, or the parent context around a single selected file.
- **Category**: any folder in the tree except ignored folders.
- **Document**: any `.md` file in the tree.
- **Selected node**: the current file or folder highlighted in the sidebar.
- **Current document**: the Markdown file currently loaded in the main pane.

### 6.2 Example mapping
Input structure:

```text
labeldesk-docs/
├── README.md
├── architecture.md
├── getting-started.md
├── environment-variables.md
├── third-party-services.md
├── authentication.md
├── database.md
├── features.md
├── decisions-and-challenges.md
└── projects/
    ├── backend/
    │   ├── README.md
    │   ├── api-reference.md
    │   ├── modules.md
    │   ├── guards-and-middleware.md
    │   └── jobs-and-queues.md
    ├── web-dashboard/
    │   ├── README.md
    │   ├── routes.md
    │   ├── state-management.md
    │   ├── components.md
    │   └── api-client.md
    ├── mobile-app/
    │   ├── README.md
    │   ├── screens.md
    │   ├── native-features.md
    │   ├── revenuecat.md
    │   └── state-management.md
    └── website/
        ├── README.md
        ├── pages.md
        └── known-issues.md
```

Expected sidebar model:

- labeldesk-docs
  - README.md
  - architecture.md
  - getting-started.md
  - environment-variables.md
  - third-party-services.md
  - authentication.md
  - database.md
  - features.md
  - decisions-and-challenges.md
  - projects
    - backend
      - README.md
      - api-reference.md
      - modules.md
      - guards-and-middleware.md
      - jobs-and-queues.md
    - web-dashboard
      - README.md
      - routes.md
      - state-management.md
      - components.md
      - api-client.md
    - mobile-app
      - README.md
      - screens.md
      - native-features.md
      - revenuecat.md
      - state-management.md
    - website
      - README.md
      - pages.md
      - known-issues.md

This means:
- `projects` is a category.
- `backend`, `web-dashboard`, `mobile-app`, and `website` are categories under `projects`.
- All folder levels must be expandable/collapsible independently.

---

## 7. Functional Requirements

## 7.1 Open root
The app must provide commands to:

1. Open a folder.
2. Open a single `.md` file.

### Acceptance criteria
- Folder selection loads the full tree rooted at that folder.
- File selection loads that file only.
- Non-Markdown files must not be shown in the document list in v1.

---

## 7.2 Sidebar tree
The app must build a tree from the selected root.

### Rules
- Include directories as categories.
- Include only files with `.md` extension, case-insensitive if feasible.
- Exclude directories named exactly `asset`.
- Exclude all descendants of any ignored `asset` directory.
- Sort folders before files.
- Sort alphabetically within each group.
- Tree must support arbitrary nesting depth.

### Acceptance criteria
- A directory named `asset` does not appear.
- A file `asset/example.md` never appears.
- Expanding/collapsing one category does not affect unrelated categories.

---

## 7.3 Category selection
The user must be able to click/select categories.

### When a category is selected
- It becomes the active sidebar selection.
- Creation actions target that category.
- Main content area may show:
  - category name
  - category path
  - actions such as `New Markdown File` and `New Folder`
  - optionally a small summary of child items

### Acceptance criteria
- User can select a folder even if no file is selected.
- New file/new folder operations use the selected folder as target.

---

## 7.4 File selection
The user must be able to select a Markdown file.

### When a file is selected
- The file loads into the main pane.
- The app can switch between viewer and editor mode for that file.
- The file becomes the current document.

### Acceptance criteria
- Clicking a file updates the main pane.
- File contents match the file on disk unless there is unsaved transient local state during save processing.

---

## 7.5 Viewer mode
The app must render Markdown for the current file.

### Requirements
- Headings, paragraphs, lists, code blocks, links, emphasis, blockquotes, tables/task lists if supported by the chosen renderer.
- Scrollable main area.
- Empty state when no document is selected.

### Acceptance criteria
- A valid Markdown file is shown in rendered form.
- Switching from editor to viewer uses the latest saved or in-memory content.

---

## 7.6 Editor mode
The app must show the raw Markdown source of the current file in an editor.

### Requirements
- Multi-line editing.
- Can edit the current file directly.
- Cursor/selection state is allowed to reset on external reload if implementation simplicity requires it, but preserving it is preferable.

### Acceptance criteria
- Typing changes text immediately.
- File content in memory matches what is visible in the editor.

---

## 7.7 Edit/view toggle
The app must have a visible toggle/button to switch between modes.

### Requirements
- Toggle should be accessible at all times when a file is selected.
- Switching mode must not lose the document content.
- No preview split view is required.

### Acceptance criteria
- User can move freely between viewer and editor.
- Current file remains selected.

---

## 7.8 Auto-save
The app must save changes automatically without manual intervention.

### Required behavior
- Every content edit updates in-memory state.
- The file is written to disk automatically.
- No explicit save button is required.

### Recommended implementation behavior
Because writing on every keystroke can be noisy and may cause watcher churn, use:

- immediate in-memory updates
- debounced disk save, e.g. 150–500 ms after the most recent edit

From the user’s point of view this still behaves as “auto-save on each change”.

### Acceptance criteria
- User types, pauses briefly, and file is saved.
- Leaving editor mode is not required for save.
- App close should flush any pending debounced save before exit.

---

## 7.9 External file change detection
If a currently visible file or any tree node is changed outside the app, the app must reflect it while running.

### Required behavior
- If the sidebar tree changes because folders/files are created, renamed, deleted, or moved externally, the tree refreshes.
- If the currently open Markdown file changes externally, the content in viewer/editor updates.

### Conflict rule
If the app has unsaved local edits when an external file change arrives, the app must not silently discard user edits.

Use one of these strategies:

#### Preferred strategy
Because auto-save is continuous, keep the unsaved window very small. Track a `dirty`/`save_pending` state.

- If external change arrives and no local dirty state exists: reload automatically.
- If external change arrives while local save is pending: defer or ignore the external event until save completes.
- If external change arrives and local content differs materially from disk while the app still considers itself dirty: show a small conflict banner or modal.

#### Simplified v1 strategy
- Debounced auto-save ensures the app is usually clean.
- Ignore watcher events that are the app’s own writes.
- If a genuine external change occurs for the current file and the document is not dirty, reload immediately.
- If dirty, show a conflict prompt:
  - Reload from disk
  - Keep local content

### Acceptance criteria
- Editing the file in another editor updates the app.
- Adding/removing Markdown files externally updates the sidebar.
- The app does not enter an infinite loop from its own save events.

---

## 7.10 Ignore `asset` folders
The app must ignore any directory named `asset`.

### Interpretation
- Exact folder name match: `asset`
- Exclude that folder itself from the sidebar.
- Exclude all children recursively.

### Clarification
- `assets` is not the same as `asset` unless product requirements later broaden this rule.
- In v1, ignore only exact `asset`.

### Acceptance criteria
- `docs/asset/logo.png` is ignored.
- `docs/asset/readme.md` is ignored.
- `docs/assets/readme.md` is not ignored unless later configured.

---

## 7.11 Create new Markdown file
When a category is selected, the user can create a new `.md` file.

### UX flow
- User selects category.
- User clicks `New Markdown File`.
- App opens a small input dialog/popover/modal for file name.
- User enters a base name or full filename.

### Rules
- If user omits `.md`, append `.md`.
- Validate name against OS-invalid characters where feasible.
- Prevent overwrite unless user explicitly confirms.
- After creation:
  - refresh tree
  - select the new file
  - open it in editor mode
  - initial content can be empty or optional template text

### Acceptance criteria
- New file appears in sidebar immediately.
- File exists on disk.
- User can start typing immediately.

---

## 7.12 Create new folder/category
When a category is selected, the user can create a new subfolder.

### UX flow
- User selects category.
- User clicks `New Folder`.
- App prompts for folder name.
- Folder is created under the selected category.

### Rules
- Validate name.
- Prevent duplicate folder names in same parent unless user confirms replacement is impossible and operation is rejected.
- After creation:
  - refresh tree
  - select the new folder
  - optionally auto-expand its parent

### Acceptance criteria
- Folder appears in sidebar immediately.
- It can be expanded/collapsed.

---

## 7.13 Deleted file handling
If the current file is deleted externally:

### Required behavior
- Sidebar refreshes.
- Current file selection is cleared or moved to a nearby valid item.
- Main pane shows a “file deleted” or empty state.

### Acceptance criteria
- App does not crash.
- Stale selection is not retained indefinitely.

---

## 7.14 Empty folder handling
If the selected folder contains no Markdown files:

### Required behavior
- Sidebar may still show categories.
- Main area shows an empty/documentation state.
- New file/new folder creation still works.

---

## 7.15 Read-only or permission errors
If file save/create operations fail due to permissions or locks:

### Required behavior
- Show a non-fatal error banner/toast/status message.
- Keep app responsive.
- Preserve in-memory content if save failed.

### Acceptance criteria
- Save failure does not crash the app.
- User receives clear feedback.

---

## 8. Suggested UI Layout

A practical layout for Iced:

### Top bar / toolbar
Contains:
- Open Folder
- Open File
- current root path
- mode toggle: Viewer / Editor
- optional status text: Saved / Saving... / Error

### Left sidebar
Contains:
- scrollable tree view
- category expand/collapse controls
- selected node highlight
- optional contextual actions for selected category

### Main content area
If a file is selected:
- viewer mode: rendered Markdown
- editor mode: text editor

If a category is selected:
- category details / actions panel

If nothing is selected:
- empty state

### Bottom status bar (optional but recommended)
Contains:
- current file path
- save state
- watcher status
- error text if present

---

## 9. State Model

A clean architecture is to centralize all app state in a single `App` struct.

## 9.1 Recommended state structures

```rust
struct App {
    root: Option<RootContext>,
    tree: Vec<TreeNode>,
    expanded: HashSet<PathBuf>,
    selected: Option<Selection>,
    current_doc: Option<OpenDocument>,
    mode: ContentMode,
    status: StatusState,
    pending_dialog: Option<DialogState>,
    watcher: WatcherState,
}
```

### RootContext
```rust
enum RootContext {
    Folder { root_path: PathBuf },
    SingleFile { file_path: PathBuf },
}
```

### Selection
```rust
enum Selection {
    Category(PathBuf),
    File(PathBuf),
}
```

### ContentMode
```rust
enum ContentMode {
    Viewer,
    Editor,
}
```

### TreeNode
```rust
struct TreeNode {
    path: PathBuf,
    name: String,
    kind: NodeKind,
    children: Vec<TreeNode>,
}

enum NodeKind {
    Category,
    MarkdownFile,
}
```

### OpenDocument
```rust
struct OpenDocument {
    path: PathBuf,
    text: String,
    editor_content: iced::widget::text_editor::Content,
    dirty: bool,
    save_pending: bool,
    last_saved_text: Option<String>,
    last_disk_mtime: Option<std::time::SystemTime>,
    external_change_pending: bool,
}
```

### StatusState
```rust
enum StatusState {
    Idle,
    Saving,
    Saved,
    Error(String),
    Conflict(String),
}
```

### DialogState
```rust
enum DialogState {
    NewMarkdownFile { parent: PathBuf, input: String },
    NewFolder { parent: PathBuf, input: String },
}
```

### WatcherState
This can hold:
- watcher handle
- event channel receiver state
- a set of temporarily ignored paths/timestamps for self-generated saves

---

## 10. Filesystem Scanning Rules

## 10.1 Traversal
Use a recursive directory walk rooted at the selected folder.

### Rules
- Skip symlink following in v1 unless deliberately supported.
- Ignore hidden file behavior can remain default; do not invent special hidden rules unless explicitly needed.
- Skip any directory named `asset` using directory pruning, not post-filtering.
- Include only `.md` files.

### Important
If possible, prune ignored directories early so children are never traversed.

## 10.2 Sorting
Each folder’s children should be sorted:
1. categories first
2. Markdown files second
3. alphabetical ascending by display name

## 10.3 Display names
Use the file/folder name only in sidebar labels, not the full path.

---

## 11. Watcher Design

## 11.1 Scope
When a folder root is open:
- watch the root recursively

When a single file is open:
- watch that file or its parent directory depending on crate/platform convenience

## 11.2 Events of interest
React to:
- create
- modify
- remove
- rename

## 11.3 Handling strategy
On filesystem event:

1. Determine whether it affects:
   - tree structure
   - current document
   - both
2. If tree-related, rescan tree or incrementally patch it.
3. If current document-related, reload or raise conflict handling.

## 11.4 Simplicity recommendation
For v1, **rescan the whole visible tree** after relevant directory events instead of building a fragile incremental tree updater. This is simpler and sufficiently fast for normal documentation-sized folders.

## 11.5 Self-write suppression
The app’s own auto-save writes will trigger watcher events.

To prevent feedback loops:
- record the path and timestamp/content hash of the app-initiated save
- ignore watcher events matching the just-written file within a short window

A simple suppression window of ~300–1000 ms is usually enough, but path + content comparison is more robust.

---

## 12. Save Strategy

## 12.1 Recommended write algorithm
When editor content changes:

1. Update `current_doc.text`
2. Mark `dirty = true`
3. Start/reset a debounce timer
4. When debounce fires:
   - write the file atomically if feasible
   - mark `dirty = false`
   - update save status
   - register self-write suppression metadata

## 12.2 Atomic save recommendation
Prefer:
- write to a temporary file in same directory
- rename/replace original

However, if this complicates watcher behavior on some platforms, direct overwrite is acceptable in v1.

## 12.3 Crash safety
On app exit, flush pending save immediately.

---

## 13. Markdown Rendering Behavior

In viewer mode, render the current file using Iced’s Markdown widget.

## 13.1 Rendering pipeline
- Take current document text.
- Parse Markdown.
- Display as a scrollable rendered document.

## 13.2 Consistency rule
Viewer mode should render the **current in-memory text**, not stale disk content.

That means if the user types, then instantly toggles to viewer before save completes, the viewer still reflects what they typed.

---

## 14. Editor Behavior

## 14.1 Content source of truth
There are two practical options:

### Option A — source of truth is plain `String`
- Editor is derived from string content
- On edits, rebuild/patch the string

### Option B — source of truth is Iced `text_editor::Content`
- String is derived from editor content when needed

### Recommendation
Use **both**:
- keep `text_editor::Content` for the widget
- keep a `String` mirror for saving/rendering

This avoids repeated costly conversion uncertainty and simplifies comparisons.

## 14.2 Selection and cursor
Preserving cursor state across mode switches is desirable but not mandatory for v1 if it significantly complicates the implementation.

---

## 15. Commands and Message Flow

Use a classic message/update/view architecture.

## 15.1 Recommended message enum

```rust
enum Message {
    OpenFolderPressed,
    OpenFilePressed,
    FolderChosen(Option<PathBuf>),
    FileChosen(Option<PathBuf>),

    TreeNodeToggled(PathBuf),
    CategorySelected(PathBuf),
    FileSelected(PathBuf),

    ModeSwitched(ContentMode),

    EditorAction(iced::widget::text_editor::Action),
    SaveDebounceElapsed,
    SaveCompleted(Result<(), String>),

    WatcherEvent(FileEvent),
    TreeReloaded(Result<TreeReload, String>),
    DocumentReloaded(Result<ReloadedDocument, String>),

    NewMarkdownFilePressed,
    NewFolderPressed,
    CreateDialogInputChanged(String),
    ConfirmCreateDialog,
    CancelCreateDialog,
    CreateCompleted(Result<CreateResult, String>),

    ConflictReloadFromDisk,
    ConflictKeepLocal,

    ClearStatus,
}
```

## 15.2 Important update rules
- Never block the UI thread with long disk operations.
- Use tasks/commands for file I/O where appropriate.
- Keep view pure and state-driven.

---

## 16. Suggested Tree-Building Algorithm

## 16.1 Folder root mode
Pseudo-logic:

```text
scan_folder(root):
  recursively walk root
  skip any directory whose name == "asset"
  include directories as category nodes
  include files if extension == .md
  sort children folders-first then files alphabetically
  return tree
```

## 16.2 Single file mode
Pseudo-logic:

```text
load_single_file(path):
  sidebar contains one file node or simplified placeholder
  current_doc = load(path)
  selection = File(path)
```

## 16.3 Expand/collapse state preservation
When rescanning the tree:
- preserve `expanded` state by path
- prune entries that no longer exist

This is important so the tree does not fully collapse every time the watcher fires.

---

## 17. Create File / Folder UX Details

## 17.1 Determining the target parent
If selected node is:
- Category(path) → parent target is `path`
- File(path) → parent target is `path.parent()`
- None → disable creation actions unless root folder is available

## 17.2 Filename normalization for new Markdown files
Input examples:
- `notes` → create `notes.md`
- `notes.md` → create `notes.md`
- `my file` → create `my file.md`

Reject:
- empty name
- `.`
- `..`
- platform-invalid names/characters

## 17.3 Post-create behavior
After a new Markdown file is created:
- refresh tree
- expand all ancestors needed to reveal it
- select it
- switch to editor mode
- load empty content

After a new folder is created:
- refresh tree
- expand parent
- select new folder

---

## 18. Error Handling Requirements

Errors should be visible but non-catastrophic.

## 18.1 Situations to handle
- folder open cancelled
- file open cancelled
- file read failed
- save failed
- watcher initialization failed
- tree scan failed for some subtree
- create file/folder failed
- invalid file/folder name
- current file deleted externally

## 18.2 Error presentation
Use one of:
- status bar error
- top banner
- toast
- modal for destructive/conflict situations

### Recommendation
- normal operational errors: banner/status
- overwrite/conflict decisions: modal/dialog

---

## 19. Performance Expectations

This is a small local docs app, not a full IDE.

### Therefore
- Full-tree rescans are acceptable for moderate folder sizes.
- Debounced saves are acceptable.
- Rendering Markdown only for the current document is enough.

### Avoid in v1
- over-engineered indexing
- virtualized tree unless needed later
- custom parser unless required

---

## 20. Accessibility / UX Quality

Recommended but not mandatory for the first pass:
- visible selected-state styling in sidebar
- keyboard navigation in sidebar
- keyboard shortcut for mode toggle
- keyboard shortcut for open file/folder
- keyboard shortcut for new file/new folder when category selected
- adequate contrast
- scroll position memory per document is a nice-to-have, not required

---

## 21. Proposed File/Module Layout

A maintainable project layout might look like this:

```text
src/
  main.rs
  app.rs
  message.rs
  state/
    mod.rs
    tree.rs
    document.rs
    selection.rs
    dialogs.rs
    status.rs
  fs/
    mod.rs
    scan.rs
    load.rs
    save.rs
    watch.rs
    create.rs
    ignore.rs
  ui/
    mod.rs
    sidebar.rs
    toolbar.rs
    content.rs
    dialogs.rs
    statusbar.rs
  model/
    mod.rs
    node.rs
    root.rs
    events.rs
  util/
    mod.rs
    paths.rs
    naming.rs
```

### Responsibilities
- `fs/scan.rs`: build tree from disk
- `fs/watch.rs`: notify watcher integration
- `fs/save.rs`: debounced save + atomic write logic
- `ui/sidebar.rs`: recursive tree rendering
- `ui/content.rs`: viewer/editor pane
- `state/document.rs`: current document state transitions

---

## 22. Suggested Milestones

## Milestone 1 — Skeleton app
- app shell
- toolbar
- empty state
- open folder/open file

## Milestone 2 — Tree navigation
- scan folder
- render sidebar tree
- expand/collapse categories
- file selection

## Milestone 3 — Document display
- viewer mode
- editor mode
- toggle between modes

## Milestone 4 — Auto-save
- track edits
- debounce save
- save state indicator

## Milestone 5 — File watching
- watch current root
- refresh tree on changes
- reload current file on external edits
- self-save suppression

## Milestone 6 — Create actions
- select categories
- create new Markdown file
- create new folder
- refresh/select new items

## Milestone 7 — Hardening
- conflicts
- deletion handling
- permission error UX
- polish

---

## 23. Acceptance Test Matrix

## 23.1 Folder open
- Open folder with nested subfolders and Markdown files.
- Verify nested folders become categories.
- Verify tree is sorted correctly.

## 23.2 Asset ignore
- Create `asset/` folder anywhere in tree with Markdown files inside.
- Verify folder and descendants do not appear.

## 23.3 File open
- Open a single `.md` file.
- Verify content loads and is editable.

## 23.4 Expand/collapse
- Expand several nested categories.
- Collapse one.
- Verify unrelated branches remain unchanged.

## 23.5 Mode switching
- Select file.
- Toggle viewer/editor repeatedly.
- Verify content remains consistent.

## 23.6 Auto-save
- Edit file.
- Pause.
- Verify file contents on disk match editor content.

## 23.7 External edit
- Open a file in the app.
- Edit it in another editor.
- Verify app updates.

## 23.8 External tree change
- Create/delete/rename Markdown files and folders externally.
- Verify sidebar refreshes.

## 23.9 New file
- Select category.
- Create `notes`.
- Verify `notes.md` is created, selected, opened in editor mode.

## 23.10 New folder
- Select category.
- Create `ideas`.
- Verify folder appears as category.

## 23.11 Delete current file externally
- Delete currently open file.
- Verify app remains stable and updates selection/UI.

## 23.12 Save failure
- Force permission error.
- Verify non-fatal error feedback.

---

## 24. Edge Cases

The implementation agent must explicitly handle these:

1. Opening a folder with no Markdown files.
2. Opening a folder with only ignored `asset` content.
3. Opening an unreadable file.
4. External file change during local typing.
5. Current file moved or renamed externally.
6. Parent folder removed externally.
7. Duplicate create name.
8. Invalid create name.
9. Very deep nesting.
10. Large Markdown files.
11. Markdown file with unusual Unicode filename.
12. Case variations in file extension such as `.MD`.

---

## 25. Recommended Defaults

### Initial expanded state
- root expanded
- first-level categories expanded is optional
- deeper categories collapsed by default is acceptable

### Initial mode
- default to **Viewer** when opening an existing file
- switch automatically to **Editor** after creating a new file

### Initial selected file in folder mode
- prefer root `README.md`
- else first Markdown file in tree order

### Save debounce
- start with **250 ms**

### Watcher refresh debounce
- start with **100–300 ms** to coalesce event bursts

---

## 26. Security / Safety Considerations

This is a local filesystem app, so keep scope narrow:
- never execute Markdown content
- do not interpret HTML or scripts as executable content
- avoid following symlinks unless intentionally supported
- do not allow path traversal when creating new files/folders beyond the selected target directory

---

## 27. Future Extensions (Not v1)

These are explicitly deferred but should not be blocked by the architecture:
- search across documents
- rename/move files/folders
- delete files/folders from inside app
- tabs / multiple open documents
- recent roots/files
- persisted expanded sidebar state
- custom ignore rules beyond `asset`
- theme support
- split preview mode
- drag and drop reordering
- backlinks/wiki-links
- image rendering from local files

---

## 28. Implementation Notes for the Agent

## 28.1 Iced-specific direction
Use Iced as the application shell and lean into its message-driven architecture.

Practical goals:
- keep one authoritative app state
- use tasks/commands for filesystem work
- keep filesystem watcher events translated into app messages
- keep view generation deterministic from state

## 28.2 Viewer/editor split
Do **not** build a side-by-side preview.
Only one of these should be visible at a time:
- Markdown viewer
- Markdown source editor

## 28.3 Simplicity bias
Prefer simple and correct over overly clever:
- whole-tree refreshes are fine
- path-based selection/expanded state is fine
- minimal conflict handling is acceptable if explicit

---

## 29. Minimal Deliverable Definition

The project is considered minimally complete when it can do all of the following reliably:

1. Open a folder or a single Markdown file.
2. Show a sidebar tree of folders-as-categories and Markdown files.
3. Ignore any `asset` folder recursively.
4. Expand/collapse categories.
5. Select categories and files.
6. Show current file in viewer mode.
7. Show current file in editor mode.
8. Auto-save changes without manual saving.
9. Detect external file/tree changes and refresh while running.
10. Create a new Markdown file or new folder in the selected category.

---

## 30. Final Build Directive for the Agent

Implement a cross-platform desktop app in Rust using Iced that behaves as a local Markdown documentation browser/editor backed directly by the filesystem.

The implementation must:
- support opening a folder or a single `.md` file
- treat folders as categories
- render the selected root folder as a nested expandable/collapsible sidebar tree
- ignore any `asset` directory and all descendants
- allow selecting categories and files
- allow switching between viewer mode and editor mode for the selected file
- allow direct editing of Markdown source in editor mode
- auto-save changes continuously with debounce-backed disk writes
- react to external file/folder changes while the app is open
- refresh the sidebar tree and current document accordingly
- allow creating a new `.md` file or new folder under the selected category
- remain stable and non-destructive in the presence of watcher events, save errors, and deleted files

The code should prioritize correctness, maintainability, and cross-platform behavior over premature optimization.
