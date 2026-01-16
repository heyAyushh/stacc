# Install.sh Bug Report

## Critical Security Issues

### 1. Path Traversal Vulnerability (Line 1184-1189)
**Severity: CRITICAL**
**Location:** `install_category()` function

**Issue:** User-supplied category names are not validated, allowing path traversal attacks.

**Attack Vector:**
```bash
./install.sh --categories "../.." --conflict overwrite
```

This could execute:
```bash
rm -rf "${HOME}/.cursor/../.."  # Deletes parent of HOME!
```

**Root Cause:**
```bash
IFS=',' read -r -a cats <<< "${SELECTED_CATEGORIES}"
for category in "${cats[@]}"; do
  install_category "${category}" "${target_root}"  # No validation!
done
```

**Fix:**
```bash
validate_category() {
  local cat="$1"
  case "${cat}" in
    commands|rules|agents|skills|stack|hooks|mcps) return 0 ;;
    *) die "invalid category: ${cat}" ;;
  esac
}

# Then in install_for_target:
for category in "${cats[@]}"; do
  validate_category "${category}"  # Add validation
  if [ "${category}" = "mcps" ]; then
    ...
```

---

### 2. Dry-Run Mode Corrupts MCP Config (Line 1090)
**Severity: HIGH**
**Location:** `merge_mcp()` function

**Issue:** In dry-run mode, the script creates an empty temp file and moves it to the destination, clobbering the existing MCP config.

**Root Cause:**
```bash
run_cmd jq -s '.[0] * .[1]' "${dest}" "${src}" > "${tmp}"
```

The redirect `> "${tmp}"` happens OUTSIDE `run_cmd`, so even in dry-run mode:
- `run_cmd` just prints the command (doesn't run jq)
- Shell creates/truncates `"${tmp}"` (empty file)
- Line 1095: `run_cmd mv "${tmp}" "${dest}"` in dry-run prints the command
- But if it somehow runs, it would move empty file to dest!

**Fix:**
```bash
merge_mcp() {
  local src="$1"
  local dest="$2"

  if command -v jq >/dev/null 2>&1; then
    if [ "${DRY_RUN}" -eq 1 ]; then
      log_verbose "[dry-run] Would merge MCP config ${src} into ${dest}"
      return 0
    fi

    if [ "${NON_INTERACTIVE}" -eq 0 ]; then
      # ... existing prompt logic ...
    fi

    local tmp
    tmp="$(mktemp)"
    # Don't use run_cmd here since redirect is external
    jq -s '.[0] * .[1]' "${dest}" "${src}" > "${tmp}" || {
      rm -f "${tmp}"
      return 1
    }
    if [ -e "${dest}" ]; then
      handle_conflict "${dest}" || { rm -f "${tmp}"; return 0; }
    fi
    log_verbose "Writing merged MCP config to ${dest}"
    mv "${tmp}" "${dest}"  # Direct call, not run_cmd
    return 0
  fi

  return 1
}
```

---

## High Severity Bugs

### 3. Missing TTY Writability Check (Line 689-695)
**Severity: HIGH**
**Location:** `init_tty()` function

**Issue:** Script checks if `/dev/tty` is readable but not writable. If TTY is read-only, all `ui_out()` calls will fail, potentially leaving the terminal in a broken state (no echo, raw mode, hidden cursor).

**Current Code:**
```bash
init_tty() {
  if [ -t 0 ]; then
    TTY_DEVICE="/dev/tty"
  elif [ -r /dev/tty ]; then    # Only checks readable!
    TTY_DEVICE="/dev/tty"
  fi
}
```

**Fix:**
```bash
init_tty() {
  if [ -r /dev/tty ] && [ -w /dev/tty ]; then
    TTY_DEVICE="/dev/tty"
  fi
}
```

---

### 4. Terminal State Not Restored on Error (Line 497-528)
**Severity: HIGH**
**Location:** `menu_single()` and `menu_multi()` functions

**Issue:** If script exits (error, signal, etc.) while in menu, terminal state is not restored. User left with:
- No echo (typed characters invisible)
- Raw mode (no line buffering)
- Hidden cursor

**Current Code:**
```bash
menu_single() {
  stty_state="$(stty -g < "${TTY_DEVICE}")"
  stty -echo -icanon time 0 min 1 < "${TTY_DEVICE}"
  tput civis > "${TTY_DEVICE}" 2>/dev/null || true

  # ... menu logic ...

  # Only restores if we reach the end:
  tput cnorm > "${TTY_DEVICE}" 2>/dev/null || true
  stty "${stty_state}" < "${TTY_DEVICE}"
}
```

**Fix:** Add trap to restore terminal state, but need to handle it carefully since RETURN trap might not be available in bash 3.2. Alternative: add terminal state to global EXIT cleanup:

```bash
SAVED_TTY_STATE=""

cleanup() {
  # Restore terminal if needed
  if [ -n "${SAVED_TTY_STATE}" ] && [ -n "${TTY_DEVICE}" ]; then
    stty "${SAVED_TTY_STATE}" < "${TTY_DEVICE}" 2>/dev/null || true
    tput cnorm > "${TTY_DEVICE}" 2>/dev/null || true
  fi

  # Existing cleanup
  if [ -n "${TMP_ROOT}" ] && [ -d "${TMP_ROOT}" ]; then
    rm -rf "${TMP_ROOT}"
  fi
}

# In menu_single/menu_multi, save state globally:
menu_single() {
  SAVED_TTY_STATE="$(stty -g < "${TTY_DEVICE}")"
  stty -echo -icanon time 0 min 1 < "${TTY_DEVICE}"
  # ... rest of function ...
  tput cnorm > "${TTY_DEVICE}" 2>/dev/null || true
  stty "${SAVED_TTY_STATE}" < "${TTY_DEVICE}"
  SAVED_TTY_STATE=""  # Clear after restore
}
```

---

### 5. Temp File Leak on Error (Line 1089)
**Severity: MEDIUM**
**Location:** `merge_mcp()` function

**Issue:** If script exits after `mktemp` but before cleanup, temp file is leaked.

**Current Code:**
```bash
tmp="$(mktemp)"
run_cmd jq -s '.[0] * .[1]' "${dest}" "${src}" > "${tmp}"
# If die() is called here, tmp is not cleaned
```

**Fix:** Use TMP_ROOT for temp files so they're cleaned by EXIT trap:
```bash
tmp="${TMP_ROOT}/mcp_merge.$$"
```

---

## Medium Severity Bugs

### 6. Whitespace in Categories Not Handled (Line 1184)
**Severity: MEDIUM**
**Location:** `install_for_target()` function

**Issue:** User can pass categories with spaces which won't match expected values.

**Example:**
```bash
./install.sh --categories "commands, rules, mcps"
# Results in categories: "commands", " rules", " mcps"
# " mcps" != "mcps", so wrong branch taken
```

**Fix:**
```bash
--categories)
  [ $# -ge 2 ] || die "--categories requires a list"
  SELECTED_CATEGORIES="$2"
  # Strip all whitespace
  SELECTED_CATEGORIES="${SELECTED_CATEGORIES// /}"
  shift 2
  ;;
```

---

### 7. Empty PROJECT_ROOT Risk (Line 8)
**Severity: MEDIUM**
**Location:** Main initialization

**Issue:** If `pwd` fails (e.g., current directory deleted), `PROJECT_ROOT` is empty, leading to installation in root directory: `/.cursor/`

**Current Code:**
```bash
PROJECT_ROOT="$(pwd)"
```

**Fix:**
```bash
PROJECT_ROOT="$(pwd)" || die "failed to determine current directory"
```

---

### 8. Division by Zero Possibility (Line 351)
**Severity: LOW**
**Location:** `ui_wrap_count()` function

**Issue:** If `cols` is 0, division by zero occurs. Currently safe because `tput cols` falls back to 80, but fragile.

**Current Code:**
```bash
ui_wrap_count() {
  local text="$1"
  local cols="$2"
  local clean
  clean="$(strip_ansi "${text}")"
  local len=${#clean}
  if [ "${len}" -le 0 ]; then
    printf '%s' 1
    return
  fi
  printf '%s' $(( (len - 1) / cols + 1 ))  # Division by zero if cols=0!
}
```

**Fix:**
```bash
ui_wrap_count() {
  local text="$1"
  local cols="$2"
  local clean
  clean="$(strip_ansi "${text}")"
  local len=${#clean}
  if [ "${len}" -le 0 ] || [ "${cols}" -le 0 ]; then
    printf '%s' 1
    return
  fi
  printf '%s' $(( (len - 1) / cols + 1 ))
}
```

---

### 9. Substring Matching Bug in Editor Selection (Line 820-822)
**Severity: LOW (not triggered by current items)**
**Location:** `select_editors()` function

**Issue:** If one editor name is substring of another, selection logic breaks.

**Example:** If editors were "Cursor" and "Cursor Pro":
```bash
case "${MENU_RESULT}" in
  *"Cursor"*) # Matches both "Cursor" and "Cursor Pro"!
esac
```

**Current items are safe:** "Cursor", "Claude Code", "OpenCode", "Codex" have no substring relationships.

**Better approach:** Since `MENU_RESULT` is delimiter-separated, check with delimiters:
```bash
# Instead of: *"${editor_items[$i]}"*
# Check: |item| or ^item| or |item$ or exact match
```

---

### 10. Unquoted Pattern in Parameter Expansion (Line 1052)
**Severity: LOW**
**Location:** `copy_tree()` function

**Issue:** Pattern not quoted in parameter expansion. If `src_dir` contains glob characters like `[`, matching could fail.

**Current Code:**
```bash
local rel="${file#${src_dir}/}"  # src_dir not quoted in pattern
```

**Fix:**
```bash
local rel="${file#"${src_dir}/"}"
```

---

### 11. Inconsistent CONFLICT_MODE Handling (Line 1079-1086)
**Severity: LOW**
**Location:** `merge_mcp()` function

**Issue:** `merge_mcp()` doesn't respect `CONFLICT_MODE` setting. In non-interactive mode, always attempts merge. Should respect skip/backup modes.

**Fix:**
```bash
merge_mcp() {
  local src="$1"
  local dest="$2"

  if command -v jq >/dev/null 2>&1; then
    if [ "${NON_INTERACTIVE}" -eq 0 ]; then
      printf "Merge MCP config into existing %s? [y/N] " "${dest}" > "${TTY_DEVICE}"
      local confirm
      prompt_read confirm
      case "${confirm}" in
        y|Y|yes|YES) ;;
        *) return 1 ;;
      esac
    else
      # In non-interactive mode, respect CONFLICT_MODE
      case "${CONFLICT_MODE}" in
        skip|skip_all) return 1 ;;
        overwrite|overwrite_all) return 1 ;;  # Don't merge, let copy_file overwrite
        backup|backup_all) ;;  # Proceed with merge
        *) ;;
      esac
    fi
    # ... rest of merge logic ...
```

---

## Minor Issues / Code Quality

### 12. Typo in Log Message (Line 711)
**Severity: TRIVIAL**

**Current:**
```bash
log_verbose "Cloning stacc /simsitory..."
```

**Fix:**
```bash
log_verbose "Cloning stacc repository..."
```

---

### 13. Unused Variable (Line 466)
**Severity: TRIVIAL**
**Location:** `render_menu_multi()` function

**Issue:** Creates local copy of `SELECTED_FLAGS` array but never uses it.

**Current Code:**
```bash
local -a selected=("${SELECTED_FLAGS[@]}")  # Never used!
```

**Fix:** Remove the line, or use the local variable consistently:
```bash
# Remove line 466 entirely
```

---

### 14. Unused SCRIPT_DIR Variable (Line 6)
**Severity: TRIVIAL**

**Issue:** `SCRIPT_DIR` is computed but never used.

**Fix:** Either remove it or document it's for future use:
```bash
# SCRIPT_DIR="$(cd "$(dirname "${SCRIPT_REF}")" && pwd)"  # Unused, remove
```

---

### 15. Potential sed Compatibility Issue (Line 324)
**Severity: LOW**
**Location:** `strip_ansi()` function

**Issue:** `\x1b` escape sequence might not work in older BSD sed implementations.

**Current Code:**
```bash
strip_ansi() {
  printf '%s' "$1" | sed 's/\x1b\[[0-9;]*m//g'
}
```

**More portable:**
```bash
strip_ansi() {
  printf '%s' "$1" | sed 's/\x1b\[[0-9;]*m//g' 2>/dev/null || \
    printf '%s' "$1" | sed $'s/\033\\[[0-9;]*m//g'
}
```

---

### 16. Fractional sleep May Not Work (Line 280)
**Severity: TRIVIAL**
**Location:** `show_caption_only()` function

**Issue:** `sleep 0.02` requires sleep implementation supporting fractional seconds. Older systems might not support it.

**Impact:** Only affects animation, not functionality. Error would stop animation but script continues.

---

### 17. Redundant grep in Directory Check (Line 1065)
**Severity: TRIVIAL**
**Location:** `install_category()` function

**Current Code:**
```bash
if [ -d "${dest}" ] && find "${dest}" -mindepth 1 -print -quit | grep -q .; then
```

**Issue:** `grep -q .` is redundant. `find ... -print -quit` already outputs something or nothing.

**Better:**
```bash
if [ -d "${dest}" ] && [ -n "$(find "${dest}" -mindepth 1 -print -quit)" ]; then
```

---

### 18. Empty Timestamp on Date Failure (Line 40)
**Severity: TRIVIAL**
**Location:** `backup_target()` function

**Issue:** If `date` fails, timestamp is empty, creating backup like `file.bak.`

**Current Code:**
```bash
ts="$(date +%Y%m%d%H%M%S)"
backup="${target}.bak.${ts}"
```

**Fix:**
```bash
ts="$(date +%Y%m%d%H%M%S)" || ts="$$"  # Fallback to PID
```

---

## Testing Recommendations

1. **Path traversal tests:**
   ```bash
   ./install.sh --categories "../.." --dry-run
   ./install.sh --categories "../../etc" --dry-run
   ```

2. **Whitespace handling:**
   ```bash
   ./install.sh --categories "commands, rules, mcps"
   ```

3. **Dry-run with MCP:**
   ```bash
   # Create existing mcp.json, then:
   ./install.sh --dry-run --categories mcps
   # Verify existing file not corrupted
   ```

4. **Terminal state:**
   ```bash
   # Kill script during menu (Ctrl+C)
   ./install.sh
   # Select editor menu appears
   # Press Ctrl+C
   # Verify terminal still works (echo, cursor visible)
   ```

5. **Empty directory:**
   ```bash
   mkdir /tmp/deleted && cd /tmp/deleted && rmdir /tmp/deleted
   # Now pwd fails
   ./install.sh
   ```

6. **Read-only TTY:**
   ```bash
   # Difficult to test, but could use: chmod -w /dev/tty
   ```

---

## Summary

**Critical:** 2 bugs (path traversal, dry-run corruption)
**High:** 3 bugs (TTY checks, terminal state, temp leak)
**Medium:** 5 bugs (whitespace, empty pwd, division, matching, conflict mode)
**Low/Trivial:** 8 issues (typos, unused vars, compatibility)

**Most Urgent Fixes:**
1. Validate category names (security)
2. Fix dry-run MCP merge (data loss)
3. Add TTY writability check (UX)
4. Restore terminal state on error (UX)
