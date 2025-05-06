use abi_stable::{StableAbi, rvec, std_types::RVec};
use plugin_interface::{BoxedInterface, ExampleLib, ExampleLib_Ref, PluginFunction};
use std::fmt::{self, Display};

use abi_stable::{
    DynTrait, export_root_module, prefix_type::PrefixTypeTrait, sabi_extern_fn, std_types::RString,
};

#[export_root_module]
pub fn get_library() -> ExampleLib_Ref {
    ExampleLib {
        plugin_functions: new_pf_vec,
    }
    .leak_into_prefix()
}

/// `DynTrait<_, TheInterface>` is constructed from this type in this example
#[derive(Debug, Clone, StableAbi)]
#[repr(C)]
pub struct StringBuilder {
    pub text: RString,
    pub appended: RVec<RString>,
}

impl Display for StringBuilder {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&self.text, f)
    }
}

impl StringBuilder {
    /// Appends the string at the end.
    pub fn append_string(&mut self, string: RString) {
        self.text.push_str(&string);
        self.appended.push(string);
    }
}

#[sabi_extern_fn]
fn new_pf_vec() -> RVec<PluginFunction> {
    rvec![PluginFunction(new_pf)]
}

fn new_boxed_interface() -> BoxedInterface<'static> {
    DynTrait::from_value(StringBuilder {
        text: "plugin1::".into(),
        appended: rvec![],
    })
}

/// Appends a string to the erased `StringBuilder`.
#[sabi_extern_fn]
fn new_pf(v: RVec<&mut BoxedInterface<'_>>) -> BoxedInterface<'static> {
    dbg!(v);
    new_boxed_interface()
}
