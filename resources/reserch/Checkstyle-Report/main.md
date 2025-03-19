


### 1. User Interface

we can use Checkstyle only from terminal. To use Checkstyle we must to download jar file (https://github.com/checkstyle/checkstyle/releases/). We specifie check which we want to execute using .xml file (example is the same directory).  Checkstyle inform us when specified metric if we exceed the limit in code. There is a GUI but only for vizualization of AST.

### 2. Aviable Metrics
Full list of aviable checks https://checkstyle.sourceforge.io/checks.html . Checkstyle is a tool only for Java so we can find some checks which are applicable only for Java. I choose 18 universal checks (aviable in file coolChecks.md).  

### 3. Extensibility
Checkstyle's TreeWalker takes a set of objects that extend the AbstractCheck class. A Check provides methods that take an AST as an argument and perform the checking process for that AST. It is important to understand that the individual Checks do not drive the AST traversal (it is possible to traverse the tree manually, but not recommended). Instead, the TreeWalker traverses the tree using a tree traversal (depth-first) algorithm. Checkstyle provide DetailAST interface for ast traversalm. 

