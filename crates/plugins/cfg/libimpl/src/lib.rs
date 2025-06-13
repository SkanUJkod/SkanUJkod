use std::collections::HashMap;

pub mod builders;
mod export;
mod optimization;
pub mod types;
mod validation;

use types::ControlFlowGraph;

#[derive(std::cmp::PartialEq, std::cmp::Eq, std::hash::Hash, std::fmt::Debug)]
pub struct CfgKey {
    pub filename: String,
    pub funcname: String,
}

#[derive(std::fmt::Debug)]
pub struct CreateProjectCfgsResult {
    result: HashMap<CfgKey, String>,
}

impl std::fmt::Display for CreateProjectCfgsResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // TODO
        Ok(())
    }
}

pub struct CreateProjectCfgsDeps<'a> {
    pub parse_dir_result: &'a parse_dir_lib::ParseDirResult,
}

pub struct CreateProjectCfgsParams {}

pub fn create_project_cfgs(
    deps: CreateProjectCfgsDeps,
    _params: CreateProjectCfgsParams,
) -> CreateProjectCfgsResult {
    create_project_cfgs_priv(deps.parse_dir_result)
}

fn create_project_cfgs_priv(ctx: &parse_dir_lib::ParseDirResult) -> CreateProjectCfgsResult {
    let cfg_map = ctx
        .packages
        .iter()
        .inspect(|a| {
            dbg!(a);
        })
        .flat_map(|(_name, package)| package.files())
        .inspect(|a| {
            dbg!(a);
        })
        .flat_map(|(filename, boxed_file)| {
            boxed_file.decls.iter().map(|decl| (filename.clone(), decl))
        })
        .inspect(|a| {
            dbg!(a);
        })
        .filter_map(|(filename, decl)| match decl {
            go_parser::ast::Decl::Func(fk) => {
                Some((filename.clone(), &ctx.ast_objects.fdecls[*fk]))
            }
            _ => None,
        })
        .inspect(|a| {
            dbg!(a);
        })
        .map(|(filename, fdecl)| {
            let cfg = ControlFlowGraph::build(&ctx.fileset, fdecl, &ctx.ast_objects);
            let funcname = &ctx.ast_objects.idents[fdecl.name].name;
            (
                CfgKey {
                    filename,
                    funcname: funcname.clone(),
                },
                cfg,
            )
        })
        .inspect(|a| {
            dbg!(a);
        })
        .map(|(key, cfg)| {
            let funcname = key.funcname.clone();
            (key, export::dot::to_dot(&cfg, funcname.as_str()))
        })
        .inspect(|a| {
            dbg!(a);
        })
        .collect();

    CreateProjectCfgsResult { result: cfg_map }
}
