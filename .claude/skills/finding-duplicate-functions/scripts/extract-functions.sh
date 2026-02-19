#!/usr/bin/env bash
# ABOUTME: Extracts function/method definitions from TypeScript/JavaScript codebase
# Outputs JSON catalog for duplicate detection analysis

set -euo pipefail

usage() {
    cat <<EOF
Usage: $(basename "$0") [OPTIONS] <source-directory>

Extract function catalog from TypeScript/JavaScript codebase.

OPTIONS:
    -o, --output FILE    Output file (default: stdout)
    -c, --context N      Lines of implementation to capture (default: 15)
    -t, --types GLOB     File types to scan (default: "*.ts,*.tsx,*.js,*.jsx")
    --include-tests      Include test files (excluded by default)
    -h, --help           Show this help

Test files excluded by default:
    *.test.*, *.spec.*, __tests__/**, test/**, tests/**

EXAMPLES:
    $(basename "$0") src/
    $(basename "$0") -o catalog.json -c 3 packages/
    $(basename "$0") --types "*.ts" src/
    $(basename "$0") --include-tests src/   # Include test files

OUTPUT FORMAT:
    JSON array of objects with: file, name, signature, context, exportType
EOF
    exit 0
}

# Defaults
OUTPUT="/dev/stdout"
CONTEXT_LINES=15
FILE_TYPES="*.ts,*.tsx,*.js,*.jsx"
INCLUDE_TESTS=false

# Parse args
while [[ $# -gt 0 ]]; do
    case $1 in
        -o|--output) OUTPUT="$2"; shift 2 ;;
        -c|--context) CONTEXT_LINES="$2"; shift 2 ;;
        -t|--types) FILE_TYPES="$2"; shift 2 ;;
        --include-tests) INCLUDE_TESTS=true; shift ;;
        -h|--help) usage ;;
        -*) echo "Unknown option: $1" >&2; exit 1 ;;
        *) SRC_DIR="$1"; shift ;;
    esac
done

if [[ -z "${SRC_DIR:-}" ]]; then
    echo "Error: source directory required" >&2
    usage
fi

if [[ ! -d "$SRC_DIR" ]]; then
    echo "Error: directory not found: $SRC_DIR" >&2
    exit 1
fi

# Build glob pattern for ripgrep (use array to avoid glob expansion)
GLOB_ARGS=()
IFS=',' read -ra TYPES <<< "$FILE_TYPES"
for type in "${TYPES[@]}"; do
    GLOB_ARGS+=(--glob "$type")
done

# Exclude test files by default
if [[ "$INCLUDE_TESTS" == "false" ]]; then
    GLOB_ARGS+=(--glob '!*.test.*' --glob '!*.spec.*')
    GLOB_ARGS+=(--glob '!**/__tests__/**' --glob '!**/test/**' --glob '!**/tests/**')
fi

# Patterns to match function definitions
# Pattern 1: export function name(
# Pattern 2: export const name = (async)? (
# Pattern 3: export const name = (async)? function
# Pattern 4: export default function
# Pattern 5: class methods (public/private/protected/async)
# Pattern 6: standalone function declarations

extract_functions() {
    local dir="$1"
    local ctx="$2"

    # Use ripgrep to find function definitions with context
    # Note: Class method pattern requires visibility/async/static to avoid matching if/for/while
    rg --json \
        -e '^export (async )?function \w+' \
        -e '^export const \w+ = (async )?\(' \
        -e '^export const \w+ = (async )?function' \
        -e '^export default (async )?function' \
        -e '^  (public |private |protected )(async |static )*(get |set )?\w+\s*\(' \
        -e '^  (async |static )(async |static )*(get |set )?\w+\s*\(' \
        -e '^  (get |set )\w+\s*\(' \
        -e '^  constructor\s*\(' \
        -e '^(async )?function \w+\s*\(' \
        "${GLOB_ARGS[@]}" \
        -A "$ctx" \
        "$dir" 2>/dev/null || true
}

# Process ripgrep JSON output into our catalog format
process_output() {
    jq -s '
        # Group by match (each match has type "begin", "match", "context", "end")
        reduce .[] as $item (
            {current: null, results: []};
            if $item.type == "begin" then
                .current = {file: $item.data.path.text, lines: []}
            elif $item.type == "match" then
                .current.lines += [{
                    line_number: $item.data.line_number,
                    text: $item.data.lines.text,
                    is_match: true
                }]
            elif $item.type == "context" then
                .current.lines += [{
                    line_number: $item.data.line_number,
                    text: $item.data.lines.text,
                    is_match: false
                }]
            elif $item.type == "end" then
                if .current.lines | length > 0 then
                    .results += [.current]
                else . end
            else . end
        ) | .results

        # Transform into catalog entries - group each match with its following context
        | map(
            .file as $file |
            .lines |
            # Find indices of match lines
            to_entries |
            reduce .[] as $entry (
                {matches: [], current_match: null, entries: []};
                if $entry.value.is_match then
                    # Save previous match group if exists
                    (if .current_match then
                        .entries += [{
                            file: $file,
                            line: .current_match.line_number,
                            match_line: .current_match.text,
                            context_lines: .context
                        }]
                    else . end) |
                    # Start new match group
                    .current_match = $entry.value |
                    .context = []
                else
                    # Add to current context if we have a match
                    if .current_match then
                        .context += [$entry.value.text]
                    else . end
                end
            ) |
            # Dont forget last match group
            (if .current_match then
                .entries += [{
                    file: $file,
                    line: .current_match.line_number,
                    match_line: .current_match.text,
                    context_lines: .context
                }]
            else . end) |
            .entries |
            map(. + {context: ((.match_line // "") + ((.context_lines // []) | join("")))})
        ) | flatten

        # Extract function name and classify export type
        | map(
            . + {
                name: (
                    .match_line |
                    capture("(?:export )?(?:async )?(?:function |const )(?<name>\\w+)") //
                    capture("(?:public |private |protected )?(?:async |static )*(?:get |set )?(?<name>\\w+)\\s*\\(") //
                    {name: "unknown"}
                ).name,
                exportType: (
                    if .match_line | test("^export default") then "default"
                    elif .match_line | test("^export ") then "named"
                    elif .match_line | test("^  ") then "method"
                    else "internal"
                    end
                )
            }
        )

        # Filter out keywords, invalid entries, and common loop variables
        | map(select(
            .name != "unknown" and
            .name != "if" and
            .name != "else" and
            .name != "for" and
            .name != "while" and
            .name != "switch" and
            .name != "try" and
            .name != "catch" and
            .name != "return" and
            .name != "throw" and
            .name != "new" and
            .name != "typeof" and
            .name != "await" and
            .name != "const" and
            .name != "let" and
            .name != "var" and
            # Common loop variables that get false-positive matched
            .name != "line" and
            .name != "item" and
            .name != "entry" and
            .name != "element" and
            .name != "key" and
            .name != "value" and
            .name != "i" and
            .name != "j" and
            .name != "k"
        ))

        # Clean up and format output
        | map({
            file: .file,
            name: .name,
            line: .line,
            exportType: .exportType,
            context: (.context | gsub("\\n+$"; ""))
        })
        | sort_by(.file, .line)
    '
}

# Main
extract_functions "$SRC_DIR" "$CONTEXT_LINES" | process_output > "$OUTPUT"

# Report stats to stderr
if [[ "$OUTPUT" != "/dev/stdout" ]]; then
    count=$(jq 'length' "$OUTPUT")
    echo "Extracted $count function definitions to $OUTPUT" >&2
fi
