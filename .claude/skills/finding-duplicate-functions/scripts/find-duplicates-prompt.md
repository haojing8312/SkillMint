# Duplicate Detection Prompt

Use this prompt with an **opus** subagent for thorough semantic analysis.

Run this prompt **once per category** that has 3+ functions.

## Prompt Template

```
You are analyzing functions in the "{CATEGORY}" category for semantic duplicates.

Semantic duplicates are functions that serve the SAME PURPOSE even if:
- They have different names
- They use different implementations
- They have slightly different signatures
- One is more general than another

## Your Task

1. Compare all functions in this category
2. Identify groups of functions that do the same thing
3. For each duplicate group, assess confidence and recommend action

## Output Format

Return a JSON array of duplicate groups:

```json
[
  {
    "intent": "<what these functions all do>",
    "confidence": "HIGH|MEDIUM|LOW",
    "functions": [
      {
        "file": "<file path>",
        "name": "<function name>",
        "line": <line number>,
        "notes": "<implementation specifics>"
      }
    ],
    "differences": "<how implementations differ, if at all>",
    "recommendation": {
      "action": "CONSOLIDATE|INVESTIGATE|KEEP_SEPARATE",
      "survivor": "<which function to keep, if CONSOLIDATE>",
      "reason": "<why this recommendation>"
    }
  }
]
```

## Confidence Levels

- **HIGH**: Definitely the same thing. Same input→output semantics.
  Example: `formatDate(d)` and `dateToString(d)` both format dates identically

- **MEDIUM**: Likely the same thing with minor differences.
  Example: `validateEmail(s)` uses regex, `checkEmail(s)` uses library, but same purpose

- **LOW**: Possibly related, worth investigating.
  Example: `sanitizeInput(s)` and `escapeHtml(s)` - related but maybe distinct purposes

## Recommendations

- **CONSOLIDATE**: Functions are duplicates. Keep the one with better name/implementation/tests.
- **INVESTIGATE**: Need to read full implementations to decide. Flag for human review.
- **KEEP_SEPARATE**: Functions look similar but serve distinct purposes.

## Guidelines

1. Read the context/implementation snippets carefully
2. Consider edge case handling - two functions might differ in how they handle nulls
3. If functions are in test files, they're less likely to be true duplicates
4. Generic utilities (identity, noop, constant) are often intentionally duplicated
5. When in doubt, recommend INVESTIGATE rather than CONSOLIDATE

## Functions in "{CATEGORY}" Category

<INSERT_CATEGORY_FUNCTIONS_HERE>
```

## Usage

1. First run categorization (see categorize-prompt.md)
2. Filter categorized.json to get functions for one category:
   ```bash
   jq '[.[] | select(.category == "validation")]' categorized.json > validation-functions.json
   ```
3. Replace `{CATEGORY}` with the category name
4. Replace `<INSERT_CATEGORY_FUNCTIONS_HERE>` with the filtered JSON
5. Dispatch opus subagent with the prompt
6. Repeat for each category with 3+ functions
7. Combine outputs into final report
