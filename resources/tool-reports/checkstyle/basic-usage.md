basic usage 
```cmd
    C:\checkstyle> java -jar checkstyle-10.21.4-all.jar -c config.xml  C:\checkstyle\testCheckStyle\src\Test.java
Starting audit...
[ERROR] C:\checkstyle\testCheckStyle\src\Test.java:17:17: Fall through from previous branch of the switch statement. [FallThrough]
Audit done.
Checkstyle ends with 1 errors.
```

```cmd
C:\checkstyle> java -jar checkstyle-10.21.4-all.jar -c sun_checks.xml  C:\checkstyle\testCheckStyle\src\Test.java
Starting audit...
[ERROR] C:\checkstyle\testCheckStyle\src\Test.java:1: File does not end with a newline. [NewlineAtEndOfFile]
[ERROR] C:\checkstyle\testCheckStyle\src\Test.java:1: Missing package-info.java file. [JavadocPackage]
[ERROR] C:\checkstyle\testCheckStyle\src\Test.java:13:13: switch without "default" clause. [MissingSwitchDefault]
[ERROR] C:\checkstyle\testCheckStyle\src\Test.java:17:22: '3' is a magic number. [MagicNumber]
Audit done.
Checkstyle ends with 4 errors.

```



```cmd 
C:\checkstyle> java -jar checkstyle-10.21.4-all.jar  -T  C:\checkstyle\testCheckStyle\src\Test.java
COMPILATION_UNIT -> COMPILATION_UNIT [6:0]
`--CLASS_DEF -> CLASS_DEF [6:0]
    |--MODIFIERS -> MODIFIERS [6:0]
    |--BLOCK_COMMENT_BEGIN -> /* [1:0]
    |   |--COMMENT_CONTENT -> *\r\n * Javadoc summary.\r\n *\r\n * Some description.\r\n  [1:2]
    |   `--BLOCK_COMMENT_END -> */ [5:1]
    |--LITERAL_CLASS -> class [6:0]
    |--IDENT -> Test [6:6]
    `--OBJBLOCK -> OBJBLOCK [6:11]
        |--LCURLY -> { [6:11]
        |--METHOD_DEF -> METHOD_DEF [10:4]
        |   |--MODIFIERS -> MODIFIERS [10:4]
        |   |   |--BLOCK_COMMENT_BEGIN -> /* [7:4]
        |   |   |   |--COMMENT_CONTENT -> *\r\n     * Some summary on method.\r\n      [7:6]
        |   |   |   `--BLOCK_COMMENT_END -> */ [9:5]
        |   |   `--LITERAL_PUBLIC -> public [10:4]
        |   |--TYPE -> TYPE [10:11]
        |   |   `--LITERAL_VOID -> void [10:11]
        |   |--IDENT -> foo [10:16]
        |   |--LPAREN -> ( [10:19]
        |   |--PARAMETERS -> PARAMETERS [10:20]
        |   |--RPAREN -> ) [10:20]
        |   `--SLIST -> { [10:22]
        |       |--VARIABLE_DEF -> VARIABLE_DEF [11:8]
        |       |   |--MODIFIERS -> MODIFIERS [11:8]
        |       |   |--TYPE -> TYPE [11:8]
        |       |   |   `--LITERAL_INT -> int [11:8]
        |       |   |--IDENT -> i [11:12]
        |       |   `--ASSIGN -> = [11:14]
        |       |       `--EXPR -> EXPR [11:16]
        |       |           `--NUM_INT -> 0 [11:16]
        |       |--SEMI -> ; [11:17]
        |       |--LITERAL_WHILE -> while [12:8]
        |       |   |--LPAREN -> ( [12:14]
        |       |   |--EXPR -> EXPR [12:17]
        |       |   |   `--GE -> >= [12:17]
        |       |   |       |--IDENT -> i [12:15]
        |       |   |       `--NUM_INT -> 0 [12:20]
        |       |   |--RPAREN -> ) [12:21]
        |       |   `--SLIST -> { [12:23]
        |       |       |--LITERAL_SWITCH -> switch [13:12]
        |       |       |   |--LPAREN -> ( [13:19]
        |       |       |   |--EXPR -> EXPR [13:20]
        |       |       |   |   `--IDENT -> i [13:20]
        |       |       |   |--RPAREN -> ) [13:21]
        |       |       |   |--LCURLY -> { [13:23]
        |       |       |   |--CASE_GROUP -> CASE_GROUP [14:16]
        |       |       |   |   |--LITERAL_CASE -> case [14:16]
        |       |       |   |   |   |--EXPR -> EXPR [14:21]
        |       |       |   |   |   |   `--NUM_INT -> 1 [14:21]
        |       |       |   |   |   `--COLON -> : [14:22]
        |       |       |   |   |--LITERAL_CASE -> case [15:16]
        |       |       |   |   |   |--EXPR -> EXPR [15:21]
        |       |       |   |   |   |   `--NUM_INT -> 2 [15:21]
        |       |       |   |   |   `--COLON -> : [15:22]
        |       |       |   |   `--SLIST -> SLIST [16:21]
        |       |       |   |       |--EXPR -> EXPR [16:21]
        |       |       |   |       |   `--POST_INC -> ++ [16:21]
        |       |       |   |       |       `--IDENT -> i [16:20]
        |       |       |   |       `--SEMI -> ; [16:23]
        |       |       |   |--CASE_GROUP -> CASE_GROUP [17:16]
        |       |       |   |   |--LITERAL_CASE -> case [17:16]
        |       |       |   |   |   |--EXPR -> EXPR [17:21]
        |       |       |   |   |   |   `--NUM_INT -> 3 [17:21]
        |       |       |   |   |   `--COLON -> : [17:22]
        |       |       |   |   `--SLIST -> SLIST [18:21]
        |       |       |   |       |--EXPR -> EXPR [18:21]
        |       |       |   |       |   `--POST_INC -> ++ [18:21]
        |       |       |   |       |       |--SINGLE_LINE_COMMENT -> // [17:24]
        |       |       |   |       |       |   `--COMMENT_CONTENT ->  violation\r\n [17:26]
        |       |       |   |       |       `--IDENT -> i [18:20]
        |       |       |   |       `--SEMI -> ; [18:23]
        |       |       |   `--RCURLY -> } [19:12]
        |       |       `--RCURLY -> } [20:8]
        |       `--RCURLY -> } [21:4]
        `--RCURLY -> } [22:0]
```

```cmd
C:\checkstyle> java -jar checkstyle-10.21.4-all.jar  -c config.xml --exclude-regexp "Person.java"  C:\checkstyle\testCheckStyle\src
Starting audit...
[ERROR] C:\checkstyle\testCheckStyle\src\Test.java:17:17: Fall through from previous branch of the switch statement. [FallThrough]
Audit done.
Checkstyle ends with 1 errors.
```

cyclomatic complexity is the same for both functions (equal 4)
```java
 public class Person {

    private int age;
    private String name;
    private String lastName;
    private final int childLimit = 18;

    public void conditions(){
        if(age < 12 && lastName == "Doe" && name == "John"){
            age++;
        }
    }

    public void conditions2(){
        if(age < 12){
            if(lastName == "Doe"){
                if(name == "John"){
                    age++;
                }
            }

        }
    }

}

```