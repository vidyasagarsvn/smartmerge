# SmartMerge - Project Conversation History

**Last Updated:** February 11, 2026

## Project Overview
Building a cross-platform file and folder diff/merge tool similar to TortoiseMerge/Beyond Compare.

---

## Tool Name: SmartMerge
**Final Decision:** SmartMerge

### Why SmartMerge?
- Familiar and professional
- Implies intelligent conflict resolution
- Not widely trademarked
- Easy to remember and pronounce

---

## Architecture - UPDATED (February 11, 2026)

### PRIMARY STACK: C# + WPF (Windows Branch)
- **Language:** C# (.NET 8+)
- **UI Framework:** WPF (Windows Presentation Foundation)
- **Target:** Windows-native performance and integration
- **Branch:** `windows`

### Why C# + WPF?
- ✅ Performance: Compiled, near-C++/C speeds, optimized GC
- ✅ Accuracy: Strong type system prevents bugs in diff logic
- ✅ Windows Integration: Native Explorer context menus, file dialogs
- ✅ Modern Development: LINQ for algorithms, latest language features
- ✅ Rich Ecosystem: Mature NuGet packages (DiffPlex, LibGit2Sharp)
- ✅ Professional Tools: VS Code, Visual Studio, debugger support

### Previous Stack (DEPRECATED)
- **Reason for Rejection:** Tauri encountered unending integration issues
- **Archive:** See git history for Tauri branch decisions

---

## Deliverables

### Output
- **Windows:** `.exe` installer + portable executable
- **macOS:** `.dmg` or `.app` bundle
- **Linux:** `.AppImage`, `.deb`, or `.rpm` packages
- **All:** Fully self-contained executables (~10MB for Tauri-based)

### User Experience
- Double-click to run (no dependencies)
- Native performance on all platforms
- Single codebase, multiple platform binaries

---

## Key Features to Implement

### Core Diff Algorithms
- **Three-way merge** for conflict resolution
- **Patience diff** for improved readability
- **Longest Common Subsequence (LCS)** algorithms
- Streaming/chunked processing for large files

### User Interface
- Visual file comparison with side-by-side view
- Syntax highlighting (via Tree-sitter)
- Conflict highlighting and resolution
- Folder/directory diff and sync

### Advanced Features
- Auto-detect encoding (UTF-8, UTF-16, binary)
- Handle line ending differences (CRLF vs LF)
- Memory-mapped I/O for large files
- Worker threads for responsive UI
- Plugin system via WebAssembly (WASM)
- Lazy rendering for performance

---

## GitHub Repository Description

### Recommended (Option 2 - Developer-focused):
```
SmartMerge - Cross-platform visual diff and merge tool with visual highlighting 
and intelligent conflict resolution. Built with Rust + Tauri for blazing-fast performance.
```

### Alternative (Option 5 - Feature-focused):
```
SmartMerge - A modern, cross-platform alternative to WinMerge. Compare and merge 
files/folders with visual highlighting, syntax awareness, and powerful diff algorithms. 
Lightweight (~10MB), built with Rust and runs natively on Windows, macOS, and Linux.
```

### README.md Template

```markdown
# SmartMerge

A lightweight, cross-platform visual diff and merge tool designed for developers and teams.

## Features
- Visual file and folder comparison with syntax highlighting
- Intelligent 3-way merge for conflict resolution
- Cross-platform: Windows, macOS, Linux
- Ultra-lightweight: ~10MB binary
- Fast performance with Rust backend
- Open-source and free
- Native performance on all platforms

## Installation
[Download latest release](https://github.com/yourusername/smartmerge/releases)

## Building from Source
### Prerequisites
- Rust 1.70+
- Node.js 16+

### Build
```bash
cargo tauri build
```

## Tech Stack
- **Backend:** Rust (core diff engine)
- **Frontend:** React/Vue.js
- **Desktop Framework:** Tauri
```

---

## Development Roadmap (Suggested)

### Phase 1: MVP
- [x] Project architecture designed
- [ ] Basic two-file diff comparison
- [ ] Side-by-side view
- [ ] Save comparison results

### Phase 2: Core Features
- [ ] Folder/directory diff
- [ ] 3-way merge
- [ ] Syntax highlighting
- [ ] Encoding detection

### Phase 3: Advanced Features
- [ ] Partial merge (line-by-line)
- [ ] Version control integration (Git)
- [ ] Plugin system (WASM)
- [ ] Performance optimization

### Phase 4: Polish & Release
- [ ] Cross-platform testing
- [ ] Documentation
- [ ] Binary releases
- [ ] Community feedback

---

## Technical Decisions

### Why Tauri over Electron?
| Aspect | Tauri | Electron |
|--------|-------|----------|
| Binary Size | ~10MB | ~150MB |
| Memory | ~30-50MB | ~150-300MB |
| Startup | ~200ms | ~1 second |
| Native Integration | Native APIs | Chromium bridge |

### Why Rust for Core Engine?
- Memory safety without GC
- Performance critical for diff algorithms
- Easy FFI bindings to other languages
- Growing ecosystem for file operations

### Why Not Just Port WinMerge?
- WinMerge is Windows-focused (MFC framework)
- Starting fresh allows modern architecture
- Opportunity for better UX/DX
- Smaller, more maintainable codebase

---

## Naming Alternatives (Rejected)

### With "Merge"
- SmartMerge ✓ (CHOSEN)
- MergeHub
- MergeFlow
- MergePoint

### With "Diff"
- DiffMeld
- SyncDelta
- FileAlign

### Modern/Abstract
- Beacon
- Nexus
- Convergence

---

## Next Steps

1. **Project Setup**
   - Create GitHub repository
   - Initialize Rust project with Tauri
   - Set up Node.js/React build pipeline
   - Add CI/CD (GitHub Actions)

2. **Core Development**
   - Implement basic diff algorithm
   - Create React UI components
   - Integrate Rust backend with Tauri

3. **Testing**
   - Cross-platform testing (Windows, macOS, Linux)
   - Performance benchmarks
   - User feedback collection

4. **Release**
   - Build platform-specific binaries
   - Create installer/DMG/AppImage
   - Publish to releases page

---

## Useful Resources

- [Tauri Documentation](https://tauri.app)
- [Rust Diff Libraries](https://crates.io/search?q=diff)
- [Tree-sitter for Syntax Highlighting](https://tree-sitter.github.io)
- [WinMerge Source Code](https://github.com/WinMerge/winmerge)
- [Git Diff Algorithm Reference](https://github.com/git/git/blob/master/xdiff/)

---

## Questions for Next Session

- [ ] Should we start with 2-way or 3-way merge?
- [ ] Any specific file types to prioritize? (source code, config, binary?)
- [ ] Should we include folder synchronization initially?
- [ ] Version control integration (Git, SVN) required?
- [ ] Plugin architecture priority level?
- [ ] Performance targets (file size limits, speed benchmarks)?

---

## Contact / Notes
Created during VS Code session on 2026-02-09. Ready to resume development in any workspace.
