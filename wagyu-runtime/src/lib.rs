#![no_std]

#[macro_use]
extern crate alloc;

pub mod executor;
pub mod instance;
pub mod instr;
pub mod parse;
pub mod stack;

pub(crate) mod helper;
pub(crate) mod module;
