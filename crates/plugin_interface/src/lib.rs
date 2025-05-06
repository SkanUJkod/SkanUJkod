use abi_stable::{
    DynTrait, StableAbi,
    library::RootModule,
    package_version_strings,
    sabi_types::VersionStrings,
    std_types::{RBox, RVec},
};

#[repr(C)]
#[derive(StableAbi)]
pub struct PluginFunction(
    pub extern "C" fn(RVec<&mut BoxedInterface<'_>>) -> BoxedInterface<'static>,
);

/// This struct is the root module,
/// which must be converted to `ExampleLib_Ref` to be passed through ffi.
///
/// The `#[sabi(kind(Prefix(prefix_ref = ExampleLib_Ref)))]`
/// attribute tells `StableAbi` to create an ffi-safe static reference type
/// for `ExampleLib` called `ExampleLib_Ref`.
///
/// The `#[sabi(missing_field(panic))]` attribute specifies that trying to
/// access a field that doesn't exist must panic with a message saying that
/// the field is inaccessible.
#[repr(C)]
#[derive(StableAbi)]
#[sabi(kind(Prefix(prefix_ref = ExampleLib_Ref)))]
#[sabi(missing_field(panic))]
pub struct ExampleLib {
    #[sabi(last_prefix_field)]
    pub plugin_functions: extern "C" fn() -> RVec<PluginFunction>,
}

/// The RootModule trait defines how to load the root module of a library.
impl RootModule for ExampleLib_Ref {
    abi_stable::declare_root_module_statics! {ExampleLib_Ref}

    const BASE_NAME: &'static str = "plugin";
    const NAME: &'static str = "plugin";
    const VERSION_STRINGS: VersionStrings = package_version_strings!();
}

#[repr(C)]
#[derive(StableAbi)]
#[sabi(impl_InterfaceType(Sync, Send, Debug, Display))]
pub struct TheInterface;

/// An alias for the trait object used in this example
pub type BoxedInterface<'borr> = DynTrait<'borr, RBox<()>, TheInterface>;
