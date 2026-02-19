# Function Categorization Prompt

Use this prompt with a **haiku** subagent for cost-effective categorization.

## Prompt Template

```
Read the function catalog at <CATALOG_PATH> and categorize each function.

Assign each function to exactly ONE category based on its primary purpose.

## Categories

- **file-ops**: Reading, writing, path manipulation, directory operations
- **string-utils**: Formatting, parsing, sanitization, case conversion, truncation
- **validation**: Input checking, schema validation, type guards, assertions
- **error-handling**: Error creation, wrapping, formatting, logging helpers
- **http-api**: Request building, response parsing, URL construction, headers
- **date-time**: Date formatting, parsing, comparison, timezone handling
- **data-transform**: Mapping, filtering, normalization, serialization
- **database**: Query building, connection management, migrations
- **logging**: Log formatting, debug helpers, telemetry
- **config**: Configuration loading, environment variables, settings
- **async-utils**: Promise helpers, retry logic, debounce, throttle
- **testing**: Test utilities, mocks, fixtures, assertions
- **ui-helpers**: DOM manipulation, event handling, component utilities
- **crypto**: Hashing, encryption, token generation
- **provider-impl**: AI provider interface implementations (createResponse, etc.)
- **tool-impl**: Tool interface implementations (executeValidated, etc.)
- **event-handling**: Event creation, emission, processing, subscription
- **session-management**: Session/thread/conversation lifecycle
- **compaction**: Message compaction, summarization, token management
- **other**: Doesn't fit above categories (note subcategory in purpose)

## Output Format

For each function, output:
{"file": "...", "name": "...", "line": N, "category": "...", "purpose": "one sentence"}

## Guidelines

1. Focus on WHAT the function does, not HOW it's implemented
2. If a function could fit multiple categories, choose the primary purpose
3. Constructors: categorize based on what the class does
4. Interface implementations: use provider-impl or tool-impl as appropriate
5. Keep purpose descriptions concise but specific

## IMPORTANT

Use the Write tool to save the complete JSON array to <OUTPUT_PATH>.
Do NOT truncate or summarize - write ALL entries.
```

## Usage

1. Run extraction: `./extract-functions.sh src/ -o catalog.json`
2. Dispatch haiku subagent with the prompt above, replacing:
   - `<CATALOG_PATH>` with path to catalog.json
   - `<OUTPUT_PATH>` with desired output path (e.g., `categorized.json`)
3. Verify output file was created with all entries

**Critical:** The subagent must use the Write tool to save output. If it only returns a summary, re-prompt with explicit file write instructions.
