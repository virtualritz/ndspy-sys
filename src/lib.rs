//! This is a thin, auto-generated wrapper around the
//! [NSI](https://nsi.readthedocs.io/)/[RenderMan®](https://renderman.pixar.com/)
//! low level display driver C API.
//! # Documentation
//! Pixar’s Developer’s Guide for [Display Driver
//! Plug-Ins](https://renderman.pixar.com/resources/RenderMan_20/dspyNote.html)
//! has in-depth information on this API that maps 1:1 to Rust.
//! # Example
//! For a Rust example demonstrting how to use this see the
//! [r-display](https://github.com/virtualritz/r-display) crate.
//!
#![allow(clippy::all)]
#![allow(improper_ctypes)]
#![allow(non_camel_case_types)]
#![allow(non_upper_case_globals)]
#![allow(non_snake_case)]
include!(concat!(env!("OUT_DIR"), "/bindings.rs"));
