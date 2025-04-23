# Report: SonarQube

## 1: User Interface

- **Analyzing an Example Project**:  
  To analyze a project using SonarQube, you first set up the SonarQube server and configure your project via a `sonar-project.properties` file. For example, you specify the project key, name, and source directories. You then run the analysis using the SonarScanner CLI tool, which sends your project’s code to the SonarQube server. Detailed tutorials can be found on the [SonarQube Documentation site](https://docs.sonarsource.com/sonarqube-server/latest/).

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

---

## 3: Extensibility

- **Adding a Custom (Dummy) Check**:  
  SonarQube supports extensibility via custom plugins written in Java. To add a dummy check:

  - **Create a Plugin Project**: Use Maven to set up a new project and include the SonarQube Plugin API as a dependency.
  - **Implement the Check**: Develop a simple rule (e.g., flagging methods that exceed a certain length). This involves extending the appropriate base classes and using the provided API to traverse the AST.
  - **Register the Rule**: Add the new check to a quality profile so that it is executed during analysis.

- **Developer Experience**:

  I did not encounter any major problems while writing the plugin itself — the documentation is well-structured and clearly explains how to work with the API and define custom rules. Any problems can be found on the [forum](https://community.sonarsource.com/)

  However, I did run into some issues when building the plugin and integrating the scanner with the SonarQube server. These parts were less straightforward, especially when dealing with version compatibility and missing dependencies.

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

### 4.2 Input Representation

- **Project vs. File-Level Analysis**:  
  SonarQube is capable of analyzing both entire projects and individual files. It abstracts input representation through language-specific plugins.
- **Abstraction Layer for Multi-Language Support**:  
  For supporting multiple languages, SonarQube employs an abstraction layer that defines common operations (e.g., parsing, metric extraction) irrespective of the language. This is achieved by using dedicated language plugins that handle the nuances of each language.

### 4.3 Parsing

- **Implementation of Parsing and Semantic Analysis**:

  - **Java**:  
    Uses the Eclipse JDT compiler to parse source code into an Abstract Syntax Tree (AST) and perform semantic analysis.
  - **Other Languages**:  
    For languages other than Java, SonarQube employs the [SonarSource Language Recognizer (SSLR)](https://github.com/SonarSource/sslr) toolkit. SSLR handles lexing, parsing, and AST construction, providing a flexible framework that supports multiple programming languages.

- **Delegation vs. Native Implementation**:
  SonarQube leverages existing compiler utilities for parsing (e.g., Eclipse JDT for Java), while additional semantic analysis is performed natively within the framework to check for context-sensitive issues like variable declarations and type checking.

- **Additional Information**:
  For more details on extending SonarQube with language-specific plugins and further understanding the parsing process, refer to the [Developing a Language Plugin](https://docs.sonarsource.com/sonarqube-server/10.8/extension-guide/developing-a-plugin/plugin-basics/) guide.

### 4.4 Linking Checks

- **Plugin Communication**:  
  Communication between the SonarQube core and plugins is managed through a well-defined [Plugin API](https://docs.sonarsource.com/sonarqube-server/latest/extension-guide/developing-a-plugin/plugin-basics/). Plugins analyze the AST and report findings back to the core.
- **Dynamic Linking**:  
  Plugins are packaged as JAR files and loaded dynamically at runtime by the SonarQube server. This modular design allows new checks to be added simply by placing the appropriate JAR file into the plugins directory, eliminating the need for static linking or recompiling the core system.

- **Data Flow**:  
  Since plugins run in the same process as the core system, data is exchanged via direct method calls. Although the API defines standardized (and serializable) data structures—which can be useful for caching or persistence—there is no reliance on inter-process communication (IPC) methods like sockets or pipes.
- **Performance Considerations**:  
  The dynamic plugin architecture is designed to minimize overhead, and performance optimizations are documented in the official SonarQube documentation and community discussions.

---

## 5: Useful Links

### **Sensors**

In SonarQube, **sensors** are components that collect and convert raw source code into structured data for analysis. They run during code analysis to gather metrics, detect issues, and support language-specific processing, often extended via plugins.

[More info → Optimize Sensors](https://github.com/SonarSource/sonar-plugin-api/blob/master/docs/optimize-sensors.md)

---

### **Skipping Unchanged Files**

When analyzing pull requests, only changed files are sent to the server, allowing analyzers to skip processing unchanged ones.

[See section in docs](https://github.com/SonarSource/sonar-plugin-api/blob/master/docs/optimize-sensors.md#skipping-unchanged-files)

---

### **Analyzer Cache**

Analyzers can store and retrieve persistent data in a server-side cache for use in future analyses. Each analyzer must ensure unique keys to avoid conflicts.

[See section in docs](https://github.com/SonarSource/sonar-plugin-api/blob/master/docs/optimize-sensors.md#analyzer-cache)

---

### **Security & Compliance**

Overview of SonarQube’s security rules, vulnerabilities, and OWASP Top 10 support.  
[Security & Compliance](https://docs.sonarsource.com/sonarqube-server/9.6/user-guide/rules/security-related-rules/)

---

### **Optimization & Best Practices**

Best practices for implementing efficient sensors in custom analyzers.  
[Optimization & Best Practices](https://github.com/SonarSource/sonar-plugin-api/blob/master/docs/optimize-sensors.md)

Guide to writing and deploying your own static analysis rules.  
[Custom Rules](https://docs.sonarsource.com/sonarqube-server/9.6/extension-guide/adding-coding-rules/)

---

### **Community & Ecosystem**

Official discussion forum for help, feedback, and news about SonarQube and SonarCloud.  
[Community Forum](https://community.sonarsource.com/)
