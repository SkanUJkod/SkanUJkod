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
    pub  extern "C" fn(
        RVec<&mut BoxedPFResult<'_>>,
        RVec<&BoxedUserParam<'_>>,
    ) -> BoxedPFResult<'static>,
);

/// This struct is the root module,
/// which must be converted to `Plugin_Ref` to be passed through ffi.
///
/// The `#[sabi(kind(Prefix(prefix_ref = Plugin_Ref)))]`
/// attribute tells `StableAbi` to create an ffi-safe static reference type
/// for `Plugin` called `Plugin_Ref`.
///
/// The `#[sabi(missing_field(panic))]` attribute specifies that trying to
/// access a field that doesn't exist must panic with a message saying that
/// the field is inaccessible.
#[repr(C)]
#[derive(StableAbi)]
#[sabi(kind(Prefix(prefix_ref = Plugin_Ref)))]
#[sabi(missing_field(panic))]
pub struct Plugin {
    #[sabi(last_prefix_field)]
    pub funcs: extern "C" fn() -> RVec<PluginFunction>,
}

/// The RootModule trait defines how to load the root module of a library.
impl RootModule for Plugin_Ref {
    abi_stable::declare_root_module_statics! {Plugin_Ref}

    const BASE_NAME: &'static str = "plugin";
    const NAME: &'static str = "plugin";
    const VERSION_STRINGS: VersionStrings = package_version_strings!();
}

#[repr(C)]
#[derive(StableAbi)]
#[sabi(impl_InterfaceType(Debug, Display))]
pub struct PluginFunctionResult;

/// An alias for the trait object used in this example
pub type BoxedPFResult<'borr> = DynTrait<'borr, RBox<()>, PluginFunctionResult>;

#[repr(C)]
#[derive(StableAbi)]
#[sabi(impl_InterfaceType(Debug, Display))]
pub struct UserParam;

/// An alias for the trait object used in this example
pub type BoxedUserParam<'borr> = DynTrait<'borr, RBox<()>, UserParam>;
