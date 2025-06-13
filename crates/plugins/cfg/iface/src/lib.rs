use abi_stable::{rvec, std_types::RVec};
use cfg_lib::{CreateProjectCfgsDeps, CreateProjectCfgsParams};
use parse_dir_lib::ParseDirResult;
use plugin_interface::{
    BoxedPFResult, PFConnector, PFDependencies, PFType, Plugin, Plugin_Ref, PluginFunction,
    QualPFID, UserParameters,
};

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
                plugin_id: "parse_dir_plugin".into(),
                pf_id: "parse_dir".into()
            }],
            user_params: rvec![]
        },
        pf_id: QualPFID {
            plugin_id: "cfg_plugin".into(),
            pf_id: "build_project_cfgs".into()
        }
    }]
}

#[sabi_extern_fn]
fn new_pf2(pf_results: PFDependencies, user_params: &UserParameters) -> BoxedPFResult<'static> {
    dbg!(&pf_results);
    dbg!(&user_params);
    assert_eq!(pf_results.len(), 1);

    let boxed_parse_result = pf_results
        .get(&QualPFID {
            plugin_id: "parse_dir_plugin".into(),
            pf_id: "parse_dir".into(),
        })
        .unwrap();
    let parse_dir_result = unsafe { boxed_parse_result.unchecked_downcast_as::<ParseDirResult>() };

    let cfgs = cfg_lib::create_project_cfgs(
        CreateProjectCfgsDeps { parse_dir_result },
        CreateProjectCfgsParams {},
    );
    dbg!(&cfgs);

    DynTrait::from_value(cfgs)
}
