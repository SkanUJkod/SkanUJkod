# Report: Frama-C

#### Disclaimer

Most of this report is based on the documentation available [here](https://www.frama-c.com/download/frama-c-plugin-development-guide.pdf). I will be referring to specific sections of the guide throughout the report. At the time of writing, the guide describes Frama-C 30.0 (Zinc).

### 1: User interface

Frama-C offers both a CLI and a GUI to run analyses. I have only used the CLI.

Configuration values for specific plugins are passed using named parameters, prefixed by the plugin "shortname". Calling `frama-c --help` lists all options of all registered plugins. Unnamed parameters refer to source files or directories to be analysed.

Some plugins have dependencies between each other, e.g. a plugin that computes the control flow graph needs to be run before a plugin that analyses this CFG. To ensure this, a `-then` parameter is used. Calling `frama-c -plugin1 -plugin2 -then -plugin3 -plugin4 -then -plugin5 ...` ensures that plugins 1 and 2 are executed before 3 and 4, which are executed before 5. Plugins in the same "section" can be thought to execute concurrently. Other `then*` operators are also available. For more details see section 2.4.3 of the [guide](#disclaimer).

While the tool offers "persistent" sessions that can be loaded once analysis is started, AFAIK, modifying the analysed source requires starting a new analysis.

### 2: Available metrics

The tool centers around formal methods, proving properties of C programs, etc.
A list of official plugins can be found [here](https://frama-c.com/html/kernel-plugin.html). However, such formal analyses are not mentioned by our project requirements.

Instead, I will focus on a suite of custom plugins extending the framework with code coverage functionalities. These are [LAnnotate](https://git.frama-c.com/pub/ltest/lannotate) and [LReplay](https://git.frama-c.com/pub/ltest/lreplay).

The LAnnotate [docs](https://git.frama-c.com/pub/ltest/lannotate/-/blob/master/doc/criteria.markdown?ref_type=heads) briefly describe the types of coverage metrics the plugin implements. While some new metrics are omitted from the doc, the most fundamental ones are there.

The plugin instruments the given source code by injecting lines calling some macros (`pc_label`) that will be tracked during test execution by LReplay. This isn't very comfortable when porting existing test suites to be instrumented, much less if a testing framework like GTest is already in use, which is why this plugin is mostly maintained for research purposes.

The coverage metrics our MVP should include would probably be:

- Decision Coverage - Tracks if each `if` node is executed, but not if all branches are executed
- Function Coverage and Function Call Coverage - The metrics track the % of functions executed by the suite and the % of function calls, respectively.

This is a conservative estimate. If overall efforts maintain the current development velocity, this list could be expanded.

### 3: Extensibility

The [guide](#disclaimer) goes into detail about plugin development. A plugin is basically a dune library, that calls the `Boot.Main.extend` function to register the plugin with the kernel. It is built with `dune build` and the resulting artifacts are moved to Frama-C's installed directory using `dune install`.

The guide describes two "dummy" plugins. One just logs "Hello, world!" to the console on each invocation of `frama-c`, while the other computes the CFG of an input program in .dot format. These are described by sections 2.3 and 2.4, respectively.

### 4: Architecture

![Frama-C architecture diagram](frama-c-plugin-development-guide.svg)

The above diagram visualizes the architecture, as well as the structure of the kernel repository.

<!-- As mentioned previously, plugins are linked dynamically.

1. How is the project structured?

   - This includes subdirectories of the source repository, modules of the actual code, etc.
   - Tools that generate UML diagrams from source code can be useful for this.

2. How are inputs represented?

   - Most tools should be able to analyse an entire project, but some also allow analysing single files.
   - For tools supporting multiple languages this is of particular interest, because they need to define an abstraction layer that defines the operations that can be performed on an input project, irrespective of the input project language. Describe how this is achieved.

3. Parsing

   - Does the tool implement its own lexing/parsing/semantic analysis? Which of these does it delegate to the compiler of the input language?
   - Semantic analysis is usually done immediately after parsing, to check context-sensitive information like whether variables are declared before they're used,for static languages, whether the types check, etc.
   - Implementing your own semantic analysis is hard, which is why a lot of static analysis tools are closely coupled with the corresponding compiler, e.g. clang-tidy, gcc's static analyser, etc.
   - We want to have native implementations for all of these steps, so if your tool does not use the compiler utilities for the above, you are free to go into more detail.

4. Linking checks

   - For plugin-oriented architectures especially, describe how specific projects implement communication between the kernel and the plugin.
   - Are the plugins linked statically, requiring re-compilation of the entire tool when adding a new plugin?
   - Are the plugins linked dynamically, as shared libraries, etc.? Do they use C interop for FFI, or native language utilities?
   - Are the plugins implemented using IPC? The known approaches is communicating using OS sockets, pipes, or shared memory. How is data flowing between plugins [de]serialized? Were there documented performance considerations?
   - If any other approach is taken (WASM, eBPF), summarise it and link to any relevant discussions between maintainers. -->
