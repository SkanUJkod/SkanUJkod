#### This doc aggregates resources we found useful while learning Rust.

## Basics

- [The Rust Programming Language](https://doc.rust-lang.org/book/)
  - The go-to resource for learning Rust.
- [Rust by Example](https://doc.rust-lang.org/rust-by-example/)
  - _Rust by Example (RBE) is a collection of runnable examples that illustrate various Rust concepts and standard libraries. To get even more out of these examples, don't forget to install Rust locally and check out the official docs._
- [Learn Rust the Dangerous Way](https://cliffle.com/p/dangerust/)
  - _LRtDW is a series of articles putting Rust features in context for low-level C programmers who maybe don’t have a formal CS background — the sort of people who work on firmware, game engines, OS kernels, and the like._

## Intermediate

- [Rust for C++ programmers](https://github.com/nrc/r4cppp)
  - _This tutorial is intended for programmers who already know how pointers and references work and are used to systems programming concepts such as integer widths and memory management. We intend to cover, primarily, the differences between Rust and C++ to get you writing Rust programs quickly without lots of fluff you probably already know._

## Design

- [Rust Design Patterns](https://rust-unofficial.github.io/patterns/)
  - _Rust is not object-oriented, and the combination of all its characteristics, such as functional elements, a strong type system, and the borrow checker, makes it unique. Because of this, Rust design patterns vary with respect to other traditional object-oriented programming languages. That’s why we decided to write this book. We hope you enjoy reading it!_

## Performance

- [The Rust Performance Book](https://nnethercote.github.io/perf-book/)
  - _This book contains techniques that can improve the performance-related characteristics of Rust programs, such as runtime speed, memory usage, and binary size. [...] This book is aimed at intermediate and advanced Rust users. Beginner Rust users have more than enough to learn and these techniques are likely to be an unhelpful distraction to them._

## Graphs

#### This section is included (and recommended) for contributors that will work with analyzers consuming control flow graphs.

- [Graphs and arena allocation](https://aminb.gitbooks.io/rust-for-c/content/graphs/) (part of rust4cpp)
- [Modeling graphs in Rust using vector indices](https://www.reddit.com/r/rust/comments/31o4wh/modeling_graphs_in_rust_using_vector_indices/)

### Contributing to this list

More resources are welcome, but mind the following:

- Keep it domain-specific, there are a lot of great resources like [The Embedded Rust Book](https://docs.rust-embedded.org/book/), but adding resources that are not directly applicable will quickly make the document bloated.
- Some links lead to 'proxy' sites like reddit or ycombinator, to also include comments from other developers. These can sometimes add additional insights. Consider this when contributing yourself.
