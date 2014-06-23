// Copyright 2014 Dietrich Epp
// See the

#![crate_id = "datetime#0.1-pre"]
#![license = "MIT/ASL2"]
#![crate_type = "rlib"]
#![crate_type = "dylib"]

extern crate libc;
pub mod duration;
pub mod instant;
mod calendar_iso8601;
mod div_mod;
mod fmtutil;
mod tick;
