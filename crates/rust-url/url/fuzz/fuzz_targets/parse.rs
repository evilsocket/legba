#![no_main]
#[macro_use] extern crate libfuzzer_sys;
extern crate url;
use std::str;

fuzz_target!(|data: &[u8]| {
    if let Ok(utf8) = str::from_utf8(data) {
        if let Ok(parsed) = url::Url::parse(utf8) {
            let as_str = parsed.as_str();
            assert_eq!(parsed, url::Url::parse(as_str).unwrap());
        }
    }
});
