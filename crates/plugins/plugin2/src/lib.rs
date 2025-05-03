use plugin_interface::{BoxedInterface, ExampleLib, ExampleLib_Ref};

use plugin1::StringBuilder;

use abi_stable::{
    DynTrait, export_root_module, prefix_type::PrefixTypeTrait, sabi_extern_fn, std_types::RString,
};

/// The function which exports the root module of the library.
///
/// The root module is exported inside a static of `LibHeader` type,
/// which has this extra metadata:
///
/// - The abi_stable version number used by the dynamic library.
///
/// - A constant describing the layout of the exported root module,and every type it references.
///
/// - A lazily initialized reference to the root module.
///
/// - The constructor function of the root module.
///
#[export_root_module]
pub fn get_library() -> ExampleLib_Ref {
    ExampleLib {
        new_boxed_interface,
        append_string,
    }
    .leak_into_prefix()
}

/// Constructs a BoxedInterface.
#[sabi_extern_fn]
fn new_boxed_interface() -> BoxedInterface<'static> {
    DynTrait::from_value(StringBuilder {
        text: "".into(),
        appended: vec![],
    })
}

/// Appends a string to the erased `StringBuilder`.
#[sabi_extern_fn]
fn append_string(wrapped: &mut BoxedInterface<'_>, string: RString) {
    wrapped
        .downcast_as_mut::<StringBuilder>() // Returns `Result<&mut StringBuilder, _>`
        .unwrap() // Returns `&mut StringBuilder`
        .append_string(string);
}
