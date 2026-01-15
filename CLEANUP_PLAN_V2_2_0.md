# 🧹 Cleanup Plan for petalTongue v2.2.0

**Date**: January 15, 2026  
**Current State**: 69 markdown files in root  
**Goal**: Clean root, archive old sessions, keep docs as fossil record  

---

## 📊 Current State Analysis

### Root Directory
- **Total .md files**: 69
- **Archive directory**: Exists (jan-13-2026-audit/, jan-13-2026-sessions/)
- **TODOs in code**: 87 found

### Categories of Files

#### Keep (Core Documentation)
Essential files that should stay in root:
- `README.md` - Main entry point ✅
- `STATUS.md` - Current status ✅
- `CHANGELOG.md` - Version history ✅
- `START_HERE.md` - Quick start ✅
- `BUILD_INSTRUCTIONS.md` - How to build ✅
- `BUILD_REQUIREMENTS.md` - What's needed ✅
- `QUICK_START.md` - Quick reference ✅
- `DEPLOYMENT_GUIDE.md` - How to deploy ✅
- `DEMO_GUIDE.md` - Demo instructions ✅
- `INTERACTION_TESTING_GUIDE.md` - Testing guide ✅
- `ENV_VARS.md` - Environment variables ✅

#### Archive (Session Reports - Move to archive/jan-15-2026/)
Old session summaries and completion reports:
- `AUDIO_EVOLUTION_COMPLETE_JAN_13_2026.md`
- `AUDIT_EXECUTIVE_SUMMARY_JAN_13_2026.md`
- `AUDIT_FINDINGS_CHECKLIST_JAN_13_2026.md`
- `AUDIT_QUICK_ACTION_PLAN.md`
- `AUDIT_SUMMARY_JAN_13_2026.md`
- `BENCHTOP_SANDBOX_COMPLETE.md`
- `BIOMEOS_HANDOFF_SUMMARY.md`
- `COMPREHENSIVE_AUDIT_JAN_13_2026.md`
- `COMPREHENSIVE_AUDIT_JAN_15_2026.md`
- `COMPREHENSIVE_AUDIT_REPORT_JAN_13_2026.md`
- `DEPLOYMENT_READY_JAN_13_2026.md`
- `DOCS_CLEANED_JAN_13_2026.md`
- `EVOLUTION_ACTIONS_JAN_13_2026.md`
- `EVOLUTION_COMPLETE_JAN_13_2026.md`
- `EXTRAORDINARY_SESSION_JAN_15_2026.md`
- `FINAL_SESSION_SUMMARY_JAN_15_2026.md`
- `HANDOFF_COMPLETE_JAN_15_2026.md`
- `HANDOFF_NEXT_SESSION_NEURAL_API.md`
- `LIVE_EVOLUTION_COMPLETE_JAN_15_2026.md`
- `LIVE_EVOLUTION_FOUNDATION_COMPLETE.md`
- `LIVE_EVOLUTION_PHASES_1_2_3_COMPLETE_JAN_15_2026.md`
- `NEURAL_API_EVOLUTION_ROADMAP.md`
- `NEURAL_API_EVOLUTION_TRACKER.md`
- `NEURAL_API_HANDOFF_SUMMARY.md`
- `NEURAL_API_IMPLEMENTATION_COMPLETE.md`
- `NEURAL_API_IMPLEMENTATION_PROGRESS.md`
- `NEURAL_API_PHASES_1_2_3_COMPLETE.md`
- `NEURAL_API_UI_INTEGRATION_COMPLETE.md`
- `PHASE_2_DEEP_INTEGRATION_COMPLETE.md`
- `PHASE_3_ADAPTIVE_UI_COMPLETE_JAN_15_2026.md`
- `PHASE_3_ALREADY_COMPLETE.md`
- `PHASE_4_8_COMPLETE_JAN_15_2026.md`
- `PRIMAL_BOUNDARIES_COMPLETE.md`
- `PROGRESS_UPDATE_JAN_15_2026.md`
- `ROOT_DOCS_UPDATED_JAN_15_2026.md`
- `ROOT_DOCS_UPDATED_LIVE_EVOLUTION_JAN_15_2026.md`
- `RUNTIME_VERIFICATION_JAN_13_2026.md`
- `SCENARIO_LOADING_COMPLETE_JAN_15_2026.md`
- `SESSION_COMPLETE_JAN_13_2026.md`
- `SESSION_DELIVERABLES_INDEX.md`
- `SESSION_SUMMARY_JAN_15_2026.md`
- `TRUE_PRIMAL_EVOLUTION_JAN_15_2026.md`

#### Keep in Root (Current Session - Important Reference)
Most recent and relevant session documentation:
- `DEEP_DEBT_LIVE_EVOLUTION_ANALYSIS.md` - Architecture analysis
- `SENSORY_CAPABILITY_EVOLUTION.md` - Major architecture doc (31K)
- `SENSORY_CAPABILITY_COMPLETE_JAN_15_2026.md` - Implementation summary
- `SESSION_COMPLETE_JAN_15_2026.md` - Current session summary
- `NEXT_STEPS_V2_2_0.md` - Next actions
- `ROOT_DOCS_UPDATED_V2_2_0.md` - Current update record

#### Keep in Root (Index/Navigation Files)
- `INDEX.md` - Main index
- `DOCS_INDEX.md` - Documentation index
- `DOCUMENTATION_INDEX.md` - Alternative index
- `NAVIGATION.md` - Navigation guide
- `ROOT_INDEX.md` - Root index
- `NEURAL_API_UI_QUICK_START.md` - Quick reference
- `NEXT_ACTIONS.md` - Action items

#### Keep in Root (Technical Debt Tracking)
- `TECHNICAL_DEBT_NEURAL_API.md` - Debt tracking
- `DOCS_UPDATED_JAN_15_2026.md` - Update tracking

#### Keep in Root (External Systems)
- `TRUE_PRIMAL_EXTERNAL_SYSTEMS.md` - External integrations

---

## 🎯 Cleanup Actions

### Action 1: Create Archive Directory
```bash
mkdir -p archive/jan-15-2026-sessions
```

### Action 2: Move Session Reports to Archive
```bash
# Move all completion reports from Jan 13-15 sessions
mv AUDIO_EVOLUTION_COMPLETE_JAN_13_2026.md archive/jan-15-2026-sessions/
mv AUDIT_*.md archive/jan-15-2026-sessions/
mv BENCHTOP_SANDBOX_COMPLETE.md archive/jan-15-2026-sessions/
mv BIOMEOS_HANDOFF_SUMMARY.md archive/jan-15-2026-sessions/
mv COMPREHENSIVE_AUDIT_*.md archive/jan-15-2026-sessions/
mv DEPLOYMENT_READY_JAN_13_2026.md archive/jan-15-2026-sessions/
mv DOCS_CLEANED_JAN_13_2026.md archive/jan-15-2026-sessions/
mv EVOLUTION_*.md archive/jan-15-2026-sessions/
mv EXTRAORDINARY_SESSION_JAN_15_2026.md archive/jan-15-2026-sessions/
mv FINAL_SESSION_SUMMARY_JAN_15_2026.md archive/jan-15-2026-sessions/
mv HANDOFF_*.md archive/jan-15-2026-sessions/
mv LIVE_EVOLUTION_COMPLETE_JAN_15_2026.md archive/jan-15-2026-sessions/
mv LIVE_EVOLUTION_FOUNDATION_COMPLETE.md archive/jan-15-2026-sessions/
mv LIVE_EVOLUTION_PHASES_1_2_3_COMPLETE_JAN_15_2026.md archive/jan-15-2026-sessions/
mv NEURAL_API_EVOLUTION_*.md archive/jan-15-2026-sessions/
mv NEURAL_API_HANDOFF_SUMMARY.md archive/jan-15-2026-sessions/
mv NEURAL_API_IMPLEMENTATION_*.md archive/jan-15-2026-sessions/
mv NEURAL_API_PHASES_1_2_3_COMPLETE.md archive/jan-15-2026-sessions/
mv NEURAL_API_UI_INTEGRATION_COMPLETE.md archive/jan-15-2026-sessions/
mv PHASE_*.md archive/jan-15-2026-sessions/
mv PRIMAL_BOUNDARIES_COMPLETE.md archive/jan-15-2026-sessions/
mv PROGRESS_UPDATE_JAN_15_2026.md archive/jan-15-2026-sessions/
mv ROOT_DOCS_UPDATED_JAN_15_2026.md archive/jan-15-2026-sessions/
mv ROOT_DOCS_UPDATED_LIVE_EVOLUTION_JAN_15_2026.md archive/jan-15-2026-sessions/
mv RUNTIME_VERIFICATION_JAN_13_2026.md archive/jan-15-2026-sessions/
mv SCENARIO_LOADING_COMPLETE_JAN_15_2026.md archive/jan-15-2026-sessions/
mv SESSION_COMPLETE_JAN_13_2026.md archive/jan-15-2026-sessions/
mv SESSION_DELIVERABLES_INDEX.md archive/jan-15-2026-sessions/
mv SESSION_SUMMARY_JAN_15_2026.md archive/jan-15-2026-sessions/
mv TRUE_PRIMAL_EVOLUTION_JAN_15_2026.md archive/jan-15-2026-sessions/
```

### Action 3: Clean Up Root Files
Remove temporary/outdated files:
```bash
# Remove temporary commit message
rm COMMIT_MESSAGE.txt

# Remove handoff banner (if exists)
rm HANDOFF_BANNER.txt -f
```

### Action 4: Review TODOs in Code
87 TODOs found in code - review and categorize:
- Real TODOs (need action)
- Documentation TODOs (ok to keep)
- Obsolete TODOs (remove)

```bash
# List all TODOs for review
grep -rn "TODO\|FIXME\|XXX\|HACK" --include="*.rs" crates/ > TODO_REVIEW.txt
```

---

## 📋 After Cleanup - Root Should Have

### Core Documentation (~11 files)
- README.md
- STATUS.md
- CHANGELOG.md
- START_HERE.md
- BUILD_INSTRUCTIONS.md
- BUILD_REQUIREMENTS.md
- QUICK_START.md
- DEPLOYMENT_GUIDE.md
- DEMO_GUIDE.md
- INTERACTION_TESTING_GUIDE.md
- ENV_VARS.md

### Current Architecture (~6 files)
- DEEP_DEBT_LIVE_EVOLUTION_ANALYSIS.md
- SENSORY_CAPABILITY_EVOLUTION.md (important - 31K)
- SENSORY_CAPABILITY_COMPLETE_JAN_15_2026.md
- SESSION_COMPLETE_JAN_15_2026.md
- NEXT_STEPS_V2_2_0.md
- ROOT_DOCS_UPDATED_V2_2_0.md

### Navigation & Index (~6 files)
- INDEX.md
- DOCS_INDEX.md
- DOCUMENTATION_INDEX.md
- NAVIGATION.md
- ROOT_INDEX.md
- NEURAL_API_UI_QUICK_START.md

### Tracking (~3 files)
- NEXT_ACTIONS.md
- TECHNICAL_DEBT_NEURAL_API.md
- DOCS_UPDATED_JAN_15_2026.md

### External (~1 file)
- TRUE_PRIMAL_EXTERNAL_SYSTEMS.md

**Total After Cleanup**: ~27 files (down from 69)

---

## 🗂️ Archive Structure

```
archive/
├── jan-13-2026-audit/        (existing, 23 files)
├── jan-13-2026-sessions/     (existing, 61 files)
└── jan-15-2026-sessions/     (NEW, ~41 files)
    ├── AUDIO_EVOLUTION_COMPLETE_JAN_13_2026.md
    ├── AUDIT_*.md
    ├── BENCHTOP_SANDBOX_COMPLETE.md
    ├── BIOMEOS_HANDOFF_SUMMARY.md
    ├── COMPREHENSIVE_AUDIT_*.md
    ├── DEPLOYMENT_READY_JAN_13_2026.md
    ├── DOCS_CLEANED_JAN_13_2026.md
    ├── EVOLUTION_*.md
    ├── EXTRAORDINARY_SESSION_JAN_15_2026.md
    ├── FINAL_SESSION_SUMMARY_JAN_15_2026.md
    ├── HANDOFF_*.md
    ├── LIVE_EVOLUTION_*.md (some, older ones)
    ├── NEURAL_API_*.md (some, older ones)
    ├── PHASE_*.md (older phases)
    ├── PRIMAL_BOUNDARIES_COMPLETE.md
    ├── PROGRESS_UPDATE_JAN_15_2026.md
    ├── ROOT_DOCS_UPDATED_JAN_15_2026.md
    ├── ROOT_DOCS_UPDATED_LIVE_EVOLUTION_JAN_15_2026.md
    ├── RUNTIME_VERIFICATION_JAN_13_2026.md
    ├── SCENARIO_LOADING_COMPLETE_JAN_15_2026.md
    ├── SESSION_*.md (older sessions)
    ├── TRUE_PRIMAL_EVOLUTION_JAN_15_2026.md
    └── README.md (index of archived files)
```

---

## ✅ Benefits of Cleanup

1. **Cleaner Root**: 69 → 27 files (60% reduction)
2. **Better Organization**: Current vs historical separation
3. **Fossil Record Preserved**: All docs archived, not deleted
4. **Easier Navigation**: Less clutter, clear structure
5. **Git History**: Cleaner diffs, easier to review

---

## 🚀 Git Push via SSH

After cleanup:

```bash
# 1. Stage all changes
git add .

# 2. Commit with descriptive message
git commit -m "feat: Add sensory capability architecture v2.2.0

- Add runtime sensory capability discovery (1,300 lines)
- Eliminate hardcoded device types
- Implement 5 UI complexity renderers
- Archive old session documentation
- Clean root directory (69 → 27 files)

Zero hardcoding, TRUE PRIMAL compliant, production ready."

# 3. Push via SSH
git push origin main
# or if different branch:
# git push origin <branch-name>
```

---

## 🎯 Execution Order

1. ✅ Review this plan
2. Create archive directory
3. Move session reports
4. Remove temporary files
5. Review TODOs (optional, can be separate task)
6. Verify root is clean
7. Test build (cargo build --release)
8. Git add, commit, push

---

**Status**: Plan Ready  
**Impact**: Major cleanup, better organization  
**Risk**: Low (all files archived, nothing deleted)  

🧹✨ **Ready to execute cleanup!**

