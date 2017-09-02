#[macro_use]
extern crate lazy_static;

#[allow(dead_code)]
#[allow(non_camel_case_types)]
#[allow(non_snake_case)]
#[allow(non_upper_case_globals)]
mod gpc {
    include!(concat!(env!("OUT_DIR"), "/bindings.rs"));
}

lazy_static! {
    pub static ref GPC_VERSION: &'static str = {
        use std::ffi::CStr;

        CStr::from_bytes_with_nul(gpc::GPC_VERSION)
            .expect("Could not convert GPC_VERSION to CStr")
            .to_str()
            .expect("GPC_VERSION was not valid UTF-8")
    };
}

#[cfg(test)]
mod tests {
    #[test]
    fn version() {
        assert_eq!("2.32", *super::GPC_VERSION);
    }
}
