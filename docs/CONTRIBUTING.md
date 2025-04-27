# Contributing Guide

This guide outlines our conventions to keep the SkanUJkod project clean, consistent, and welcoming to contributors.

---

## Branch Naming

Branches should follow the pattern:

| **Type**  | **Description**                            |
| --------- | ------------------------------------------ |
| `feature` | New functionality                          |
| `bug`     | Bug fixes                                  |
| `chore`   | Maintenance tasks (e.g., dependency bumps) |
| `test`    | Adding or updating tests                   |

- **short-description**: concise phrase in _kebab-case_ (lowercase, words separated by hyphens).

**Examples:**

```text
feature/add-verbose-flag
bug/fix-parse-error
chore/update-readme
test/add-coverage-tests
```

---

## Commit Message Template

Use the following **literal template** for commit messages:

### Components

| **Field**       | **Details**                                               |
| --------------- | --------------------------------------------------------- |
| **Type**        | One of `FEAT`, `FIX`, `DOCS`, `REFACTOR`, `TEST`, `CHORE` |
| **Scope**       | Module or area, e.g., `CLI`, `coverage`, `parser`         |
| **Description** | Short description about your changes                      |

### Examples

```text
FEAT(CLI): add --verbose flag for detailed output
```

```text
FIX(Parser): prevent panic on empty input
```

```text
DOCS(README): update installation instructions
```

```text
REFACTOR(AST): extract instrumentation into helper module
```

```text
TEST(Coverage): add tests for statement coverage function
```

```text
CHORE(Deps): bump go_parser to v0.1.6
```

---
