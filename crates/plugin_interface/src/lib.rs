use abi_stable::{
    DynTrait, StableAbi,
    library::RootModule,
    package_version_strings,
    sabi_types::VersionStrings,
    std_types::{RBox, RHashMap, RString, RVec},
};

pub type PFDependencies<'a> = RHashMap<QualPFID, &'a mut BoxedPFResult<'static>>;
pub type UserParameters<'a> = RHashMap<RString, BoxedUserParam<'static>>;

#[repr(C)]
#[derive(StableAbi, Debug)]
pub struct PluginFunction(
    #[allow(improper_ctypes_definitions)]
    pub  extern "C" fn(PFDependencies, &UserParameters) -> BoxedPFResult<'static>,
);

pub type PluginID = RString;
pub type PFID = RString;

#[repr(C)]
#[derive(StableAbi, Debug, Clone, std::cmp::PartialEq, std::cmp::Eq, std::hash::Hash)]
pub struct QualPFID {
    pub plugin_id: PluginID,
    pub pf_id: PFID,
}

#[repr(C)]
#[derive(StableAbi, Debug)]
pub struct PFType {
    pub pf_dependencies: RVec<QualPFID>,
    pub user_params: RVec<RString>,
}

#[repr(C)]
#[derive(StableAbi, Debug)]
pub struct PFConnector {
    pub pf: PluginFunction,
    pub pf_type: PFType,
    pub pf_id: QualPFID,
}

/// This struct is the root module,
/// which must be converted to `PluginRef` to be passed through ffi.
///
/// The `#[sabi(kind(Prefix(prefix_ref = PluginRef)))]`
/// attribute tells `StableAbi` to create an ffi-safe static reference type
/// for `Plugin` called `PluginRef`.
///
/// The `#[sabi(missing_field(panic))]` attribute specifies that trying to
/// access a field that doesn't exist must panic with a message saying that
/// the field is inaccessible.
#[repr(C)]
#[derive(StableAbi)]
#[sabi(kind(Prefix(prefix_ref = PluginRef)))]
#[sabi(missing_field(panic))]
pub struct Plugin {
    #[sabi(last_prefix_field)]
    pub funcs: extern "C" fn() -> RVec<PFConnector>,
}

/// The RootModule trait defines how to load the root module of a library.
impl RootModule for PluginRef {
    abi_stable::declare_root_module_statics! {PluginRef}

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
