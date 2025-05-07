use abi_stable::{rvec, std_types::RVec};
use plugin_interface::{BoxedPFResult, BoxedUserParam, Plugin, Plugin_Ref, PluginFunction};
use plugin1::ParseResult;

use abi_stable::{DynTrait, export_root_module, prefix_type::PrefixTypeTrait, sabi_extern_fn};

#[export_root_module]
pub fn get_library() -> Plugin_Ref {
    Plugin { funcs: new_pf_vec2 }.leak_into_prefix()
}

#[sabi_extern_fn]
fn new_pf_vec2() -> RVec<PluginFunction> {
    rvec![PluginFunction(new_pf2)]
}

/// Appends a string to the erased `StringBuilder`.
#[sabi_extern_fn]
fn new_pf2(
    pf_results: RVec<&mut BoxedPFResult<'_>>,
    user_params: RVec<&BoxedUserParam<'_>>,
) -> BoxedPFResult<'static> {
    dbg!(&pf_results);
    dbg!(&user_params);

    assert_eq!(pf_results.len(), 1);
    let parse_result = unsafe { pf_results[0].unchecked_downcast_as::<ParseResult>() };
    let count_res = count_funcs(&parse_result.file);
    dbg!(count_res);

    DynTrait::from_value(count_res)
}

fn count_funcs(file: &go_parser::ast::File) -> u32 {
    file.decls
        .iter()
        .filter(|decl| matches!(decl, go_parser::ast::Decl::Func(_)))
        .map(|_| 1)
        .sum()
}
