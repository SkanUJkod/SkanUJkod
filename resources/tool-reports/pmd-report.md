# Report: PMD
This report provides a comprehensive analysis of the PMD tool, focusing on its User Interface, Available Metrics, Extensibility, and Architecture. PMD, an open-source static code analysis tool, is widely used for detecting programming flaws and improving code quality across multiple languages, primarily Java and Apex. The following sections detail each aspect, drawing from recent documentation and resources.

## 1: User interface
PMD, as an established open-source static code analysis tool, offers developers a multifaceted approach to integrating code quality checks into their workflows. Primarily recognized for its capabilities in analyzing Java and Apex code, PMD also extends its support to a wide array of other programming languages.

The utility of such a tool in modern software development cannot be overstated, as it plays a crucial role in identifying potential programming flaws, enforcing coding standards, and ultimately improving the overall quality and maintainability of software projects. This report aims to provide a detailed examination of PMD, focusing on its user interface, the metrics it provides for code assessment, the mechanisms it offers for extensibility, and its foundational architectural design. This analysis builds upon an initial overview of PMD and delves deeper into its functionalities and underlying principles to provide a comprehensive understanding of its capabilities.

### Using PMD via CLI
PMD's CLI allows users to run static code analysis from the terminal, making it suitable for automation and scripting. To use PMD via CLI, follow these steps:

1. **Download PMD**: Obtain the latest PMD binary distribution from [PMD Releases](https://github.com/pmd/pmd/releases). The latest version is typically available as a zip file (e.g., `pmd-bin-7.0.0.zip`).
2. **Extract the Archive**: Unzip the downloaded file to a directory, e.g., `/path/to/pmd`.
3. **Run PMD**: Use the `pmd` script (Linux/macOS) or `pmd.bat` (Windows) located in the `bin` directory. A basic command looks like:
   ```bash
   /path/to/pmd/bin/pmd check -d /path/to/source/code -R rulesets/java/quickstart.xml -f text
   
### Using PMD via Build Tools (Dependencies and Integration)
PMD can be integrated into build systems like Maven and Gradle, automating analysis during builds.

**Maven**:
Add Dependency: Include the PMD Maven plugin in your pom.xml:

```
 <plugin>
      <groupId>org.apache.maven.plugins</groupId>
      <artifactId>maven-pmd-plugin</artifactId>
      <version>3.21.2</version>
 </plugin>
```

**Run**: Execute mvn pmd:check to analyze code. Customize rulesets via <rulesets> in the plugin configuration.
Dependencies: Requires Maven and Java. The plugin downloads PMD automatically.

**Gradle**:
Add Plugin: In build.gradle, apply the PMD plugin:

```
plugins {
  id 'pmd'
}
pmd {
  ruleSets = ['category/java/bestpractices.xml']
  toolVersion = '7.0.0'
}
```

**Run**: Use ./gradlew pmdMain to run analysis.
Dependencies: Requires Gradle and Java. PMD is fetched via Gradle’s dependency management.

## 2: Available metrics

PMD offers a robust set of over 400 built-in rules as metrics to assess code quality across multiple categories, complemented by a Copy-Paste Detector (CPD) for identifying duplicated code.

- It clearly states the purpose of PMD's metrics: Before diving into the specifics of rule categories and examples, this sentence immediately tells the reader why these metrics are important. It sets the stage by explaining that they are not just arbitrary checks, but tools designed to achieve specific goals.
- It reinforces the overall message of the report: Our research aims to provide a comprehensive analysis of PMD's capabilities. This sentence aligns with that goal by emphasizing the positive impact of PMD's features on code quality and development practices.
### Rule Categories and Examples
PMD's rules are grouped into key areas, with examples from [PMD Java Rules](https://pmd.github.io/pmd/pmd_rules_java.html):

- **Best Practices**: Ensures sound coding habits (e.g., `AvoidPrintStackTrace`).
- **Code Style**: Enhances readability (e.g., `MethodNamingConventions`).
- **Design**: Identifies design flaws (e.g., `GodClass`).
- **Documentation**: Checks documentation quality (e.g., `CommentRequired`).
- **Error Prone**: Flags potential bugs (e.g., `UnusedLocalVariable`, `EmptyCatchBlock`).
- **Performance**: Optimizes efficiency (e.g., `UseCollectionIsEmpty`).
- **Security**: Addresses vulnerabilities (e.g., `SqlInjection`).

For instance:
- AvoidPrintStackTrace:
```java
try {
    int result = 10 / 0;
} catch (ArithmeticException e) {
    e.printStackTrace(); // PMD will flag this line
}
```
This rule flags the use of printStackTrace() because it's generally better to use a logging framework to handle exceptions. Logging provides more control over the output and its destination.   

- MethodNamingConventions'
```java
public class MyClass {
    public void DoSomething() { // PMD will flag this method name
        //... method body...
    }
}
```
This rule expects method names to start with a lowercase letter and follow camelCase. DoSomething violates this convention.   

- UnusedLocalVariable
```java
public class Example {
    public void processData() {
        int counter = 0; // PMD will flag this variable
        int result = 10;
        System.out.println("The result is: " + result);
    }
}
```
The local variable counter is declared but never used within the processData method, which would trigger this rule.   

- SqlInjection
```java
import java.sql.Connection;
import java.sql.Statement;
import java.sql.SQLException;

public class UserDAO {
    public void findUser(String username) {
        Connection connection = getConnection();
        try {
            Statement statement = connection.createStatement();
            String sql = "SELECT * FROM users WHERE username = '" + username + "'"; // PMD will flag this
            statement.executeQuery(sql);
        } catch (SQLException e) {
            //... handle exception...
        }
    }

    private Connection getConnection() {
        //... returns a database connection...
        return null;
    }
}
```
This rule detects potential SQL injection vulnerabilities where user-provided input (username) is directly concatenated into a SQL query without proper sanitization. This could allow malicious users to execute arbitrary SQL commands.   

- UseCollectionIsEmpty
```java
import java.util.List;
import java.util.ArrayList;

public class DataProcessor {
    public void checkList(List<String> items) {
        if (items.size() == 0) { // PMD will suggest using isEmpty()
            System.out.println("The list is empty.");
        }
    }
}
```

## 3: Extensibility

PMD’s extensibility allows users to create custom rules, tailoring it to specific coding standards or project needs, enhancing its versatility.

### Key Features
- **Custom Rules**: Write rules in Java (extending `AbstractJavaRule`) or XPath for AST pattern matching ([Writing Custom Rules for PMD](https://pmd.github.io/latest/pmd_userdocs_extending_writing_pmd_rules.html)).
- **Java Rules**: Compile and bundle into JAR files for integration ([PMD Writing Java Rules](https://docs.pmd-code.org/latest/pmd_userdocs_extending_writing_java_rules.html)).
- **XPath Rules**: Simpler approach for specific patterns.
- **Rule Designer**: Graphical tool for AST inspection and XPath editing, with examples at [PMD Rule Designer](https://pmd.github.io/pmd/pmd_userdocs_extending_designer_reference.html).

## 4:Architecture

PMD’s architecture is built for robustness and flexibility, focusing on parsing and analyzing code across multiple languages with a modular, extensible design.

### Key Components
- **Parsing**: Uses JavaCC for Java and Antlr for other languages to create Abstract Syntax Trees (ASTs) ([PMD Main Documentation](https://pmd.github.io/pmd/)).
- **Rule Application**: Traverses ASTs to detect violations, with thread-safe rule execution via deep copies ([PMD Writing Java Rules](https://docs.pmd-code.org/latest/pmd_userdocs_extending_writing_java_rules.html)).

### Features
- **Multi-Language Support**: Covers Java, Apex, JavaScript, Kotlin, and over 16 other languages with modular parsers.
- **Integrations**: Supports IDEs, build tools, and CI systems for seamless workflow inclusion.
- **Extensibility**: Allows custom rules and rulesets for tailored analysis.

### Design Insights
- **ADRs**: Architecture Decision Records provide development history ([PMD Architecture Decision Records](https://docs.pmd-code.org/latest/pmd_projectdocs_decisions_adr_1.html)).
- **Open-Source**: Licensed under BSD and Apache 2.0, fostering community contributions ([PMD (software) - Wikipedia](https://en.wikipedia.org/wiki/PMD_%28software%29)).
