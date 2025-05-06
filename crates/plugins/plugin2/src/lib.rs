use abi_stable::{rvec, std_types::RVec};
use plugin_interface::{BoxedInterface, ExampleLib, ExampleLib_Ref, PluginFunction};
use plugin1::StringBuilder;

use abi_stable::{DynTrait, export_root_module, prefix_type::PrefixTypeTrait, sabi_extern_fn};

#[export_root_module]
pub fn get_library() -> ExampleLib_Ref {
    ExampleLib {
        plugin_functions: new_pf_vec2,
    }
    .leak_into_prefix()
}

#[sabi_extern_fn]
fn new_pf_vec2() -> RVec<PluginFunction> {
    rvec![PluginFunction(new_pf2)]
}

fn new_boxed_interface2() -> BoxedInterface<'static> {
    DynTrait::from_value(StringBuilder {
        text: "plugin2::".into(),
        appended: rvec![],
    })
}

/// Appends a string to the erased `StringBuilder`.
#[sabi_extern_fn]
fn new_pf2(mut v: RVec<&mut BoxedInterface<'_>>) -> BoxedInterface<'static> {
    assert_eq!(v.len(), 1);
    dbg!(&v);
    let bi = &mut v[0];
    dbg!(&bi);
    let a = bi.downcast_as_mut::<StringBuilder>().unwrap();
    dbg!(&a);
    a.append_string("plugin2 was here!".into());

    new_boxed_interface2()
}
