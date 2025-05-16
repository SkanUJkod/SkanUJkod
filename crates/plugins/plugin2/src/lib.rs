use abi_stable::{rvec, std_types::RVec};
use plugin_interface::{
    BoxedPFResult, PFConnector, PFDependencies, PFType, Plugin, Plugin_Ref, PluginFunction,
    QualPFID, UserParameters,
};
use plugin1::ParseResult;

use abi_stable::{DynTrait, export_root_module, prefix_type::PrefixTypeTrait, sabi_extern_fn};

#[export_root_module]
pub fn get_library() -> Plugin_Ref {
    Plugin { funcs: new_pf_vec2 }.leak_into_prefix()
}

#[sabi_extern_fn]
fn new_pf_vec2() -> RVec<PFConnector> {
    rvec![PFConnector {
        pf: PluginFunction(new_pf2),
        pf_type: PFType {
            pf_dependencies: rvec![QualPFID {
                plugin_id: "plugin1".into(),
                pf_id: "parse_file".into()
            }],
            user_params: rvec![]
        },
        pf_id: QualPFID {
            plugin_id: "plugin2".into(),
            pf_id: "count_funcs".into()
        }
    }]
}

/// Appends a string to the erased `StringBuilder`.
#[sabi_extern_fn]
fn new_pf2(pf_results: PFDependencies, user_params: &UserParameters) -> BoxedPFResult<'static> {
    dbg!(&pf_results);
    dbg!(&user_params);
    assert_eq!(pf_results.len(), 1);

    let boxed_parse_result = pf_results
        .get(&QualPFID {
            plugin_id: "plugin1".into(),
            pf_id: "parse_file".into(),
        })
        .unwrap();
    let parse_result = unsafe { boxed_parse_result.unchecked_downcast_as::<ParseResult>() };
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
