# UI Infrastructure Specification
## petalTongue as Universal UI Infrastructure Primal

**Version**: 2.0.0  
**Date**: January 13, 2026  
**Status**: Formal Specification  
**Phase**: Design & Planning  
**Domain**: Universal UI Infrastructure

---

## 1. Executive Summary

### 1.1 Vision Statement

> **"petalTongue provides UI infrastructure primitives that enable on-the-fly interface creation for any scenario, not specific UI outcomes."**

**Evolution Path**:
- **Current**: Topology visualization tool
- **Future**: Universal UI infrastructure primal
- **Goal**: Become the "React/Vue of the primal ecosystem"

### 1.2 Core Principle

**Focus on Infrastructure, Not Outcomes**

```
❌ Don't Build:              ✅ Build Infrastructure That Enables:
   - Steam                      - Gaming platforms
   - Discord                    - Communication tools
   - VS Code                    - Code editors
   - Slack                      - Collaboration tools
   
petalTongue provides the primitives. Others compose them.
```

### 1.3 Success Criteria

**Phase 1** (Current - v1.6.0):
- ✅ Multi-modal rendering (GUI, TUI, Audio, API)
- ✅ Primal topology visualization
- ✅ TRUE PRIMAL architecture
- ✅ ToadStool basic integration

**Phase 2** (Target - v2.0.0):
- 🔲 Core rendering primitives (Tree, Table, Form, etc.)
- 🔲 Panel layout system
- 🔲 Extension architecture
- 🔲 ToadStool deep integration
- 🔲 Real-time collaboration primitives

**Phase 3** (Vision - v3.0.0):
- 🔲 Schema-driven UI generation
- 🔲 AI-assisted layout (via Squirrel)
- 🔲 On-the-fly UI creation
- 🔲 Multi-user editing

---

## 2. Architecture Philosophy

### 2.1 Separation of Concerns

**petalTongue's Domain** (What We Own):
```rust
// Rendering Primitives
render_tree(data: &TreeNode) -> Result<()>
render_table(data: &TableData) -> Result<()>
render_form(schema: &FormSchema) -> Result<()>
render_code(text: &str, lang: Language) -> Result<()>

// Layout Primitives
panel_layout(regions: Vec<Panel>) -> Result<()>
tab_system(tabs: Vec<Tab>) -> Result<()>
overlay(content: impl Render, modal: bool) -> Result<()>

// Interaction Primitives
command_palette(commands: Vec<Command>) -> Result<()>
drag_drop(source: DragSource, target: DropTarget) -> Result<()>
context_menu(actions: Vec<Action>) -> Result<()>
```

**Other Primals' Domains** (What We Leverage):
```rust
// ToadStool: Heavy Computation
toadstool.compute_layout(graph) -> Layout
toadstool.syntax_highlight(code, lang) -> HighlightedText
toadstool.search_index(content) -> SearchResults

// Squirrel: AI Assistance
squirrel.suggest_layout(data) -> LayoutSuggestion
squirrel.autocomplete(prefix, context) -> Vec<Completion>
squirrel.summarize(content) -> Summary

// Songbird: Discovery & Events
songbird.discover_capabilities() -> Vec<Capability>
songbird.subscribe_events() -> EventStream
songbird.broadcast_presence(state) -> Result<()>

// NestGate: Persistence
nestgate.save_workspace(state) -> Result<()>
nestgate.load_preferences(key) -> Result<Preferences>
nestgate.store_session(session) -> Result<()>

// BearDog: Security
beardog.authenticate(user) -> Result<Session>
beardog.authorize(user, resource) -> Result<bool>
beardog.encrypt(data) -> Result<Vec<u8>>
```

### 2.2 Declarative API Design

**Pattern**: Describe WHAT, not HOW

```rust
// ❌ Bad: Imperative (current egui style)
if ui.button("Save").clicked() {
    save_file();
}
if ui.button("Load").clicked() {
    load_file();
}

// ✅ Good: Declarative (future petalTongue style)
UIBuilder::new()
    .panel("left", render_file_tree)
    .panel("center", render_editor)
    .panel("bottom", render_terminal)
    .on_command("save", save_file)
    .on_command("load", load_file)
    .build()?
```

### 2.3 Progressive Enhancement

**Graceful Degradation Based on Capabilities**

```rust
// Detect available capabilities
let capabilities = detect_universe_capabilities().await?;

// Adapt UI accordingly
let ui = match capabilities {
    // Best: Full GUI + ToadStool GPU
    Capabilities { display: Display::GUI, toadstool: Some(_), .. } => {
        UIBuilder::new()
            .with_gpu_rendering()
            .with_animations()
            .with_rich_panels()
    }
    
    // Good: GUI without GPU
    Capabilities { display: Display::GUI, .. } => {
        UIBuilder::new()
            .with_cpu_rendering()
            .with_simple_panels()
    }
    
    // Acceptable: Terminal only
    Capabilities { display: Display::Terminal, .. } => {
        UIBuilder::new()
            .terminal_mode()
            .with_text_only()
    }
    
    // Fallback: API only (headless)
    _ => {
        UIBuilder::new()
            .headless()
            .api_only()
    }
};
```

---

## 3. Core Primitives Specification

### 3.1 Rendering Primitives

#### 3.1.1 Tree Primitive

**Purpose**: Render hierarchical data (files, categories, org charts, etc.)

```rust
/// Tree node primitive
pub struct TreeNode<T> {
    pub data: T,
    pub children: Vec<TreeNode<T>>,
    pub expanded: bool,
    pub icon: Option<Icon>,
}

pub trait TreeRenderer {
    /// Render a tree with callback on selection
    fn render_tree<T>(&mut self, 
        root: &TreeNode<T>,
        on_select: impl Fn(&T),
    ) -> Result<()>;
    
    /// Filter tree based on predicate
    fn filter_tree<T>(&mut self, 
        root: &TreeNode<T>,
        predicate: impl Fn(&T) -> bool,
    ) -> Result<()>;
    
    /// Expand to specific path
    fn expand_to(&mut self, path: &[String]) -> Result<()>;
}
```

**Use Cases**: File browsers, category navigation, org charts, menu systems

#### 3.1.2 Table Primitive

**Purpose**: Render tabular data (logs, metrics, search results, etc.)

```rust
/// Table data primitive
pub struct TableData {
    pub columns: Vec<Column>,
    pub rows: Vec<Row>,
    pub sortable: bool,
    pub filterable: bool,
}

pub trait TableRenderer {
    /// Render a table with sorting/filtering
    fn render_table(&mut self, data: &TableData) -> Result<()>;
    
    /// Sort by column
    fn sort_by(&mut self, column: usize, direction: SortDirection) -> Result<()>;
    
    /// Filter rows
    fn filter_rows(&mut self, predicate: impl Fn(&Row) -> bool) -> Result<()>;
    
    /// Select rows
    fn on_row_select(&mut self, callback: impl Fn(&Row)) -> Result<()>;
}
```

**Use Cases**: Logs, metrics, search results, data tables, spreadsheets

#### 3.1.3 Form Primitive

**Purpose**: Render editable data (settings, config, data entry, etc.)

```rust
/// Form schema primitive
pub struct FormSchema {
    pub fields: Vec<FormField>,
    pub validation: ValidationRules,
}

pub enum FormField {
    Text { label: String, default: String },
    Number { label: String, min: f64, max: f64 },
    Select { label: String, options: Vec<String> },
    Checkbox { label: String, checked: bool },
    Date { label: String, format: DateFormat },
    File { label: String, accept: Vec<String> },
}

pub trait FormRenderer {
    /// Render a form from schema
    fn render_form(&mut self, schema: &FormSchema) -> Result<FormData>;
    
    /// Validate form data
    fn validate(&self, data: &FormData) -> Result<Vec<ValidationError>>;
    
    /// On submit callback
    fn on_submit(&mut self, callback: impl Fn(FormData)) -> Result<()>;
}
```

**Use Cases**: Settings panels, configuration UIs, data entry forms

#### 3.1.4 Code Primitive

**Purpose**: Render syntax-highlighted code (editor, diff viewer, etc.)

```rust
/// Code primitive
pub struct CodeBuffer {
    pub text: String,
    pub language: Language,
    pub line_numbers: bool,
    pub readonly: bool,
}

pub trait CodeRenderer {
    /// Render code with syntax highlighting
    fn render_code(&mut self, buffer: &CodeBuffer) -> Result<()>;
    
    /// Render diff (side-by-side or inline)
    fn render_diff(&mut self, 
        before: &CodeBuffer,
        after: &CodeBuffer,
        mode: DiffMode,
    ) -> Result<()>;
    
    /// On edit callback
    fn on_edit(&mut self, callback: impl Fn(&str)) -> Result<()>;
}
```

**Use Cases**: Code editors, diff viewers, log viewers, markdown preview

#### 3.1.5 Timeline Primitive

**Purpose**: Render temporal data (history, events, gantt charts, etc.)

```rust
/// Timeline primitive
pub struct Timeline {
    pub events: Vec<TimelineEvent>,
    pub range: TimeRange,
    pub zoom_level: f32,
}

pub trait TimelineRenderer {
    /// Render timeline
    fn render_timeline(&mut self, timeline: &Timeline) -> Result<()>;
    
    /// Zoom to range
    fn zoom_to(&mut self, range: TimeRange) -> Result<()>;
    
    /// On event select
    fn on_event_select(&mut self, callback: impl Fn(&TimelineEvent)) -> Result<()>;
}
```

**Use Cases**: Git history, event logs, gantt charts, audit trails

#### 3.1.6 Chat Primitive

**Purpose**: Render message streams (chat, comments, notifications, etc.)

```rust
/// Chat primitive
pub struct ChatStream {
    pub messages: Vec<Message>,
    pub users: Vec<User>,
    pub auto_scroll: bool,
}

pub trait ChatRenderer {
    /// Render chat stream
    fn render_chat(&mut self, stream: &ChatStream) -> Result<()>;
    
    /// Send message
    fn on_message_send(&mut self, callback: impl Fn(&str)) -> Result<()>;
    
    /// On user mention
    fn on_mention(&mut self, callback: impl Fn(&User)) -> Result<()>;
}
```

**Use Cases**: Chat interfaces, comment threads, notifications, activity feeds

#### 3.1.7 Dashboard Primitive

**Purpose**: Render metrics and KPIs (monitoring, analytics, etc.)

```rust
/// Dashboard primitive
pub struct Dashboard {
    pub widgets: Vec<DashboardWidget>,
    pub layout: GridLayout,
    pub refresh_interval: Option<Duration>,
}

pub enum DashboardWidget {
    Metric { label: String, value: f64, unit: String },
    Graph { data: TimeSeriesData, kind: GraphKind },
    Status { label: String, state: HealthState },
    Alert { level: AlertLevel, message: String },
}

pub trait DashboardRenderer {
    /// Render dashboard
    fn render_dashboard(&mut self, dashboard: &Dashboard) -> Result<()>;
    
    /// Update widget
    fn update_widget(&mut self, id: &str, widget: DashboardWidget) -> Result<()>;
}
```

**Use Cases**: System monitoring, analytics, business metrics, health dashboards

#### 3.1.8 Canvas Primitive

**Purpose**: Free-form drawing (diagrams, sketches, visualizations, etc.)

```rust
/// Canvas primitive
pub struct Canvas {
    pub width: u32,
    pub height: u32,
    pub elements: Vec<CanvasElement>,
}

pub enum CanvasElement {
    Line { from: Point, to: Point, color: Color, width: f32 },
    Rect { pos: Point, size: Size, fill: Color },
    Circle { center: Point, radius: f32, fill: Color },
    Text { pos: Point, text: String, font: Font },
    Image { pos: Point, data: Vec<u8> },
}

pub trait CanvasRenderer {
    /// Render canvas
    fn render_canvas(&mut self, canvas: &Canvas) -> Result<()>;
    
    /// Add element
    fn add_element(&mut self, element: CanvasElement) -> Result<()>;
    
    /// On mouse draw
    fn on_draw(&mut self, callback: impl Fn(Point, Point)) -> Result<()>;
}
```

**Use Cases**: Diagrams, flowcharts, whiteboarding, data visualization

---

### 3.2 Layout Primitives

#### 3.2.1 Panel System

**Purpose**: Dockable, resizable regions

```rust
/// Panel layout primitive
pub struct PanelLayout {
    pub panels: HashMap<String, Panel>,
    pub splits: Vec<Split>,
}

pub struct Panel {
    pub id: String,
    pub region: Region,
    pub render_fn: Box<dyn Fn(&mut Frame)>,
    pub size: Size,
    pub resizable: bool,
}

pub enum Region {
    Left,
    Right,
    Top,
    Bottom,
    Center,
    Floating { x: f32, y: f32 },
}

pub trait PanelLayoutManager {
    /// Add panel to region
    fn add_panel(&mut self, id: &str, region: Region, render_fn: RenderFn) -> Result<()>;
    
    /// Split panel
    fn split_panel(&mut self, id: &str, direction: SplitDirection) -> Result<()>;
    
    /// Resize panel
    fn resize_panel(&mut self, id: &str, size: Size) -> Result<()>;
    
    /// Dock panel to another
    fn dock_panel(&mut self, source: &str, target: &str, position: DockPosition) -> Result<()>;
}
```

**Use Cases**: IDE layouts, dashboard panels, multi-pane interfaces

#### 3.2.2 Tab System

**Purpose**: Multiple contexts in same space

```rust
/// Tab system primitive
pub struct TabSystem {
    pub tabs: Vec<Tab>,
    pub active: usize,
    pub closeable: bool,
}

pub struct Tab {
    pub id: String,
    pub label: String,
    pub icon: Option<Icon>,
    pub render_fn: Box<dyn Fn(&mut Frame)>,
    pub dirty: bool,
}

pub trait TabManager {
    /// Add tab
    fn add_tab(&mut self, id: &str, label: &str, render_fn: RenderFn) -> Result<()>;
    
    /// Switch to tab
    fn switch_to(&mut self, id: &str) -> Result<()>;
    
    /// Close tab
    fn close_tab(&mut self, id: &str) -> Result<()>;
    
    /// On tab close (allow veto)
    fn on_tab_close(&mut self, callback: impl Fn(&str) -> bool) -> Result<()>;
}
```

**Use Cases**: Multi-file editors, tabbed browsers, workspace switchers

#### 3.2.3 Overlay System

**Purpose**: Modal and non-modal overlays

```rust
/// Overlay primitive
pub struct Overlay {
    pub content: Box<dyn Fn(&mut Frame)>,
    pub modal: bool,
    pub position: OverlayPosition,
    pub size: OverlaySize,
}

pub enum OverlayPosition {
    Center,
    TopRight,
    BottomRight,
    Custom { x: f32, y: f32 },
}

pub trait OverlayManager {
    /// Show overlay
    fn show_overlay(&mut self, overlay: Overlay) -> Result<()>;
    
    /// Close current overlay
    fn close_overlay(&mut self) -> Result<()>;
    
    /// Show notification
    fn notify(&mut self, message: &str, level: NotificationLevel) -> Result<()>;
}
```

**Use Cases**: Dialogs, popups, notifications, tooltips, modals

---

### 3.3 Interaction Primitives

#### 3.3.1 Command Palette

**Purpose**: Universal command access via fuzzy search

```rust
/// Command palette primitive
pub struct CommandPalette {
    pub commands: Vec<Command>,
    pub recent: Vec<String>,
    pub fuzzy_search: bool,
}

pub struct Command {
    pub id: String,
    pub label: String,
    pub description: Option<String>,
    pub shortcut: Option<Shortcut>,
    pub action: Box<dyn Fn()>,
}

pub trait CommandPaletteManager {
    /// Register command
    fn register_command(&mut self, command: Command) -> Result<()>;
    
    /// Show palette
    fn show_palette(&mut self) -> Result<()>;
    
    /// Execute command by ID
    fn execute(&mut self, id: &str) -> Result<()>;
}
```

**Use Cases**: VS Code command palette, Spotlight search, quick actions

#### 3.3.2 Drag and Drop

**Purpose**: Move/copy items between regions

```rust
/// Drag and drop primitive
pub trait DragDropManager {
    /// Register drag source
    fn drag_source<T>(&mut self, 
        id: &str,
        data: T,
        on_drag_start: impl Fn(&T),
    ) -> Result<()>;
    
    /// Register drop target
    fn drop_target<T>(&mut self,
        id: &str,
        on_drop: impl Fn(&T) -> Result<()>,
    ) -> Result<()>;
    
    /// Check if drop allowed
    fn can_drop(&self, source: &str, target: &str) -> bool;
}
```

**Use Cases**: File organization, panel rearrangement, priority ordering

#### 3.3.3 Context Menu

**Purpose**: Right-click actions

```rust
/// Context menu primitive
pub struct ContextMenu {
    pub items: Vec<MenuItem>,
}

pub enum MenuItem {
    Action { label: String, action: Box<dyn Fn()>, shortcut: Option<Shortcut> },
    Separator,
    Submenu { label: String, items: Vec<MenuItem> },
}

pub trait ContextMenuManager {
    /// Show context menu at position
    fn show_menu(&mut self, menu: ContextMenu, position: Point) -> Result<()>;
    
    /// Close menu
    fn close_menu(&mut self) -> Result<()>;
}
```

**Use Cases**: File operations, text editing actions, custom actions

---

### 3.4 State Management Primitives

#### 3.4.1 Undo/Redo System

**Purpose**: Action history with undo/redo

```rust
/// Undo/redo primitive
pub trait UndoManager {
    /// Record action
    fn record<T>(&mut self, 
        action: Action<T>,
        undo: impl Fn(&T),
        redo: impl Fn(&T),
    ) -> Result<()>;
    
    /// Undo last action
    fn undo(&mut self) -> Result<()>;
    
    /// Redo last undone action
    fn redo(&mut self) -> Result<()>;
    
    /// Clear history
    fn clear(&mut self) -> Result<()>;
}
```

**Use Cases**: Text editing, drawing, configuration changes

#### 3.4.2 Session State

**Purpose**: Remember state across restarts

```rust
/// Session state primitive
pub trait SessionManager {
    /// Save current state
    fn save_state<T: Serialize>(&self, key: &str, state: &T) -> Result<()>;
    
    /// Load state
    fn load_state<T: DeserializeOwned>(&self, key: &str) -> Result<Option<T>>;
    
    /// Save workspace
    fn save_workspace(&self, workspace: &Workspace) -> Result<()>;
    
    /// Load workspace
    fn load_workspace(&self, id: &str) -> Result<Option<Workspace>>;
}
```

**Use Cases**: Resume where you left off, workspace management

---

## 4. ToadStool Deep Integration

### 4.1 Compute Delegation Patterns

**Heavy computation offloaded to ToadStool**

```rust
/// ToadStool integration for UI infrastructure
pub trait ToadStoolUIBridge {
    /// Compute complex layouts
    async fn compute_layout(&self, graph: &GraphData) -> Result<Layout>;
    
    /// Syntax highlighting (language server)
    async fn highlight_syntax(&self, code: &str, lang: Language) -> Result<HighlightedText>;
    
    /// Diff computation
    async fn compute_diff(&self, before: &str, after: &str) -> Result<DiffResult>;
    
    /// Search indexing
    async fn index_content(&self, content: &[Document]) -> Result<SearchIndex>;
    
    /// Full-text search
    async fn search(&self, query: &str, index: &SearchIndex) -> Result<Vec<SearchResult>>;
    
    /// Render complex visualization
    async fn render_visualization(&self, data: &VisualizationData) -> Result<RenderOutput>;
}
```

### 4.2 Benefits of ToadStool Integration

**Performance**:
- Layout computation: 10x faster with GPU
- Syntax highlighting: Parallel processing
- Search: Distributed indexing

**Scalability**:
- Handle large files (>100MB)
- Index entire codebases
- Real-time collaboration sync

**Battery Life**:
- Offload to desktop/server
- Mobile/laptop stays light

---

## 5. Extension Architecture

### 5.1 Extension Points

**Where extensions can hook in**

```rust
/// Extension trait for petalTongue
pub trait UIExtension {
    /// Extension metadata
    fn metadata(&self) -> ExtensionMetadata;
    
    /// Register commands
    fn register_commands(&self) -> Vec<Command>;
    
    /// Render custom panel
    fn render_panel(&self, ctx: &mut RenderContext) -> Result<()>;
    
    /// Handle custom events
    fn handle_event(&self, event: &Event) -> Result<()>;
    
    /// Provide completions
    fn provide_completions(&self, context: &CompletionContext) -> Vec<Completion>;
    
    /// Initialize extension
    fn initialize(&mut self, ctx: &ExtensionContext) -> Result<()>;
}

pub struct ExtensionMetadata {
    pub id: String,
    pub name: String,
    pub version: String,
    pub author: String,
    pub description: String,
    pub capabilities: Vec<String>,
}
```

### 5.2 Extension Loading

**How extensions are discovered and loaded**

```rust
/// Extension registry
pub struct ExtensionRegistry {
    extensions: HashMap<String, Box<dyn UIExtension>>,
}

impl ExtensionRegistry {
    /// Load extension from WASM module
    pub fn load_wasm(&mut self, path: &Path) -> Result<()>;
    
    /// Load extension from dynamic library
    pub fn load_dylib(&mut self, path: &Path) -> Result<()>;
    
    /// Load built-in extension
    pub fn register<E: UIExtension + 'static>(&mut self, extension: E) -> Result<()>;
    
    /// Get extension by ID
    pub fn get(&self, id: &str) -> Option<&dyn UIExtension>;
}
```

---

## 6. Data-Driven UI Generation

### 6.1 Schema-Driven Forms

**Generate forms from data schemas**

```rust
/// Derive UI from data schema
#[derive(UISchema)]
pub struct PrimalConfig {
    #[ui(widget = "text", label = "Primal Name")]
    name: String,
    
    #[ui(widget = "number", min = 1024, max = 65535, label = "Port")]
    port: u16,
    
    #[ui(widget = "select", options = ["debug", "info", "warn", "error"])]
    log_level: String,
    
    #[ui(widget = "checkbox", label = "Enable GPU Acceleration")]
    enable_gpu: bool,
    
    #[ui(widget = "file", accept = [".toml", ".json"], label = "Config File")]
    config_path: Option<PathBuf>,
}

// Auto-generate form UI
let form = FormBuilder::from_schema::<PrimalConfig>()?;
let data = ui.render_form(form)?;
```

### 6.2 Layout from Data Structure

**Automatically choose best layout based on data**

```rust
/// Generate UI from arbitrary data
let data = json!({
    "users": [...],  // Array → Table
    "settings": {...},  // Object → Form
    "logs": [...],  // Time-series → Timeline
    "files": {...},  // Hierarchical → Tree
});

// petalTongue chooses optimal representation
let ui = UIGenerator::from_data(&data)
    .with_capabilities(capabilities)
    .adapt_to_user(user_prefs)
    .generate()?;
```

---

## 7. Implementation Roadmap

### Phase 1: Foundation (Complete - v1.6.0) ✅
- ✅ Multi-modal rendering
- ✅ Graph visualization
- ✅ ToadStool basic integration
- ✅ TRUE PRIMAL architecture
- ✅ Primal discovery

### Phase 2: Core Primitives (v2.0.0 - 3 months)
**Priority Order**:
1. 🔲 **Tree Renderer** (1 month)
   - File browser primitive
   - Category navigation
   - Hierarchical data display

2. 🔲 **Table Renderer** (2 weeks)
   - Log viewer
   - Data tables
   - Search results

3. 🔲 **Panel Layout System** (3 weeks)
   - Dockable panels
   - Resizable regions
   - Tab system

4. 🔲 **Command Palette** (1 week)
   - Fuzzy command search
   - Keyboard shortcuts
   - Command registration

5. 🔲 **Form Renderer** (2 weeks)
   - Settings panels
   - Configuration UIs
   - Data entry

### Phase 3: Advanced Features (v2.5.0 - 3 months)
6. 🔲 **Code Renderer** (1 month)
   - Syntax highlighting (via ToadStool)
   - Diff viewer
   - Code editor mode

7. 🔲 **Extension System** (1 month)
   - Extension points
   - WASM module loading
   - Built-in extension API

8. 🔲 **Real-Time Collaboration** (1 month)
   - Presence system
   - Live cursors
   - Change synchronization

### Phase 4: ToadStool Deep Integration (v3.0.0 - 3 months)
9. 🔲 **Compute Offloading** (1 month)
   - Layout computation
   - Syntax highlighting
   - Search indexing

10. 🔲 **Schema-Driven UI** (1 month)
    - Auto-generate forms
    - Layout inference
    - Type-aware rendering

11. 🔲 **AI-Assisted Layout** (1 month via Squirrel)
    - Suggested layouts
    - Automatic responsiveness
    - Accessibility automation

---

## 8. API Design Examples

### 8.1 Building an IDE

```rust
// Build a code editor using petalTongue primitives
let ide = UIBuilder::new()
    // Left sidebar: File tree
    .panel("files", Region::Left, |ui| {
        ui.render_tree(&file_system_root, |file| {
            editor.open(file);
        })
    })
    
    // Center: Code editor
    .panel("editor", Region::Center, |ui| {
        ui.render_code(&current_file, Language::Rust)?;
    })
    
    // Bottom: Terminal + problems
    .panel("bottom", Region::Bottom, |ui| {
        ui.tabs()
            .tab("terminal", render_terminal)
            .tab("problems", render_problems)
            .render()
    })
    
    // Right: Debug panel
    .panel("debug", Region::Right, |ui| {
        ui.render_tree(&call_stack, on_frame_select)?;
        ui.render_table(&variables)?;
    })
    
    // Commands
    .command("save", Shortcut::Ctrl('S'), save_file)
    .command("find", Shortcut::Ctrl('F'), show_find)
    .command("palette", Shortcut::CtrlShift('P'), show_palette)
    
    // ToadStool integration
    .with_toadstool(|toadstool| {
        toadstool.enable_syntax_highlighting();
        toadstool.enable_language_server();
    })
    
    .build()?;
```

### 8.2 Building a Dashboard

```rust
// Build a monitoring dashboard
let dashboard = UIBuilder::new()
    .dashboard(|db| {
        // Metrics row
        db.widget(Metric {
            label: "CPU Usage",
            value: cpu_percent,
            unit: "%",
            threshold: Some(80.0),
        });
        
        db.widget(Metric {
            label: "Memory",
            value: mem_used,
            unit: "GB",
            threshold: Some(16.0),
        });
        
        // Graph row
        db.widget(Graph {
            title: "Network Traffic",
            data: network_time_series,
            kind: GraphKind::Line,
        });
        
        // Alert panel
        db.widget(AlertList {
            alerts: active_alerts,
            on_select: view_alert_details,
        });
    })
    
    // Real-time updates via Songbird
    .subscribe_events(|event| {
        match event {
            Event::MetricUpdate(metric) => dashboard.update(metric),
            Event::Alert(alert) => dashboard.add_alert(alert),
            _ => {}
        }
    })
    
    .build()?;
```

### 8.3 Building a Chat Interface

```rust
// Build a collaboration tool
let chat = UIBuilder::new()
    // Left: User list + channels
    .panel("sidebar", Region::Left, |ui| {
        ui.render_tree(&channels, on_channel_select)?;
        ui.render_tree(&users, on_user_select)?;
    })
    
    // Center: Messages
    .panel("messages", Region::Center, |ui| {
        ui.render_chat(&message_stream)?;
    })
    
    // Bottom: Message input
    .panel("input", Region::Bottom, |ui| {
        ui.text_input(|message| {
            send_message(message);
        })
    })
    
    // Real-time via Songbird
    .subscribe_events(|event| {
        match event {
            Event::Message(msg) => chat.append_message(msg),
            Event::UserJoined(user) => chat.add_user(user),
            Event::UserLeft(user) => chat.remove_user(user),
            Event::Typing(user) => chat.show_typing(user),
            _ => {}
        }
    })
    
    .build()?;
```

---

## 9. Success Metrics

### 9.1 Developer Experience

**Goal**: Make UI creation simple and fast

**Metrics**:
- Lines of code to build basic UI: < 50
- Time to prototype interface: < 1 hour
- Learning curve: < 1 day for basics
- Extension development time: < 1 week

### 9.2 Performance

**Goal**: Fast, responsive, efficient

**Metrics**:
- Frame rate: 60 FPS minimum
- Startup time: < 1 second
- Memory usage: < 100MB base
- ToadStool offload: 10x speedup on heavy operations

### 9.3 Adoption

**Goal**: Become standard UI infrastructure for primal ecosystem

**Metrics**:
- Number of primals using petalTongue: > 5
- Number of custom UIs built: > 10
- Number of extensions: > 20
- Community satisfaction: > 90%

---

## 10. Appendices

### A. Comparison with Existing UI Frameworks

| Feature | petalTongue | egui | iced | gtk | Qt |
|---------|------------|------|------|-----|-----|
| Multi-modal | ✅ | ❌ | ❌ | ❌ | ❌ |
| TUI support | ✅ | ❌ | ❌ | ❌ | ❌ |
| Audio rendering | ✅ | ❌ | ❌ | ❌ | ❌ |
| Primal network | ✅ | ❌ | ❌ | ❌ | ❌ |
| Declarative API | ✅ | Partial | ✅ | ❌ | ❌ |
| Extension system | ✅ | ❌ | ❌ | Limited | ✅ |
| Pure Rust | ✅ | ✅ | ✅ | ❌ | ❌ |

### B. Terminology

- **Primitive**: Basic building block (tree, table, form, etc.)
- **Panel**: Dockable UI region
- **Modality**: Rendering mode (GUI, TUI, Audio, API)
- **Extension**: Third-party plugin that adds functionality
- **ToadStool**: Compute primal for heavy processing
- **Songbird**: Discovery primal for primal network
- **Schema**: Data structure definition that drives UI generation

---

**Status**: Specification complete, ready for implementation  
**Next**: Create tracking document and begin Phase 2 implementation

🌸 **petalTongue v2.0: Universal UI Infrastructure Primal** 🚀

