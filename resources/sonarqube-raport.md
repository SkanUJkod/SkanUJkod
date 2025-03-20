# Report: SonarQube

## 1: User Interface

- **Analyzing an Example Project**:  
  To analyze a project using SonarQube, you first set up the SonarQube server and configure your project via a `sonar-project.properties` file. For example, you specify the project key, name, and source directories. You then run the analysis using the SonarScanner CLI tool, which sends your project’s code to the SonarQube server. Detailed tutorials can be found on the [SonarQube Documentation site](https://docs.sonarqube.org/latest/analysis/scan/).

- **CLI or GUI?**  
  SonarQube offers both a CLI (through SonarScanner) for integrating into CI/CD pipelines and a web-based GUI for reviewing analysis results, configuring projects, and managing quality profiles.

- **Configuration and Parameter Passing**:

  - **Project-Level Configuration**: Managed via the `sonar-project.properties` file or directly through the web interface.
  - **Command-Line Parameters**: Additional settings can be passed when running SonarScanner, for example:
    ```bash
    sonar-scanner -Dsonar.projectKey=your_project_key -Dsonar.sources=./src
    ```
  - **Global Settings**: Managed within the SonarQube web dashboard.

- **Order of Plugins/Checks Execution**:  
  There is no explicit mechanism to set the execution order of plugins or checks. SonarQube internally manages the processing sequence to optimize performance and ensure comprehensive analysis.

- **Online vs. Offline Operation**:  
  SonarQube operates in an offline analysis mode where it processes the entire input project and outputs the results after completion. However, when integrated into CI/CD pipelines, it effectively provides near real-time feedback on code quality changes.

---

## 2: Available Metrics

- **Specialized Analyses**:  
  SonarQube specializes in several key areas of code quality analysis:

  - **Bugs**: Detection of potential defects that may cause runtime errors.
  - **Security Vulnerabilities**: Identification of security risks and code sections susceptible to attacks.
  - **Code Smells**: Highlighting maintainability issues that could lead to technical debt.
  - **Duplications**: Detection of duplicate code blocks that may need refactoring.
  - **Test Coverage**: Assessment of how much code is covered by automated tests.
  - **Complexity Metrics**: Analysis of cyclomatic complexity and other measures to identify overly complicated code.

- **Metrics for MVP**:  
  For an MVP, focusing on bugs, security vulnerabilities, and code smells provides the most immediate benefit by directly impacting code reliability and maintainability.

---

## 3: Extensibility

- **Adding a Custom (Dummy) Check**:  
  SonarQube supports extensibility via custom plugins written in Java. To add a dummy check:

  - **Create a Plugin Project**: Use Maven to set up a new project and include the SonarQube Plugin API as a dependency.
  - **Implement the Check**: Develop a simple rule (e.g., flagging methods that exceed a certain length). This involves extending the appropriate base classes and using the provided API to traverse the AST.
  - **Register the Rule**: Add the new check to a quality profile so that it is executed during analysis.

- **Documentation**:  
  Detailed instructions for creating custom plugins and checks are available in the [SonarQube Plugin Development Guide](https://docs.sonarsource.com/sonarqube/latest/extend-developer-guide/developing-a-plugin/).

- **Developer Experience**:

  - **Challenges**: Understanding the API structure and configuring the plugin correctly may have a steep learning curve if you are new to SonarQube internals.
  - **Ease**: Once familiar with the API, the modular design makes it straightforward to add and test custom rules without affecting the core functionality.

- **Plugin Linking**:  
  Plugins are dynamically loaded as JAR files at runtime. This dynamic linking means you can add or update plugins without recompiling the entire SonarQube server.

---

## 4: Architecture

### 4.1 Project Structure

- **Modular Design**:  
  The SonarQube project is divided into several modules:
  - **Server**: Handles web requests, processes analysis data, and serves the GUI.
  - **Database**: Stores configuration settings, analysis results, and historical data.
  - **Scanners**: Tools like SonarScanner perform code analysis and send the results to the server.
- **Repository Organization**:  
  The source code is organized into subdirectories corresponding to these core components. Tools like UML diagram generators can help visualize the module relationships.

### 4.2 Input Representation

- **Project vs. File-Level Analysis**:  
  SonarQube is capable of analyzing both entire projects and individual files. It abstracts input representation through language-specific plugins.
- **Abstraction Layer for Multi-Language Support**:  
  For supporting multiple languages, SonarQube employs an abstraction layer that defines common operations (e.g., parsing, metric extraction) irrespective of the language. This is achieved by using dedicated language plugins that handle the nuances of each language.

### 4.3 Parsing

- **Implementation of Parsing and Semantic Analysis**:
  - **Java**: Uses the Eclipse JDT compiler to parse source code into an Abstract Syntax Tree (AST) and perform semantic analysis.
  - **Other Languages**: Utilizes the SonarSource Language Recognizer (SSLR) toolkit for lexing, parsing, and building ASTs.
- **Delegation vs. Native Implementation**:  
  SonarQube leverages existing compiler utilities for parsing (e.g., Eclipse JDT for Java), while additional semantic analysis is performed natively within the framework to check for context-sensitive issues like variable declarations and type checking.

### 4.4 Linking Checks

- **Plugin Communication**:  
  Communication between the SonarQube core and plugins is managed through a well-defined Plugin API. Plugins analyze the AST and report findings back to the core.
- **Dynamic Linking**:  
  Plugins are dynamically loaded as JAR files at runtime. This modular approach avoids the need for static linking or recompiling the core system when adding new checks.
- **Data Flow**:  
  Data exchanged between the core and plugins is passed through method calls and serialized structures defined by the API. There is no use of inter-process communication (IPC) such as sockets or pipes.
- **Performance Considerations**:  
  The dynamic plugin architecture is designed to minimize overhead, and performance optimizations are documented in the official SonarQube documentation and community discussions.

---
