use std::path::PathBuf;

use abi_stable::std_types::RString;
use abi_stable::{DynTrait, rvec};

use abi_stable::library::lib_header_from_path;
use plugin_interface::Plugin_Ref;

fn main() {
    let args = std::env::args().collect::<Vec<String>>();

    dbg!(&args);
    assert_eq!(args.len(), 2);

    let source_path: RString = args[1].clone().into();
    let boxed_source_path = DynTrait::from_value(source_path);
    let user_params = rvec![&boxed_source_path];

    let paths = [
        "target/debug/libplugin1.dylib",
        "target/debug/libplugin2.dylib",
    ];

    let libs: Vec<Plugin_Ref> = paths
        .iter()
        .map(std::convert::Into::<PathBuf>::into)
        .map(|path| {
            let header = lib_header_from_path(&path).unwrap();
            header.init_root_module::<Plugin_Ref>().unwrap()
        })
        .collect();
    assert_eq!(libs.len(), 2);

    let plugin1 = libs[0];

    let plugin2 = libs[1];
    {
        let pfs1 = plugin1.funcs()();
        dbg!(pfs1[0].0);
        let mut bi1 = pfs1[0].0(rvec![], user_params.clone());
        let pfs2 = plugin2.funcs()();
        dbg!(pfs2[0].0);
        let bi2 = pfs2[0].0(rvec![&mut bi1], user_params);

        assert_eq!(unsafe { *bi2.unchecked_downcast_as::<u32>() }, 2);
    }

    println!("success");
}
