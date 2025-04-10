fn main() {
    parse_file();
}


fn parse_file() {
    let source = "package main";
    let mut fs = go_parser::FileSet::new();
    let o = &mut go_parser::AstObjects::new();
    let el = &mut go_parser::ErrorList::new();
    let (p, ast) = go_parser::parse_file(o, &mut fs, el, "./main.go", source, false);
    // print!("{}", p.get_errors());
    println!("{:?}", p.get_errors());
    println!("{:?}", o.idents.vec());


    for ao in o.fdecls.vec() {
        println!("{:?}", ao);
    }
}