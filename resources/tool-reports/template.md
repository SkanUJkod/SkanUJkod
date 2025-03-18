# Report: [Tool Name]

### 1: User interface

- Describe how the tool is used to analyse an example project. A link to a tutorial or your own summary can be put here.
- Does the tool offer a CLI, or a GUI?
- How does it deal with configuration, passing parameters, etc.?
- Is there a way to explicitly specify the order in which the plugins/checks are executed?
- Can the tool be used in an on-line fashion, where it updates its metrics when it detects changes to the project? Or is it strictly offline, consuming the input project and outputing the analysis results once it finishes?

### 2: Available metrics

- This will be specific to the chosen tool. What kind of analyses does it specialise in? Highlight the ones you think can be most useful or interesting, and the ones you'd like our MVP to implement itself.

### 3: Extensibility

- We want our project to allow easily implementing additional custom checks. Describe how the chosen tools allow extending them by trying to add a "dummy" check. It should be very simple, only to describe the contact surface of the tool's core and your plugin.
- If there exists documentation describing how to extend the tool with your own check, link it here.
- Describe your attempt at extending the tool. What did you have difficulty with, and what turned out to be easy?
- Are the plugins linked statically (adding a check requires re-compiling the whole tool), or dynamically?

### 4: Architecture

Prepare a summary of the architecture of the tool. This is the most difficult part, and will probably involve inspecting at least a little bit of the source code.

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
   - If any other approach is taken (WASM, eBPF), summarise it and link to any relevant discussions between maintainers.
