


### 1. User Interface
We can use Checkstyle only from the terminal. To use it, we must download the .jar file from the Checkstyle releases page (https://github.com/checkstyle/checkstyle/releases/). We specify the checks we want to execute using an .xml configuration [file](.\config.xml) (an example file is available in the same directory). Checkstyle notifies us when the code exceeds the limits defined by the specified metrics.

There is also a GUI available, but it is only for visualizing the Abstract Syntax Tree (AST), not for running checks.

### 2. Available Metrics
Full list of aviable checks https://checkstyle.sourceforge.io/checks.html. Checkstyle is a tool only for Java so we can find some checks which are applicable just for Java. I choose 18 universal checks (aviable in file [coolChecks.md](../coolChecks.md)).  

### 3. Extensibility
Basics
A few basics to understand:
Checkstyle's TreeWalker takes a set of objects that extend the AbstractCheck class. A Check provides methods that take an AST as an argument and perform the checking process for that AST. It is important to understand that the individual checks do not drive the AST traversal (it is possible to traverse the tree manually, but not recommended). Instead, the TreeWalker traverses the tree using a tree traversal (depth-first) algorithm. Checkstyle provide DetailAST interface for AST traversals. 

### 4. Architecture 
plugins are linked dynamically https://checkstyle.sourceforge.io/writingchecks.html#Understanding_the_visitor_pattern
Checkstyle analize entire project, but we can specify single file to check. 
### Parsing 
Checkstyle uses ANTLR4 for parsing 
Checkstyle is aviable only for Java 



