use abi_stable::{
    DynTrait, export_root_module, prefix_type::PrefixTypeTrait, rvec, sabi_extern_fn,
    std_types::RString, std_types::RVec,
};
use plugin_interface::{
    BoxedPFResult, BoxedUserParam, PFConnector, PFDependencies, PFType, Plugin, Plugin_Ref,
    PluginFunction, QualPFID, UserParameters,
};

use git_metrics_lib::{
    CommitsByAuthorDeps,
    CommitsByAuthorParams,
    //  ContributorsInTimeframeDep,
    // ContributorsInTimeframeParams, FirstLastCommitDep, FirstLastCommitParams, LinesAddedRemovedDep,
    // LinesAddedRemovedParams,
    ReadRepoDeps,
    ReadRepoParams,
    ReadRepoResult,
    // TotalCommitPercentageDep, TotalCommitPercentageParams,
};

#[export_root_module]
pub fn get_library() -> Plugin_Ref {
    Plugin { funcs: new_pf_vec }.leak_into_prefix()
}

#[sabi_extern_fn]
fn new_pf_vec() -> RVec<PFConnector> {
    rvec![
        PFConnector {
            pf: PluginFunction(read_repo),
            pf_type: PFType {
                pf_dependencies: rvec![],
                user_params: rvec!["project_path".into()]
            },
            pf_id: QualPFID {
                plugin_id: "git_metrics_plugin".into(),
                pf_id: "read_repo".into()
            }
        },
        PFConnector {
            pf: PluginFunction(commits_by_author),
            pf_type: PFType {
                pf_dependencies: rvec![QualPFID {
                    plugin_id: "git_metrics_plugin".into(),
                    pf_id: "read_repo".into()
                }],
                user_params: rvec![]
            },
            pf_id: QualPFID {
                plugin_id: "git_metrics_plugin".into(),
                pf_id: "commits_by_author".into()
            }
        },
    ]
}

#[sabi_extern_fn]
fn read_repo(pf_results: PFDependencies, user_params: &UserParameters) -> BoxedPFResult<'static> {
    dbg!(&pf_results);
    dbg!(&user_params);

    let boxed_project_path: &BoxedUserParam = user_params.get("project_path").unwrap();
    let project_path = unsafe { boxed_project_path.unchecked_downcast_as::<RString>() };

    let result = git_metrics_lib::read_repo(
        ReadRepoDeps {},
        ReadRepoParams {
            dir: project_path.clone().into(),
        },
    );

    DynTrait::from_value(result)
}

#[sabi_extern_fn]
fn commits_by_author(
    pf_results: PFDependencies,
    user_params: &UserParameters,
) -> BoxedPFResult<'static> {
    dbg!(&pf_results);
    dbg!(&user_params);

    let boxed_read_repo = pf_results
        .get(&QualPFID {
            plugin_id: "git_metrics_plugin".into(),
            pf_id: "read_repo".into(),
        })
        .unwrap();
    let read_repo = unsafe { boxed_read_repo.unchecked_downcast_as::<ReadRepoResult>() };

    let result = git_metrics_lib::commits_by_author(
        CommitsByAuthorDeps { read_repo },
        CommitsByAuthorParams {},
    );

    DynTrait::from_value(result)
}

// #[sabi_extern_fn]
// fn contributors_in_timeframe(
//     pf_results: PFDependencies,
//     user_params: &UserParameters,
// ) -> BoxedPFResult<'static> {
//     dbg!(&pf_results);
//     dbg!(&user_params);
//     assert!(pf_results.is_empty());
//     assert_eq!(user_params.len(), 1);

//     let boxed_project_path: &BoxedUserParam = user_params.get("project_path").unwrap();
//     let project_path = unsafe { boxed_project_path.unchecked_downcast_as::<RString>() };

//     let result = git_metrics_lib::contributors_in_timeframe(
//         ContributorsInTimeframeDep {},
//         &ContributorsInTimeframeParams {
//             dir: project_path.clone(),
//         },
//     );

//     DynTrait::from_value(result)
// }

// #[sabi_extern_fn]
// fn total_commit_percentage(
//     pf_results: PFDependencies,
//     user_params: &UserParameters,
// ) -> BoxedPFResult<'static> {
//     dbg!(&pf_results);
//     dbg!(&user_params);
//     assert!(pf_results.is_empty());
//     assert_eq!(user_params.len(), 1);

//     let boxed_project_path: &BoxedUserParam = user_params.get("project_path").unwrap();
//     let project_path = unsafe { boxed_project_path.unchecked_downcast_as::<RString>() };

//     let result = git_metrics_lib::total_commit_percentage(
//         TotalCommitPercentageDep {},
//         &TotalCommitPercentageParams {
//             dir: project_path.clone(),
//         },
//     );

//     DynTrait::from_value(result)
// }

// #[sabi_extern_fn]
// fn first_last_commit(
//     pf_results: PFDependencies,
//     user_params: &UserParameters,
// ) -> BoxedPFResult<'static> {
//     dbg!(&pf_results);
//     dbg!(&user_params);
//     assert!(pf_results.is_empty());
//     assert_eq!(user_params.len(), 1);

//     let boxed_project_path: &BoxedUserParam = user_params.get("project_path").unwrap();
//     let project_path = unsafe { boxed_project_path.unchecked_downcast_as::<RString>() };

//     let result = git_metrics_lib::first_last_commit(
//         FirstLastCommitDep {},
//         &FirstLastCommitParams {
//             dir: project_path.clone(),
//         },
//     );

//     DynTrait::from_value(result)
// }

// #[sabi_extern_fn]
// fn lines_added_removed(
//     pf_results: PFDependencies,
//     user_params: &UserParameters,
// ) -> BoxedPFResult<'static> {
//     dbg!(&pf_results);
//     dbg!(&user_params);
//     assert!(pf_results.is_empty());
//     assert_eq!(user_params.len(), 1);

//     let boxed_project_path: &BoxedUserParam = user_params.get("project_path").unwrap();
//     let project_path = unsafe { boxed_project_path.unchecked_downcast_as::<RString>() };

//     let result = git_metrics_lib::lines_added_removed(
//         LinesAddedRemovedDep {},
//         &LinesAddedRemovedParams {
//             dir: project_path.clone(),
//         },
//     );

//     DynTrait::from_value(result)
// }
