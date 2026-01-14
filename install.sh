#!/usr/bin/env bash
set -euo pipefail

# =============================================================================
# stacc - AI Agent Configuration Installer
# =============================================================================
# Modern, interactive installer for Cursor IDE and Claude Code configurations
# Usage: curl -fsSL https://stacc.ayush.contact/install.sh | bash
# =============================================================================

VERSION="1.0.0"
REPO_URL="https://github.com/heyAyushh/stacc"
DEFAULT_BRANCH="main"
BRANCH="${STACC_BRANCH:-$DEFAULT_BRANCH}"
RAW_BASE="https://raw.githubusercontent.com/heyAyushh/stacc/${BRANCH}"

# -----------------------------------------------------------------------------
# Colors and formatting (with terminal detection)
# -----------------------------------------------------------------------------
INTERACTIVE=false
if [[ -t 1 ]] && [[ -t 0 ]]; then
    INTERACTIVE=true
    RED='\033[0;31m'
    GREEN='\033[0;32m'
    YELLOW='\033[0;33m'
    BLUE='\033[0;34m'
    MAGENTA='\033[0;35m'
    CYAN='\033[0;36m'
    WHITE='\033[0;37m'
    BOLD='\033[1m'
    DIM='\033[2m'
    RESET='\033[0m'
    REVERSE='\033[7m'
    CHECKMARK='✓'
    ARROW='❯'
    BULLET='•'
    STAR='★'
    BOX_EMPTY='○'
    BOX_FILLED='●'
else
    RED=''
    GREEN=''
    YELLOW=''
    BLUE=''
    MAGENTA=''
    CYAN=''
    WHITE=''
    BOLD=''
    DIM=''
    RESET=''
    REVERSE=''
    CHECKMARK='[OK]'
    ARROW='->'
    BULLET='*'
    STAR='*'
    BOX_EMPTY='[ ]'
    BOX_FILLED='[x]'
fi

# -----------------------------------------------------------------------------
# Logging functions
# -----------------------------------------------------------------------------
info() {
    echo -e "${BLUE}${BULLET}${RESET} $1"
}

success() {
    echo -e "${GREEN}${CHECKMARK}${RESET} $1"
}

warn() {
    echo -e "${YELLOW}!${RESET} $1"
}

error() {
    echo -e "${RED}✗${RESET} $1" >&2
}

step() {
    echo -e "\n${CYAN}${ARROW}${RESET} ${BOLD}$1${RESET}"
}

# -----------------------------------------------------------------------------
# Banner
# -----------------------------------------------------------------------------
print_banner() {
    echo -e "${CYAN}"
    cat << 'EOF'
     _                   
 ___| |_ __ _  ___ ___ 
/ __| __/ _` |/ __/ __|
\__ \ || (_| | (_| (__ 
|___/\__\__,_|\___\___|
                       
EOF
    echo -e "${RESET}"
    echo -e "${DIM}AI Agent Configurations for Cursor & Claude Code${RESET}"
    echo -e "${DIM}Version ${VERSION} • Branch: ${BRANCH}${RESET}"
    echo ""
}

# -----------------------------------------------------------------------------
# Dependency checks
# -----------------------------------------------------------------------------
check_dependencies() {
    step "Checking dependencies"
    
    local missing=()
    
    if ! command -v curl &> /dev/null; then
        missing+=("curl")
    fi
    
    if ! command -v git &> /dev/null; then
        warn "git not found - will use curl fallback for downloads"
        HAS_GIT=false
    else
        HAS_GIT=true
        success "git is available"
    fi
    
    if [[ ${#missing[@]} -gt 0 ]]; then
        error "Missing required dependencies: ${missing[*]}"
        echo -e "Please install them and try again."
        exit 1
    fi
    
    success "All required dependencies found"
}

# -----------------------------------------------------------------------------
# Arrow key navigation - Single select
# -----------------------------------------------------------------------------
arrow_select() {
    local title="$1"
    shift
    local options=("$@")
    local selected=0
    local key=""
    
    # Hide cursor
    tput civis 2>/dev/null || true
    
    # Ensure cursor is shown on exit
    trap 'tput cnorm 2>/dev/null || true' RETURN
    
    echo -e "\n${CYAN}?${RESET} ${BOLD}${title}${RESET}"
    echo -e "${DIM}  Use arrow keys to navigate, Enter to select${RESET}"
    
    while true; do
        # Print options
        for i in "${!options[@]}"; do
            if [[ $i -eq $selected ]]; then
                echo -e "  ${CYAN}${ARROW}${RESET} ${REVERSE} ${options[$i]} ${RESET}"
            else
                echo -e "    ${options[$i]}"
            fi
        done
        
        # Read single keypress
        IFS= read -rsn1 key
        
        # Handle arrow keys (escape sequences)
        if [[ "$key" == $'\x1b' ]]; then
            read -rsn2 key 2>/dev/null || true
            case "$key" in
                '[A') # Up arrow
                    ((selected > 0)) && ((selected--))
                    ;;
                '[B') # Down arrow
                    ((selected < ${#options[@]} - 1)) && ((selected++))
                    ;;
            esac
        elif [[ "$key" == "" ]]; then
            # Enter pressed
            break
        elif [[ "$key" == "j" ]]; then
            ((selected < ${#options[@]} - 1)) && ((selected++))
        elif [[ "$key" == "k" ]]; then
            ((selected > 0)) && ((selected--))
        fi
        
        # Move cursor up to redraw
        for _ in "${options[@]}"; do
            tput cuu1 2>/dev/null || echo -ne "\033[1A"
        done
    done
    
    # Show cursor
    tput cnorm 2>/dev/null || true
    
    SELECTED_INDEX=$selected
    SELECTED_VALUE="${options[$selected]}"
    success "Selected: ${SELECTED_VALUE}"
}

# -----------------------------------------------------------------------------
# Arrow key navigation - Multi select with checkboxes
# -----------------------------------------------------------------------------
arrow_multi_select() {
    local title="$1"
    shift
    local options=("$@")
    local selected=0
    local key=""
    local -a checked=()
    
    # Initialize all as unchecked
    for i in "${!options[@]}"; do
        checked[$i]=0
    done
    
    # Hide cursor
    tput civis 2>/dev/null || true
    
    # Ensure cursor is shown on exit
    trap 'tput cnorm 2>/dev/null || true' RETURN
    
    echo -e "\n${CYAN}?${RESET} ${BOLD}${title}${RESET}"
    echo -e "${DIM}  ↑/↓ navigate, Space toggle, a=all, Enter confirm${RESET}"
    
    while true; do
        # Print options
        for i in "${!options[@]}"; do
            local checkbox
            if [[ ${checked[$i]} -eq 1 ]]; then
                checkbox="${GREEN}${BOX_FILLED}${RESET}"
            else
                checkbox="${DIM}${BOX_EMPTY}${RESET}"
            fi
            
            if [[ $i -eq $selected ]]; then
                echo -e "  ${CYAN}${ARROW}${RESET} ${checkbox} ${REVERSE} ${options[$i]} ${RESET}"
            else
                echo -e "    ${checkbox} ${options[$i]}"
            fi
        done
        
        # Read single keypress
        IFS= read -rsn1 key
        
        # Handle arrow keys (escape sequences)
        if [[ "$key" == $'\x1b' ]]; then
            read -rsn2 key 2>/dev/null || true
            case "$key" in
                '[A') # Up arrow
                    ((selected > 0)) && ((selected--))
                    ;;
                '[B') # Down arrow
                    ((selected < ${#options[@]} - 1)) && ((selected++))
                    ;;
            esac
        elif [[ "$key" == "" ]]; then
            # Enter pressed - finish selection
            break
        elif [[ "$key" == " " ]]; then
            # Space - toggle current item
            if [[ ${checked[$selected]} -eq 1 ]]; then
                checked[$selected]=0
            else
                checked[$selected]=1
            fi
        elif [[ "$key" == "a" ]] || [[ "$key" == "A" ]]; then
            # Toggle all
            local all_checked=1
            for i in "${!options[@]}"; do
                if [[ ${checked[$i]} -eq 0 ]]; then
                    all_checked=0
                    break
                fi
            done
            for i in "${!options[@]}"; do
                if [[ $all_checked -eq 1 ]]; then
                    checked[$i]=0
                else
                    checked[$i]=1
                fi
            done
        elif [[ "$key" == "j" ]]; then
            ((selected < ${#options[@]} - 1)) && ((selected++))
        elif [[ "$key" == "k" ]]; then
            ((selected > 0)) && ((selected--))
        fi
        
        # Move cursor up to redraw
        for _ in "${options[@]}"; do
            tput cuu1 2>/dev/null || echo -ne "\033[1A"
        done
    done
    
    # Show cursor
    tput cnorm 2>/dev/null || true
    
    # Build selected indices array
    SELECTED_INDICES=()
    local selected_names=()
    for i in "${!options[@]}"; do
        if [[ ${checked[$i]} -eq 1 ]]; then
            SELECTED_INDICES+=("$i")
            selected_names+=("${options[$i]}")
        fi
    done
    
    if [[ ${#SELECTED_INDICES[@]} -eq 0 ]]; then
        warn "No items selected, selecting all by default"
        for i in "${!options[@]}"; do
            SELECTED_INDICES+=("$i")
        done
        success "Selected: All options"
    else
        success "Selected: ${selected_names[*]}"
    fi
}

# -----------------------------------------------------------------------------
# Fallback selection menus (for non-interactive mode)
# -----------------------------------------------------------------------------
simple_select() {
    local title="$1"
    shift
    local options=("$@")
    
    echo -e "\n${CYAN}?${RESET} ${BOLD}${title}${RESET}"
    for i in "${!options[@]}"; do
        echo -e "  ${DIM}$((i+1)))${RESET} ${options[$i]}"
    done
    
    while true; do
        echo -ne "${CYAN}${ARROW}${RESET} Enter choice [1-${#options[@]}]: "
        read -r choice
        
        if [[ "$choice" =~ ^[0-9]+$ ]] && [ "$choice" -ge 1 ] && [ "$choice" -le "${#options[@]}" ]; then
            SELECTED_INDEX=$((choice-1))
            SELECTED_VALUE="${options[$SELECTED_INDEX]}"
            success "Selected: ${SELECTED_VALUE}"
            return 0
        else
            warn "Please enter a number between 1 and ${#options[@]}"
        fi
    done
}

multi_select() {
    local title="$1"
    shift
    local options=("$@")
    
    echo -e "\n${CYAN}?${RESET} ${BOLD}${title}${RESET}"
    echo -e "${DIM}  (Enter numbers separated by spaces, or 'a' for all)${RESET}"
    for i in "${!options[@]}"; do
        echo -e "  ${DIM}$((i+1)))${RESET} ${options[$i]}"
    done
    
    while true; do
        echo -ne "${CYAN}${ARROW}${RESET} Enter choices: "
        read -r choices
        
        SELECTED_INDICES=()
        
        if [[ "$choices" == "a" ]] || [[ "$choices" == "A" ]] || [[ "$choices" == "all" ]]; then
            for i in "${!options[@]}"; do
                SELECTED_INDICES+=("$i")
            done
            success "Selected: All options"
            return 0
        fi
        
        local valid=true
        for choice in $choices; do
            if [[ "$choice" =~ ^[0-9]+$ ]] && [ "$choice" -ge 1 ] && [ "$choice" -le "${#options[@]}" ]; then
                SELECTED_INDICES+=($((choice-1)))
            else
                valid=false
                break
            fi
        done
        
        if $valid && [ ${#SELECTED_INDICES[@]} -gt 0 ]; then
            local selected_names=()
            for idx in "${SELECTED_INDICES[@]}"; do
                selected_names+=("${options[$idx]}")
            done
            success "Selected: ${selected_names[*]}"
            return 0
        else
            warn "Please enter valid numbers between 1 and ${#options[@]}, or 'a' for all"
        fi
    done
}

# Wrapper functions to choose interactive vs fallback
select_one() {
    if $INTERACTIVE; then
        arrow_select "$@"
    else
        simple_select "$@"
    fi
}

select_many() {
    if $INTERACTIVE; then
        arrow_multi_select "$@"
    else
        multi_select "$@"
    fi
}

# -----------------------------------------------------------------------------
# Download functions
# -----------------------------------------------------------------------------
clone_or_download() {
    step "Downloading stacc configurations"
    
    TEMP_DIR=$(mktemp -d)
    trap 'rm -rf "$TEMP_DIR"' EXIT
    
    if $HAS_GIT; then
        info "Cloning repository from ${REPO_URL} (branch: ${BRANCH})..."
        if git clone --depth 1 --single-branch --branch "$BRANCH" "$REPO_URL" "$TEMP_DIR/stacc"; then
            success "Repository cloned successfully"
            SOURCE_DIR="$TEMP_DIR/stacc/configs"
            return 0
        else
            warn "Git clone failed (exit code: $?), falling back to curl..."
        fi
    fi
    
    # Fallback: download via curl
    info "Downloading via curl from ${RAW_BASE}..."
    SOURCE_DIR="$TEMP_DIR/configs"
    mkdir -p "$SOURCE_DIR"
    
    # Download manifest or known files
    download_category_files
    
    success "Download completed"
}

download_category_files() {
    local categories=("commands" "rules" "agents" "skills" "stack" "mcps" "hooks")
    
    for category in "${categories[@]}"; do
        mkdir -p "$SOURCE_DIR/$category"
    done
    
    # Download commands
    local commands=("commit.md" "commit-push.md" "commit-push-pr.md" "deslop.md" 
                    "explore.md" "init.md" "onboard-new-developer.md" "refactor.md" 
                    "review.md" "ultrathink.md" "visualize.md")
    for file in "${commands[@]}"; do
        curl -fsSL "${RAW_BASE}/configs/commands/${file}" -o "$SOURCE_DIR/commands/${file}" 2>/dev/null || true
    done
    
    # Download mcps
    curl -fsSL "${RAW_BASE}/configs/mcps/mcp.json" -o "$SOURCE_DIR/mcps/mcp.json" 2>/dev/null || true
    
    # Download hooks
    curl -fsSL "${RAW_BASE}/configs/hooks/hook.md" -o "$SOURCE_DIR/hooks/hook.md" 2>/dev/null || true
    
    # Download rules
    local rules=("clean-code.mdc" "commit-message-format.mdc" "pr-message-format.mdc" "prompt-injection-gaurd.mdc")
    for file in "${rules[@]}"; do
        curl -fsSL "${RAW_BASE}/configs/rules/${file}" -o "$SOURCE_DIR/rules/${file}" 2>/dev/null || true
    done
    
    # Download agents
    local agents=("askuserquestion.md" "verifier.md")
    for file in "${agents[@]}"; do
        curl -fsSL "${RAW_BASE}/configs/agents/${file}" -o "$SOURCE_DIR/agents/${file}" 2>/dev/null || true
    done
    
    # Download stack rules
    local stack_files=("bun.mdc" "next-js.mdc" "postgresql.mdc" "rust.mdc" "typescript.mdc")
    for file in "${stack_files[@]}"; do
        curl -fsSL "${RAW_BASE}/configs/stack/${file}" -o "$SOURCE_DIR/stack/${file}" 2>/dev/null || true
    done
    
    # Download skills (just the SKILL.md files for simplicity)
    local skills=("changelog-generator" "frontend-design" "mcp-builder" "skill-creator")
    for skill in "${skills[@]}"; do
        mkdir -p "$SOURCE_DIR/skills/$skill"
        curl -fsSL "${RAW_BASE}/configs/skills/${skill}/SKILL.md" -o "$SOURCE_DIR/skills/${skill}/SKILL.md" 2>/dev/null || true
    done
}

# -----------------------------------------------------------------------------
# Installation functions
# -----------------------------------------------------------------------------
install_category() {
    local category="$1"
    local target_dir="$2"
    local source_path="$SOURCE_DIR/$category"
    
    if [[ ! -d "$source_path" ]]; then
        warn "Category '$category' not found in source"
        return 1
    fi
    
    # Special handling for mcps - handled separately in install_to_target
    if [[ "$category" == "mcps" ]]; then
        return 0
    fi
    
    # Determine target subdirectory
    local target_subdir
    case "$category" in
        commands)
            target_subdir="commands"
            ;;
        rules)
            target_subdir="rules"
            ;;
        agents)
            target_subdir="agents"
            ;;
        skills)
            target_subdir="skills"
            ;;
        hooks)
            target_subdir="hooks"
            ;;
        stack)
            # Stack .mdc files go to rules/
            target_subdir="rules"
            ;;
        *)
            target_subdir="$category"
            ;;
    esac
    
    mkdir -p "${target_dir}/${target_subdir}"
    
    if [[ "$category" == "stack" ]]; then
        # Copy only .mdc files from stack to rules
        find "$source_path" -maxdepth 1 -name "*.mdc" -exec cp {} "${target_dir}/${target_subdir}/" \; 2>/dev/null || true
    else
        cp -r "$source_path"/* "${target_dir}/${target_subdir}/" 2>/dev/null || true
    fi
    
    success "Installed $category to ${target_dir}/${target_subdir}/"
}

get_target_directory() {
    local editor="$1"
    local scope="$2"
    
    case "$editor" in
        "Cursor")
            if [[ "$scope" == "Global" ]]; then
                echo "$HOME/.cursor"
            else
                echo ".cursor"
            fi
            ;;
        "Claude Code")
            if [[ "$scope" == "Global" ]]; then
                echo "$HOME/.claude"
            else
                echo ".claude"
            fi
            ;;
    esac
}

install_to_target() {
    local editor="$1"
    local scope="$2"
    shift 2
    local categories=("$@")
    
    local target_dir
    target_dir=$(get_target_directory "$editor" "$scope")
    
    info "Installing to ${target_dir}/ for ${editor} (${scope})..."
    mkdir -p "$target_dir"
    
    for category in "${categories[@]}"; do
        # Special handling for MCPs
        if [[ "$category" == "mcps" ]]; then
            local mcp_source="$SOURCE_DIR/mcps/mcp.json"
            if [[ -f "$mcp_source" ]]; then
                local mcp_target
                if [[ "$editor" == "Claude Code" ]]; then
                    # Claude Code: .mcp.json inside .claude directory
                    mcp_target="${target_dir}/.mcp.json"
                else
                    # Cursor: mcp.json inside .cursor directory
                    mcp_target="${target_dir}/mcp.json"
                fi
                
                # Copy and verify
                if cp "$mcp_source" "$mcp_target" && [[ -f "$mcp_target" ]]; then
                    success "Installed MCP config to ${mcp_target}"
                else
                    error "Failed to install MCP config to ${mcp_target}"
                fi
            else
                warn "MCP config not found at ${mcp_source}"
            fi
        else
            install_category "$category" "$target_dir"
        fi
    done
}

# -----------------------------------------------------------------------------
# Print usage instructions
# -----------------------------------------------------------------------------
print_success_message() {
    local editors=("$@")
    
    echo ""
    echo -e "${GREEN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${RESET}"
    echo -e "${GREEN}${STAR}${RESET} ${BOLD}Installation Complete!${RESET}"
    echo -e "${GREEN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${RESET}"
    echo ""
    
    echo -e "${BOLD}What's next?${RESET}"
    echo ""
    
    for editor in "${editors[@]}"; do
        if [[ "$editor" == "Cursor" ]]; then
            echo -e "  ${CYAN}Cursor IDE:${RESET}"
            echo -e "    ${BULLET} Commands are available via ${BOLD}Ctrl/Cmd + Shift + P${RESET} ${ARROW} 'Run Command'"
            echo -e "    ${BULLET} Rules are automatically applied to your projects"
            echo -e "    ${BULLET} Skills provide specialized AI capabilities"
            echo -e "    ${BULLET} MCP servers configured in ${BOLD}.cursor/mcp.json${RESET}"
            echo ""
        fi
        
        if [[ "$editor" == "Claude Code" ]]; then
            echo -e "  ${MAGENTA}Claude Code:${RESET}"
            echo -e "    ${BULLET} Commands can be invoked with ${BOLD}/command-name${RESET}"
            echo -e "    ${BULLET} Rules and agents enhance Claude's behavior"
            echo -e "    ${BULLET} Skills are loaded automatically"
            echo -e "    ${BULLET} MCP servers configured in ${BOLD}.claude/.mcp.json${RESET}"
            echo ""
        fi
    done
    
    echo -e "${DIM}For more information, visit: ${REPO_URL}${RESET}"
    echo ""
}

# -----------------------------------------------------------------------------
# Main flow
# -----------------------------------------------------------------------------
main() {
    print_banner
    check_dependencies
    
    # Editor selection
    local editor_options=("Cursor" "Claude Code" "Both")
    select_one "Which editor do you want to configure?" "${editor_options[@]}"
    local selected_editor="$SELECTED_VALUE"
    
    # Scope selection
    local scope_options=("Global (all projects)" "Project (current directory only)")
    select_one "Installation scope?" "${scope_options[@]}"
    local selected_scope
    if [[ "$SELECTED_VALUE" == "Global"* ]]; then
        selected_scope="Global"
    else
        selected_scope="Project"
    fi
    
    # Category selection
    local category_options=("commands" "rules" "agents" "skills" "hooks" "stack (framework configs)" "mcps (MCP servers)")
    select_many "Which categories to install?" "${category_options[@]}"
    
    # Map selected indices to category names
    local categories=()
    for idx in "${SELECTED_INDICES[@]}"; do
        case $idx in
            0) categories+=("commands") ;;
            1) categories+=("rules") ;;
            2) categories+=("agents") ;;
            3) categories+=("skills") ;;
            4) categories+=("hooks") ;;
            5) categories+=("stack") ;;
            6) categories+=("mcps") ;;
        esac
    done
    
    # Download configurations
    clone_or_download
    
    # Install to selected targets
    step "Installing configurations"
    
    local installed_editors=()
    
    if [[ "$selected_editor" == "Cursor" ]] || [[ "$selected_editor" == "Both" ]]; then
        install_to_target "Cursor" "$selected_scope" "${categories[@]}"
        installed_editors+=("Cursor")
    fi
    
    if [[ "$selected_editor" == "Claude Code" ]] || [[ "$selected_editor" == "Both" ]]; then
        install_to_target "Claude Code" "$selected_scope" "${categories[@]}"
        installed_editors+=("Claude Code")
    fi
    
    # Success message
    print_success_message "${installed_editors[@]}"
}

# Run main function
main "$@"
