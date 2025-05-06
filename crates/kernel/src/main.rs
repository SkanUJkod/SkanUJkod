use std::path::PathBuf;

use abi_stable::rvec;

use abi_stable::library::lib_header_from_path;
use plugin_interface::ExampleLib_Ref;
fn main() {
    let paths = [
        "target/debug/libplugin1.dylib",
        "target/debug/libplugin2.dylib",
    ];

    let libs: Vec<ExampleLib_Ref> = paths
        .iter()
        .map(std::convert::Into::<PathBuf>::into)
        .map(|path| {
            let header = lib_header_from_path(&path).unwrap();
            header.init_root_module::<ExampleLib_Ref>().unwrap()
        })
        .collect();
    assert_eq!(libs.len(), 2);

    let plugin1 = libs[0];

    let plugin2 = libs[1];
    {
        let pfs1 = plugin1.plugin_functions()();
        dbg!(pfs1[0].0);
        let mut bi1 = pfs1[0].0(rvec![]);
        let pfs2 = plugin2.plugin_functions()();
        dbg!(pfs2[0].0);
        let bi2 = pfs2[0].0(rvec![&mut bi1]);

        assert_eq!(&*bi1.to_string(), "plugin1::plugin2 was here!");
        assert_eq!(&*bi2.to_string(), "plugin2::");
    }

    println!("success");
}
