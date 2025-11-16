use std::ffi::{CStr, CString};

use pgrx::{GucContext, GucFlags, GucRegistry, GucSetting};

static JAEGER_ENABLED_PARAM: &CStr = c"fhir.jaeger_enabled";
pub static JAEGER_ENABLED: GucSetting<Option<CString>> = GucSetting::<Option<CString>>::new(None);

static JAEGER_HOST_PARAM: &CStr = c"fhir.jaeger_host";
pub static JAEGER_HOST: GucSetting<Option<CString>> = GucSetting::<Option<CString>>::new(None);

pub fn init() {
    GucRegistry::define_string_guc(
        JAEGER_ENABLED_PARAM,
        c"Enable Jaeger tracing",
        c"Enables exporting all traces to an external Jaeger instance",
        &JAEGER_ENABLED,
        GucContext::Userset,
        GucFlags::SUPERUSER_ONLY,
    );

    GucRegistry::define_string_guc(
        JAEGER_HOST_PARAM,
        c"Jaeger host",
        c"Host of the Jaeger instance to export traces to",
        &JAEGER_HOST,
        GucContext::Userset,
        GucFlags::SUPERUSER_ONLY,
    );
}
