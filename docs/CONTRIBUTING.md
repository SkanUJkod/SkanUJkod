# Contributing Guide

This guide outlines our conventions to keep the SkanUJkod project clean, consistent, and welcoming to contributors.

---

## Branch Naming

Branches should follow the pattern:

| **Type**   | **Description**                             |
| ---------- | ------------------------------------------- |
| `feature`  | New functionality                           |
| `bug`      | Bug fixes                                   |
| `chore`    | Maintenance tasks (e.g., dependency bumps)  |
| `test`     | Adding or updating tests                    |
| `docs`     | Documentation updates                       |
| `refactor` | Code refactoring without functional changes |

- **short-description**: concise phrase in _kebab-case_ (lowercase, words separated by hyphens).

**Examples:**

```text
feature/add-verbose-flag
bug/fix-parse-error
chore/bump-dependencies
docs/update-readme
test/add-coverage-tests
refactor/simplify-instrumentation
```

---

## Commit Message Template

Use the following **literal template** for commit messages:

### Components

| **Field**       | **Details**                                                           |
| --------------- | --------------------------------------------------------------------- |
| **Type**        | One of `FEAT`, `BUG`, `CHORE`, `TEST`, `DOCS`, `REFACTOR`             |
| **Scope**       | Module or area, e.g., `CLI`, `coverage`, `parser`                     |
| **Description** | Short description about your changes                                  |
| **Issue/PR**    | (Optional) Add issue or PR reference in the format (#123) at the end. |

### Examples

```text
FEAT(CLI): add --verbose flag for detailed output (#1)
```

```text
BUG(Parser): prevent panic on empty input (#2)
```

```text
DOCS(README): update installation instructions (#3)
```

```text
REFACTOR(AST): extract instrumentation into helper module (#4)
```

```text
TEST(Coverage): add tests for statement coverage function (#5)
```

```text
CHORE(Deps): bump go_parser to v0.1.6 (#6)
```

---
