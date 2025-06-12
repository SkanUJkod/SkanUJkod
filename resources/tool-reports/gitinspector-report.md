# Report: Gitinspector

## 1: User Interface

### Usage and Example Analysis

GitInspector is a tool designed for analyzing Git repositories. It provides insights into repository activity, such as author contributions, commit history, and file changes. The tool is particularly useful for understanding team dynamics in software development projects.

For a more comprehensive tutorial and examples of usage, refer to the official documentation: [GitInspector Documentation](https://github.com/ejwa/gitinspector/wiki/Documentation)

### CLI vs GUI

GitInspector is purely a **CLI** tool. There is no native GUI. However, it can generate HTML reports, which offer a more readable, graphical representation of its analysis results.

### Configuration and Parameter Handling

GitInspector allows customization through a range of command-line parameters. Users can specify options related to:

- **Report format** (e.g., HTML, text)
- **Filtering commits** by date range or author
- **Including/excluding specific files or directories**
- **Code ownership analysis**

Example of filtering commits by date range:
```sh
gitinspector --since=2024-01-01 --until=2024-03-01 -HTlrm
```

There is no separate configuration file - **all settings must be provided via command-line arguments**.

### Order of Plugin / Check Execution

GitInspector does not have a dedicated plugin system where users can specify the order of execution. However, the available options allow users to enable or disable specific analysis checks. Reports always follow a structured format, showing commit statistics first, followed by author contributions, file changes, and code ownership information.

### Online vs Offline Analysis

GitInspector is a **static analysis tool** that works in an **offline mode**. It does not continuously monitor a repository but instead processes its commit history and outputs a report at a specific point in time. Users need to re-run GitInspector manually to update the analysis results.

Users can automate its execution using cron jobs or CI/CD pipelines to generate reports at regular intervals.

## 2: Available Metrics

GitInspector specializes in **historical analysis of Git repositories**, focusing on author contributions, commit frequency, and code evolution. It provides detailed reports that help understand **who contributed what, how frequently commits occur, and how files have changed over time**.

#### a) **Commit and Author Statistics**
   - **Total commits per author**
   - **Active authors** – Shows which contributors have made commits within a given timeframe.
   - **First and last commit dates per author** – Helps determine how long an author has been active in the project.
   - **Percentage of total commits**

#### b) **Code Ownership Analysis**
   - **Lines of code added/removed per author** – Helps track who contributes most to the actual codebase.
   - **Who last modified each file** – Useful for identifying the primary maintainer of specific files.
   - **Code ownership percentage**

#### c) **File and Repository Activity Metrics**
   - **Most frequently modified files**
   - **Files with the highest churn (lines added + lines removed)** – Highlights unstable or rapidly evolving parts of the project.
   - **Total number of commits per file**
   - **Total repository size and line count**

#### d) **Commit Message Analysis**
   - **Most common words in commit messages** – Helps detect patterns or common themes in development.
   - **Commit message length distribution** – Can reveal whether commit messages follow best practices.

### Most Useful Metrics

- **Commit statistics per author** – Essential for understanding contributor activity.
- **Code ownership percentage** – Helps track who maintains what.
- **Most frequently modified files** – Useful for detecting hotspots in the codebase.
- **Stale files and branches** – Useful for detecting technical debt.
- **Churn rate (lines added + lines removed per file)** – Helps identify unstable parts of the project *(not directly present in the project, but very useful)*.

## 3: Extensibility

### Extending GitInspector

GitInspector does **not** have a built-in plugin system or official documentation for extending its functionality with custom checks. It is a standalone script that generates reports based on predefined logic. To modify or extend GitInspector, one must directly modify its Python source code.

### Dummy Check Implementation - couting the number of commit messages containing the word *'fix'*

#### 1. Locating the processing logic
The main logic of GitInspector is in `gitinspector.py`, where it parses commits and generates reports.

#### 2. Adding a custom function
Modify `gitinspector.py` to include a function that counts commits containing "fix":

```python
import re

def count_fix_commits(commits):
    """Counts how many commit messages contain the word 'fix'."""
    fix_count = sum(1 for commit in commits if re.search(r'\bfix\b', commit.message, re.IGNORECASE))
    return fix_count
```

#### 3. Integrating it into the report
Find the function where the final report is generated (usually around where statistics are compiled), and modify it to include the new check:

```python
fix_commit_count = count_fix_commits(commit_list)
print(f"Commits containing 'fix': {fix_commit_count}")
```

#### What Was Easy?
- Since GitInspector is a simple Python script, modifying it was straightforward.
- The commit parsing logic is well-structured, making it easy to extract commit messages.

#### What Was Difficult
- There is **no official plugin system**, so every modification requires direct changes to the source code.
- Adding new checks requires manually modifying report generation logic.
- The CLI argument parser must be manually updated to support custom options.

### Static vs. Dynamic Linking

Since GitInspector does not have a plugin system, new checks must be **hardcoded** into the source code. This means that every change **requires modifying and re-running the script**. Also, there is no way to dynamically load external scripts or plugins without modifying the core tool.

## 4: Architecture

### 1. Project Structure

### Key Modules

- **`gitinspector.py`** – The main script that executes the analysis, gathering statistical data from a Git repository. It coordinates different modules to produce insightful reports on code contributions.
- **`metrics`** – Analyzes various code quality indicators, such as commit frequency and changes per author. This module helps assess coding activity and patterns over time.
- **`responsibilities`** – Identifies which authors are responsible for different files in the repository. It provides an overview of contributions to specific sections of the project.
- **`format`** – Converts output data into different formats, including HTML, JSON, XML, and plain text. This makes results easier to share and visualize in various tools.
- **`file-types`** – Allows filtering of analysis based on file extensions. Users can narrow down statistics to specific types of code or documents.
- **`grading`** – Structures collected data into a format useful for grading coding projects. This is particularly helpful in educational settings to evaluate student contributions.

### Observations
- Reports are generated in-memory and then output as text or HTML.
- No external dependencies except for standard Python libraries.

### 2. How Inputs Are Represented

GitInspector analyzes **entire Git repositories** rather than single files. It extracts historical data using the `git log` command and processes it internally.

The input consists of:

- **Commit history** (`git log --pretty=format:%H`)
- **Author details** (`git log --format='%an <%ae>'`)
- **Changed files** (`git log --name-only --pretty=format:''`)
- **File line contributions** (`git blame`)

### 3. Linking Checks

Since GitInspector **does not have a plugin system**, all checks are hardcoded into the main script.

- All metrics and checks (e.g., commit counts, file modifications) are part of the core codebase.
- Adding a new check requires modifying the main scripts (`gitinspector.py`, `commits.py`, `output.py`).
- No dynamic configuration allows adding checks externally.
