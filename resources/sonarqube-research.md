## usefull links to further investigation

[core](https://github.com/SonarSource/sonarqube)
[commons](https://github.com/SonarSource/sonar-analyzer-commons)

[scanner go](https://github.com/SonarSource/sonar-go)

[cli](https://github.com/SonarSource/sonar-scanner-cli)


[Developing a Query Engine for Source Code Analyzers](https://github.com/SonarSource/analysis-ast-query/blob/master/MasterThesis.pdf)
[thesis gh project](https://github.com/SonarSource/analysis-ast-query)

[Start Analyzing your Projects with SonarQube](https://docs.bitnami.com/general/how-to/analyze-projects-sonarqube/)


## **SonarQube Architecture**​
- **scanner:** analyzes source code and reports findings to the server, integrated with build systems​
- **server:** hub that computes metrics and provides a web interface​
- **database:** stores analysis results for efficient querying​
- **plugins:** extend capabilities to support more languages and rules​
- **web interface:** platform for viewing code quality metrics and trends​
- **APIs:** enable integration with CI/CD systems  
  

## **Scanner/Analyzer Architecture:**

- **command-line tool:** triggers source code analysis, integrates with build systems like Maven and Gradle, and fits into CI/CD pipelines
- **language plugins:** parses code and applies language-specific rules to detect issues, bugs, and code smells
- **configuration files:** uses `sonar-project.properties` to set analysis parameters, customize quality profiles, and specify file inclusion/exclusion
- **static code analysis engine:** executes rule-based analysis to evaluate code quality and gather metrics
- **output & reporting:** generates detailed reports of findings and submits results to the SonarQube server for processing
- **extendability:** supports custom rules and community-driven enhancements via plugins

The SonarScanner analyzes source code and sends results to SonarQube Community Build for quality gate calculations and report generation. Language analyzers, downloaded at installation, perform the analysis. SonarScanners integrate with Gradle, Maven, .NET, NPM, and Python, while a CLI version supports other project types with more manual setup.

https://docs.sonarsource.com/sonarqube-community-build/analyzing-source-code/analysis-overview/

## **sensors**
In SonarQube, **sensors** are components that collect and transform raw code data into structured information for analysis. They run during code analysis to gather metrics, identify code issues, and process language-specific syntax, supporting plugins to extend functionality. Sensors help convert source code into actionable insights on quality and security.
https://github.com/SonarSource/sonar-plugin-api/blob/master/docs/optimize-sensors.md


## [sonar-plugin-api:](https://github.com/SonarSource/sonar-plugin-api)

[Skipping Unchanged Files](https://github.com/SonarSource/sonar-plugin-api/blob/master/docs/optimize-sensors.md#skipping-unchanged-files)
Currently, the strategy when analyzing pull requests is to only send files to the server that were detected as changed.  In that situation, some analyzers can skip the analysis of unchanged files. 

[Analyzer cache](https://github.com/SonarSource/sonar-plugin-api/blob/master/docs/optimize-sensors.md#analyzer-cache)
Analyzers can persist data in a cache that will be made available to it in a later analysis. The cached data is stored on the server side and the analyzer can store and retrieve data using any key. Analyzers must make sure that there’s no risk of key collision between different analyzers.


https://arc.net/l/quote/qobclsqu