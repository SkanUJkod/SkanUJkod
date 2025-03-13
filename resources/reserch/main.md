
![example graphs](metricslevel.png)

we can messure static metrics on diffrent level 




https://www.sonarsource.com/learn/cyclomatic-complexity/

cyclomatic complexity will be messure on method level.

Cyclomatic complexity – minimum numer of paths that you need to test to ensure each decision point is executed at least once. 
Formula C = E – N + 2P 

![example graphs](img1.png)

Higher cyclomatic complexity means that we need cover more paths. More paths can cause higher probability of bugs. 
Every decision point (loop or condition statement) contribute to a new path. 


```python
    if a: 
        if b: 
            if c: 
                #statement

    if a and b and c: 
        #statement 
```

Cyclomatic complexity code analysis involves reviewing your program’s source code to understand its structure and identify areas where the complexity can be reduced.

calculating cyclomatic complexity for each function or module.



https://github.com/mauricioaniche/ck



LOC (Lines of code): It counts the lines of count, ignoring empty lines and comments (i.e., it's Source Lines of Code, or SLOC). The number of lines here might be a bit different from the original file, as we use JDT's internal representation of the source code to calculate it

DIT (Depth Inheritance Tree): It counts the number of "fathers" a class has. All classes have DIT at least 1 (everyone inherits java.lang.Object). In order to make it happen, classes must exist in the project (i.e. if a class depends upon X which relies in a jar/dependency file, and X depends upon other classes, DIT is counted as 2).


CBO (Coupling between objects): Counts the number of dependencies a class has. The tools checks for any type used in the entire class (field declaration, method return types, variable declarations, etc). It ignores dependencies to Java itself (e.g. java.lang.String).

FAN-IN: Counts the number of input dependencies a class has, i.e, the number of classes that reference a particular class. For instance, given a class X, the fan-in of X would be the number of classes that call X by referencing it as an attribute, accessing some of its attributes, invoking some of its methods, etc.

FAN-OUT: Counts the number of output dependencies a class has, i.e, the number of other classes referenced by a particular class. In other words, given a class X, the fan-out of X is the number of classes called by X via attributes reference, method invocations, object instances, etc.

NOC (Number of Children): It counts the number of immediate subclasses that a particular class has. 

LOC (Lines of code): It counts the lines of count, ignoring empty lines and comments (i.e., it's Source Lines of Code, or SLOC). The number of lines here might be a bit different from the original file, as we use JDT's internal representation of the source code to calculate it

