use std::collections::HashMap;

pub type CountFuncsResult = usize;

pub struct CountFuncsDeps<'a> {
    pub parse_dir_result: &'a parse_dir_lib::ParseDirResult,
}

pub struct CountFuncsParams {}

pub fn count_funcs(deps: CountFuncsDeps, _params: CountFuncsParams) -> CountFuncsResult {
    count_funcs_priv(&deps.parse_dir_result.packages)
}

fn count_funcs_priv(packages: &HashMap<String, go_parser::ast::Package>) -> CountFuncsResult {
    packages
        .iter()
        .flat_map(|(_name, package)| package.files())
        .flat_map(|(_filename, boxed_file)| &boxed_file.decls)
        .filter(|decl| matches!(decl, go_parser::ast::Decl::Func(_)))
        .count()
}
