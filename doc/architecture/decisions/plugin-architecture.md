# Achieving Plugin Architecture

This doc describes the challenges we faced when implementing a plugin-based architecture in Rust, based largely on the knowledge from the _amazing_ series of [blog posts](https://nullderef.com/series/rust-plugins/) by [Mario Ortiz Manero](https://nullderef.com/about/). If you want to get a general grasp of the problems, we recommend reading it first. We will also refer to specific sections from it throughout this document, either for issues we've shared with the Tremor plugin system, or where our use-cases and solutions differed.

From the beginning the framework was supposed to target a plugin-based architecture, but there were a few options as to how this could be achieved. The two main approaches are dynamic linking and serialization based. The user-facing requirement was that adding a plugin did not require recompiling the core executable, which throws static linking out the window.

We will shortly go through the technologies considered by Manero, before going in-depth into dynamic linking.

## Scripting languages

This was considered briefly, but with much hope, as the kernel code was basically acting like a dynamic script, passing data to functions based on the IDs they communicate. The issue is that, AFAWK, a scripting language that allows calling Rust functions still needs to be aware of them in some meaningful way, like having access to its source code, and no scripting language exists that could load a Rust module compiled to a dynamic library file (`.so/.dylib/.dll`). It was also important for the script and the called functions to share the same runtime and not have to require serialization (which we will also go over), which unaviodably impacts performance.

Manero looks at the use of scripts from the opposite perspective, having the plugins be written in a scripting language. However, we preferred the analyses are implemented in high-performance/compiled languages as well, as they could become compute-heavy in the future.

## IPC

The graph at https://nullderef.com/blog/plugin-tech/#inter-process-communication neatly presents the three main options for IPC based plugins, and Manero describes their utility accurately. However, all of them use serialization which unavoidably impacts performance. The drawbacks of dynamic linking made us circle back to consider IPC again, but the end result seems to work. Further work could attempt a form of abstraction that allows easily changing the dynamic linking 'backend' to an IPC based one, since it should be straightforward to implement.

We note WASM based plugins in this section but they involve serialization as well, and are not as well-adopted as IPC.

## Dynamic linking

As Manero outlines in his series, dynamic linking in Rust is surprisingly hard due to a volatile ABI.

#### But what is actually an Application Binary Interface?

The term is very muddy and throughout our research there were many definitions. We'll save the more technical ones for the sake of more intuitive one-liners found on the internet.

> An ABI is a mapping from the execution model of the language to a particular machine/operating system/compiler combination. It makes no sense to define one in the language specification because that runs the risk of excluding C implementations on some architectures.

~ JeremyP - [SO Question: _Does C have a standard ABI?_](https://stackoverflow.com/questions/4489012/does-c-have-a-standard-abi) [...no!]

This tells us that a language's specification is decoupled from a compiler implementation of it. So ABI stabilization does not depend on the language, but the compiler that chooses to implement it, and the specific platform it targets. _Platform_ is also vague here, since it can encompass the OS, CPU architecture, etc. It is the `rustc` devs that choose to leave the ABI unstable to allow compiler optimizations that might break backwards compatibility.

> ABI can mean a lot of different things to different people. At the end of the day it’s a catch-all term for “implementation details that at least two things need to agree on for everything to work”. In this document we refer to ABI as covering type layout and how the different types/values are passed between C functions, as these are the aspects of the Rust ABI that are guaranteed and useful.
>
> There are additional details of Rust’s ABI which are currently unspecified and unstable, such as vtable layouts for trait objects, and how linker/debug symbols are mangled. It’s fine if that was gibberish to you, because you aren’t really allowed to care about those things right now! (Although that doesn’t necessarily stop people from trying…)

~ Aria Desires (faultlore) - [Notes on Type Layouts and ABIs in Rust](https://faultlore.com/blah/rust-layouts-and-abis/)

[Wikipedia](https://en.wikipedia.org/wiki/Application_binary_interface) also extends this two element list (type layout and calling conventions) but when thinking about ABI they seem to be the most important. Building on top of this understanding, ABI is basically the contract (interface) for one compiled program to call another compiled program's functions from its runtime. We will come back to this when discussing future refinements to the general linking model for Rust.

#### Why Rust's ABI is worse than unstable

I used `volatile` for a reason. It's not just unstable in the sense that you need the same version of the compiler to produce compatible binaries. Some layout optimizations are non-deterministic, and there's even an option to explicitly shuffle layouts for compiler testing. Two separate compiler runs can compile the same library source and produce different binaries.

This makes sense from a language developer standpoint, and explains why most modern languages capitalize on static linking almost everywhere. A stable ABI is only something a mature and adopted language can allow itself the luxury of. For new, shiny languages, even fundamental aspects like struct layouts can fluctuate, and sticking to a backwards-compatible ABI will make that more difficult in more ways than one.

#### C FFI

The standard way to overcome Rust's own ABI being unstable is to use the C "ABI"[^1] and make FFI calls to C dynlibs compiled from native Rust crates. This has many drawbacks and is particularly frustrating for Rust-to-Rust calls:

#### Type layouts

- Reduced set of FFI-safe types - Any type meant to be passed over the FFI boundary needs to have a C-conforming layout, specified via `#[repr(C)]` before the type definition. Due to the same reasons the Rust Compiler devs do not want a stable ABI, they also do not annotate most types from the standard library with `repr(C)`. This helps them to sometimes lay out data in a more optimal way than C does for whatever reason, but for our case it means having a clone of the standard library that pins its data representations to C's.

- Discriminated unions (Rust `enum`s) are especially hard to translate since they're not natively supported in C in the same way that `union`s are. Ubiquitous types like `Option<T>` are also candidates for zero-size optimization, which needs to be implemented by whatever wraps the low-level C interface. For an example of a library that targets discriminated unions explicitly, see https://github.com/ZettaScaleLabs/stabby.

#### repr(C) contd.

From [a reddit thread on `repr(C)`](https://www.reddit.com/r/rust/comments/qond4l/comment/hjo4yrw/):

> `repr(C)` does not mean FFI-safe -- it only indicates that the `struct` layout should follow C conventions: fields in the order they are written with padding in between if necessary.

It follows that any pinnable representation would work to provide a dependable ABI, it's just that the C ABI is the only one available.

TODO: When are `repr(C)` types NOT FFI-safe?

#### Type erasure

This is again neatly [discussed by Manero](https://nullderef.com/blog/plugin-dynload/#generics-in-plugins), and was already touched upon in the [model](./model.md) document. Plugins may be interdependent, to which the core is oblivious. In C you would model these data dependencies with a `void*` type. Guided by Manero's decision process on choosing the best dependency to wrap manual linking, we choose the `abi_stable` crate, which also seems to be the de-facto standard for higher level FFI, or Rust-to-Rust dynamic linking.

Modeling `void*`-like types in `abi_stable` is not straightforward. There are two types that can be utilised for it.

TODO: describe shortly

The problem with opaque types as they're implemented now is that downcasting is only expected to be done in the same library that produced the opaque type. This directly differs from Manero's requirements, where plugins were not interdependent. `abi_stable` is explicit about this both in its [docs](https://docs.rs/abi_stable/latest/abi_stable/struct.DynTrait.html#impl-DynTrait%3C'borr,+P,+I,+EV%3E-3) but even the error message it produces when one attempts to downcast in a different library.

There are two solutions we've tried:

- use `unchecked_downcast_*` instead. While this works, it's an `unsafe` call and makes NO metadata runtime checks. It is the approach used as of the time of writing this doc.
- Modify the crate to remove the [`executable_identity`](https://github.com/rodrimati1992/abi_stable_crates/blob/9966b8f0084fc768e3fb557bf81affea0b5868d8/abi_stable/src/std_types/utypeid.rs#L78) member from `UTypeID`. This looks to be the main metadata-carrying type, and it includes data other than `executable_identity`, which seems to correspond directly to the parent library. One can probably consider the field as a hash of the binary code of the library. This is more safe than the `unsafe_downcast_*` call because it performs _some_ runtime checks, save for the one possibly corresponding only to the origin library. However it is unclear what effect it has on the overall safeness and whether our linking model is still sound under such modification.

I've raised a question about this in the `abi_stable_crates` repo by reviving a seemingly [dead issue](https://github.com/rodrimati1992/abi_stable_crates/issues/35#issuecomment-2850448303). Unfortunately the library itself has not been updated in some time and no prominent forks came up to replace it.

#### Cargo "cdylib" bundling

Another caveat of using `abi_stable` is that it requires `crate-type = "cdylib"` to build the crates as C FFI compatible libraries. Though it's hard to verify from the official docs, it looks like this option produces a binary file that statically links ALL of its cargo dependencies, except perhaps ones like the standard library. For our model this introduces unnecessary redundancies. When `plugin2` depends on `plugin1`, and our pipeline wants to load both, `plugin1` will be loaded two times. This is an unfortunate trade-off, and it seems that only active advancements to the Rust ecosystem in dynamic linking can alleviate it. Which brings us to the final point.

#### The future

Of course in the process of researching dynamic linking possibilities for Rust I was also looking for a light in the tunnel to prove it won't always have to be such a pain. There are two that I believe will become viable in the near future:

- [`extern "crabi"`](https://github.com/rust-lang/compiler-team/issues/631) - This accepted proposal basically proposes to stabilise the ABI anyway as an opt-in. Discussions on the need for a stable ABI in the Rust spaces have been ongoing for a long time and while compiler devs have already gone on record to say this is not a priority, having accepted an experimental feature gate proposal gives the issue traction that seems to not have disappeared (https://github.com/rust-lang/rust/issues/111423). While `rustc` devs might not have interest in `crABI`, the community seems to, and most of the theoretical groundwork looks resolved by reading through the proposal.

- Type preserving compilation - I might be a crazy revolutionary in regards to this idea, but the fact I found it in two unrelated places, [once](https://www.youtube.com/watch?v=3yVc5t-g-VU) before even starting this project, and another([1](https://www.reddit.com/r/rust/comments/10cpt6b/we_dont_need_a_stable_abi/?rdt=46159),[2](https://blaz.is/blog/post/we-dont-need-a-stable-abi/)) time specifically in reference to Rust. It's exciting, novel, and in some ways necessary to help the modern languages that cannot afford to stabilize their ABI still be adoptable. For someone with a background in compilation theory, it might even be grounds for a thesis.

I also think a differential approach in analyzing the current global software development ecosystem will be beneficial, like a talk on `Implementing a plugin system in x, where x is your favorite programming language`.

[^1]: As a side note, most languages actually implement an "escape hatch" C FFI, even when it vastly violates their invariants. Haskell allows C FFI calls for low level operations to allow high-performance implementations of things like array-based data structures, even though it's supposed to be a pure language. Most interop is provided via the C FFI for multi-language project, as discussed in [this brilliant talk](https://www.youtube.com/watch?v=3yVc5t-g-VU) on high-level language interoperability (which we'll circle back to when considering future refinements). Rust isn't different in this sense by providing the `unsafe` keyword as a more general escape hatch from its ownership model, which is also required when using [`extern` functions](https://doc.rust-lang.org/book/ch20-01-unsafe-rust.html#using-extern-functions-to-call-external-code) for FFI.
