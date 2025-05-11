# Conceptual model

The SkanUJKod general-purpose static analysis framework is planned to consist of the _core_ (or _kernel_, used interchangeably) and a number of plugins. The base framework should provide some plugins, but may be extended with user developed plugins, offering additional analyses.

Pipeline architecture is targeted. This means a single "run" of the framework consumes a project and runs analyses specified by the user in some order. This means the application is fully ephemeral, e.g. no state or data has to be stored persistently, and no on-line analysis is supported.

Some helpful features directly contradict this assumption and the pipeline architecture, like intermediate results caching or integration into IDEs for quickly re-running analyses invalidated by user changes (online model). Attempts can be made to accomodate such features in the future, but it is not planned now and would incur more or less changes to the overall model. In short, an attempt at supporting for example caching, should take the entire framework structure into consideration.

## Plugin model

A plugin is modeled like a set of _mathematical_ functions with some additional constraints, hereafter referred to as _plugin functions_. The constraints for a plugin function `f` are as follows:

1. [PF Output](#pf-output): The type of the output result of `f` is a subtype of some general type `R`, broadly denoting plugin function result.
1. [PF Inputs](#pf-inputs): An input of `f` falls into one of two categories:
   1. The result of another plugin, e.g. another subtype of `R`.
   1. A parameter specified on a per-run basis by the user.
1. [Referential Transparency](#pf-reftran): `f` should be referentially transparent.

[Composition Operators](#composition-operators): To allow ad-hoc composition, a "vertical" and a "horizontal" composition operator are planned, but may be dropped if they prove to be unnecessary.

## Pipeline topology

A singular analysis "run" of the framework is a pipeline of plugin functions, executed in an order either specified by the user, or inferred by the core. This order can be modeled by a directed graph, which we shall also refer to as the "execution plan".

Some key assumptions:

- [Granularity](#granularity): The granularity of a plugin function should be high enough to allow modularity and reusability, but low enough to discourage complicated execution plans.
- [Acyclicity](#acyclicity): The execution plan must not contain cycles, e.g. it forms a DAG.

#### Granularity

##### PFs are simple

Plugin functions should be small enough to allow re-use, e.g. wherever it makes sense to separate an analysis, a plugin developer should do so, to allow the user to compose the analyses in their own manner.

For example, parsing and semantic analysis should be separate. An implementor may instead expose a parsing PF and a semantic analysis PF, and have the latter take as input the output of the former.

The implementor may still choose to export as a PF such composition of the two, which would roughly look like:

```rust
fn parse_and_type_pf(plugin_dependencies: ParseProjectDeps, user_parameters: &UserParameters) -> TypedAST {
   let parsed_project_result = parse_project_pf(plugin_dependencies, &user_parameters);
   compute_typed_ast_pf(ComputeTypedASTDeps{parsed_project: parsed_project_result}, &user_parameters)
}
```

Assuming all relevant entities are defined.

The implementor should not bloat the plugin interface with all PFs combined this way, only combinations for which it makes sense should be exposed.

##### PFs are coarse

A plugin function should not do little enough so as to require being called in a loop or a variable amount of times. To explain the origin of this restriction, let's consider how an end user would specify a custom execution plan.

We can generally assume the execution plan will be defined using a type of Domain Specific Language, in particular a graphical environment akin to a flowchart maker. A PF that parses a single source file, for example, would have to be called with different arguments, and a variable amount of times. To express this using a DSL, we would need at the very least a looping construct, not to mention other higher order control flow constructs. This brings the DSL very close towards a fully-fledged programming language, which is a common pitfall for any DSL.

One guideline to avoid developing PFs with this problem is to consider the inputs and whether we can write a "plural" version of it. For our example this would be a PF that takes as input a list of source files, and "maps" the original PF over it in an element-wise manner.

As a last resort, writing your own plugin is always an option. It should be easy enough to implement a PF composing two other PFs, but it would be best to have to write no code at all.

#### Acyclicity

Some static analysis frameworks allow mutual plugin dependencies, creating cycles in the execution plan graph. For an example of this, see Frama-C's [Plugin Development Guide](https://git.frama-c.com/pub/pub.frama-c.com/-/blob/master/download/plugin-development-guide-30.0-Zinc.pdf?ref_type=heads), doccumented by section _4.8.2 Dynamic Registration and Access_ at the time of writing this doc (version 30.0 Zinc).

For this framework, we decided this is an unnecessary feature at the present. There are rarely situations where this kind of dependency is required, further so under the conceptual model of PFs we employ. Allowing cycles in the execution plan graph would also complicate implementing features like parallel execution, which in our opinion would provide more value.

## Framework core

#### Responsibilities

The framework, in general, should be responsible for the following:

1. [Orchestration](#core-orchestration)
1. [Plugin loading](#core-plugin-loading)
1. [Data flow](#core-dataflow)

This list is very lean due to the underlying approach. A program that's supposed to call some functions in order does not make very complicated software. The complexity is hidden in one of two areas: PF design guidelines and implementation details.

##### Hidden complexity

###### PFs: Convention over Validation

Some guidelines when designing plugins and plugin functions were already mentioned in the [Granularity](#granularity) section. More of them follow in the [Plugin Functions](#plugin-functions) section.

Most of these guidelines cannot be enforced (validated) statically and thus fall under the category of 'conventions'. The efficacy of the entire framework hinges on each component following said conventions. Otherwise, user experience deteriorates and easy things become hard. To keep track of each 'convention', we've enumerated the most important ones in [Conventions](#conventions).

###### Implementation details

Leading ideas that influenced the conceptual model described by this document were type safety and simplicity, from a functional programming paradigm's perspective. Initially, a plugin was a regular function - thus the core was simply a procedure that aggregates the plugins and executes them, in an order inferred from the dependencies among them. The shift to 'a plugin is a _set_ of functions' was straightforward.

Unfortunately, while a plugin-oriented architecture, at least in Rust, can be realised in many ways, none make the effort to model operating on such dynamically loaded functions comfortable enough. The execution order inference based on input types alone is probably still impossible to implement, and may be difficult even for a language like Haskell. AFAIK, shared libraries and their analogues do not retain type information so a framework that requires plugins loaded in any way other than static must implement its core in a type-unsafe manner.

More on dynamic linking and its trade-offs is documented in the [Plugin loading](#core-plugin-loading) section.

##### Orchestration

The kernel is responsible for the order in which PFs are executed. Assuming it has access to how the plugins broadly depend on one another, it can build an execution plan DAG by itself. Sorting the graph topologically offers one of many possible orders of execution. Then, once it has a properly ordered sequence of PFs, it can be "folded", leaving out the responsibility of ensuring proper data flow.

If and when parallel execution becomes attainable, this approach should be easy enough to modify.

Basic logging should be supported at this stage to inform the user of when any specific analysis starts and ends. Diagnostics like performance can also be gathered in this section.

##### Plugin loading

There are many good [reasons why](choose-rust.md) Rust was chosen as the implementing language of the framework.

However, what surprised us was the immature state of support for dynamic linking in Rust - as of version 1.85, Rust still doesn't offer a stable ABI, or a standard way to layout data in memory. Because of various optimisations the compiler employs, it may decide, even between two runs of the same `rustc` binary, to order the fields of a struct in a different way.

To overcome this, software that needs to link dynamically annotates data that has to pass the "dynamic" boundary with `[#repr(C)]`, to force C-compliant layout, and then uses C FFI for dynamic library calls. There are some crates that help with the process, but none are very widely used, and some of the time other approaches to implement plugins are used, like using serialization.

###### Why not serialize?

At this point it is still unclear how error-resilient dynamic linking in Rust is. We might pivot to a serialization based approach like OS pipes that allow recovery from wrongly formatted data, at the cost of more code from the plugin implementor side. The current argument against serialization based approaches is one of performance. Say there are two independent PFs that require the AST as a dependency. Passing such data twice through separate pipes can already prove to be inefficient.

POCs of both approaches are under way.

###### Type erasure

While it is easy for one plugin to specify an input from another dependent plugin and explicitly annotate its type, this information is unavailable to the kernel code. Mentioned briefly in [Implementation details](#implementation-details), the binary shared libraries would have to retain type info. This idea has recently started getting traction, as seen [here](https://www.youtube.com/watch?v=3yVc5t-g-VU) and [here](https://blaz.is/blog/post/we-dont-need-a-stable-abi/).

However, the kernel code still has to be generic to whatever types the PFs want to juggle around, so any type checking has to be done at runtime. Plugin developers should employ caution when downcasting the results coming from other plugins, because the dependency plugin bundled in the dependent plugin might have a version different from the same plugin used as the shared object library.

##### Data flow

As mentioned previously, type erasure in the kernel will make it impossible to utilise type info about the underlying data. The kernel thus becomes agnostic to the data, and might as well treat it as raw binary buffers. Since a common interface will have to be defined to model the plugins, the PF Result type can be realised using a trait, and this is how the data will be stored, possibly in a map from PF name to result.

Executing a PF will involve passing the required subset of this data map, inspired by TS's structural subtyping approach. A subset of the map in the kernel could be a record/struct of named fields in the PF.

Rust actually relies on composition and serialization to achieve a flavour of [structural subtyping](https://nickb.dev/blog/a-workaround-for-rusts-lack-of-structural-subtyping/). This could be useful for a serialization-based linking approach, but is still useful conceptually for the dynamic linking model.

Another approach would be functional optics, but this has the same disadvantage as choosing a functional language for development - the semantics aren't clear to developers who've yet to discover functional programming.

###### Error handling

What happens if the representation of a datum differs between the kernel and a PF? Here we need to rely on whatever backend allows dynamic calls, and whether it allows adding hooks when it detects an error. It is still too early for us to know how this will be handled.

Another case of this is when a plugin fails. Does execution of independent PFs continue? Ideally, yes. This will have to be handled by the kernel implementation.

## Plugin Functions

This section delves more into the requirements and recommendations of design and development of a PF, from a perspective of the _kernel_ developer, as well as a third-party plugin developer.

### PF Output

The result type must not put too much limitation on the plugin developer, like constraining the format to JSON. Coincidentally, to avoid problems with inter-dependent data in a runtime that not much can be assumed about, the result type must be encodable in an unambiguous manner. With that in mind, we can more exactly constrain as follows:

- The common plugin result type is a type that _is able to_ implement `std::any::Any`. This roughly means a type that fully owns all of its constituent data (no references that aren't also `'static`).
- This constraint may be extended to only allow serializable types in the future.
- All additional constraints that come from the mechanism used to implement dynamic linking.

### PF Inputs

We've already [mentioned](#plugin-model), a plugin function expects inputs that fall into one of two groups: other plugin results or user parameters.

While there is reason to assume otherwise in the future, for now all inputs need to have a corresponding, unique name. This makes sense for plugin results, but is not so straightforward for user parameters.

#### Parameter uniqueness

We could expect the framework user to define one constant parameter and pass it to multiple plugin functions. This requires consensus among the PFs on how they communicate their requirement of this parameter to the kernel. Orthogonally, it makes sense to delegate the needed documentation of the parameter to dependent plugins. There is no obvious way to have our cake and eat it too, as some plugin functions may interpret the common parameter differently, or even expect it to be a fundamentally different type (int vs string). Other frameworks deal with this issue by supporting both types of parameters via different utilities. Parameters that are expected to be unique are, as expected, registered to the kernel and information like a `--help` description are offered by the plugin. Parameters expected to be shared may be passed as environment variables, or are fundamental enough to be exposed via some global kernel function, like the path to the project to be analysed. However, we'd like to avoid the second approach in the general design, and will touch upon the reason why later in this section.

Another nice property of this "name uniqueness" is a unique execution plan, which the kernel can infer by simply linking the names together. A cycle can still be detected, which we can break by, for example, excluding any offending plugin function to turn the execution plan back into a DAG.

This will probably change in the future, when support for another target language and multi-language project analysis is added. We could imagine two parsing PFs and one that aggregates them by taking the language ID as a parameter. The user would then like to parametrise the same PF and have it appear twice in the execution plan.

#### Input Data Structures

The following assumptions can be made:

- The order of arguments is not important
- Each argument should have a unique name (established in [_Parameter uniqueness_](#parameter-uniqueness))
- The PF knows the exact set of inputs at compile time.

This actually makes a `struct` the perfect data structure to represent the set of inputs. The fields are named, and it doesn't have the overhead of a map-like structure. Unfortunately, the kernel code must employ a dynamic structure, since the set of PFs is dynamic itself, which is why some sort of conversion would have to be implemented (like a custom trait and a derive macro). Research can be made if there's a map DS that allows fast "subset extraction".

### Referential transparency

The above property roughly translates to "same inputs produce the same output". This unfortunately cannot be enforced through design choices, and would probably be very painful to verify statically. Many factors could influence the transparency of a PF, the most obvious of which being the underlying operating system.

It's enough for a PF to read an environment variable to violate referential transparency, as two executors could provide a different value, and produce a different output. The simplest solution is to refactor the read variable into a user parameter, if it's available at the start of the pipeline.

Referential transparency is most often mentioned along with function purity, and pure functions are a subset of referentially transparent functions. Purity means no side effects occur, so things like writing to a file, performing a network request, etc. violates purity. Sandboxing can help by, for example, intercepting all OS calls, but requires a lot of effort. Purity in general could inconvenience plugin developers and there are valid usecases, like dumping results to a file.

### Consequences of the design

A very nice property of the model above is explicit flow of data. To our knowledge, our framework is currently the only one that prioritises said property to such extent. Because static analysis is a mostly offline effort, we think a "functional" approach made sense. Actual benefits of this will be studied once the framework takes up a more concrete shape and plugin developers share their feedback.

In summary, the general shape of a plugin function is `fn example_pf(ExampleDeps, ExampleUserParams) -> ExampleResult`, where `ExampleDeps` and `ExampleUserParams` are both `struct`s.
