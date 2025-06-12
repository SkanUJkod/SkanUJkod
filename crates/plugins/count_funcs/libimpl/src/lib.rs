pub type CountFuncsResult = u32;

pub struct CountFuncsDeps<'a> {
    pub parse_file_result: &'a parse_file_lib::ParseFileResult,
}

pub struct CountFuncsParams {}

pub fn count_funcs(deps: CountFuncsDeps, _params: CountFuncsParams) -> CountFuncsResult {
    count_funcs_priv(&deps.parse_file_result.file)
}

fn count_funcs_priv(file: &go_parser::ast::File) -> u32 {
    file.decls
        .iter()
        .filter(|decl| matches!(decl, go_parser::ast::Decl::Func(_)))
        .map(|_| 1)
        .sum()
}
