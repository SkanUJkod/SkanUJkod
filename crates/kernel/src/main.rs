use abi_stable::library::RootModule;

use plugin_interface::{BoxedInterface, ExampleLib_Ref};

fn main() {
    // The type annotation is for the reader
    let plugin1 = ExampleLib_Ref::load_from_file("target/debug/libplugin1.dylib".as_ref())
        .unwrap_or_else(|e| panic!("{}", e));

    let plugin2 = ExampleLib_Ref::load_from_file("target/debug/libplugin2.dylib".as_ref())
        .unwrap_or_else(|e| panic!("{}", e));
    {
        ///////////////////////////////////////////////////////////////////////////////////
        //
        //  This block demonstrates the `DynTrait<>` trait object.
        //
        //  `DynTrait` is used here as a safe opaque type which can only be unwrapped back to
        //  the original type in the dynamic library that constructed the `DynTrait` itself.
        //
        ////////////////////////////////////////////////////////////////////////////////////

        // The type annotation is for the reader
        let mut unwrapped: BoxedInterface = plugin1.new_boxed_interface()();

        plugin2.append_string()(&mut unwrapped, "Hello".into());
        plugin2.append_string()(&mut unwrapped, ", world!".into());

        assert_eq!(&*unwrapped.to_string(), "Hello, world!");
    }

    {
        ///////////////////////////////////////////////////////////////////////////////////
        //
        //  This block demonstrates the `DynTrait<>` trait object.
        //
        //  `DynTrait` is used here as a safe opaque type which can only be unwrapped back to
        //  the original type in the dynamic library that constructed the `DynTrait` itself.
        //
        ////////////////////////////////////////////////////////////////////////////////////

        // The type annotation is for the reader
        let mut unwrapped: BoxedInterface = plugin2.new_boxed_interface()();

        plugin1.append_string()(&mut unwrapped, "Hello".into());
        plugin2.append_string()(&mut unwrapped, ", world!".into());

        assert_eq!(&*unwrapped.to_string(), "Hello, world!");
    }

    println!("success");
}
