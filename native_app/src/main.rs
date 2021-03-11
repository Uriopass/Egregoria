#![allow(clippy::too_many_arguments)]
#![allow(clippy::float_cmp)]
#![warn(
    clippy::await_holding_lock,
    clippy::dbg_macro,
    clippy::debug_assert_with_mut_call,
    clippy::empty_enum,
    clippy::enum_glob_use,
    clippy::exit,
    clippy::explicit_into_iter_loop,
    clippy::filter_map_next,
    clippy::fn_params_excessive_bools,
    clippy::if_let_mutex,
    clippy::imprecise_flops,
    clippy::inefficient_to_string,
    clippy::large_types_passed_by_value,
    clippy::let_unit_value,
    clippy::linkedlist,
    clippy::lossy_float_literal,
    clippy::macro_use_imports,
    clippy::map_err_ignore,
    clippy::map_flatten,
    clippy::map_unwrap_or,
    clippy::match_on_vec_items,
    clippy::match_same_arms,
    clippy::match_wildcard_for_single_variants,
    clippy::mem_forget,
    clippy::mismatched_target_os,
    clippy::needless_borrow,
    clippy::needless_continue,
    clippy::option_option,
    clippy::pub_enum_variant_names,
    clippy::ref_option_ref,
    clippy::rest_pat_in_fully_bound_structs,
    clippy::string_add_assign,
    clippy::string_add,
    clippy::todo,
    clippy::unimplemented,
    clippy::unnested_or_patterns,
    clippy::unused_self,
    clippy::verbose_file_reads,
    future_incompatible,
    nonstandard_style,
    rust_2018_idioms
)]
#![allow(elided_lifetimes_in_paths)]

use crate::context::Context;
use crate::logger::MyLog;
use log::LevelFilter;

#[macro_use]
extern crate common;

#[macro_use]
extern crate egregoria;

mod audio;
mod context;
mod game_loop;
mod gui;
mod input;
mod logger;
mod rendering;

fn main() {
    let leaked = Box::leak(Box::new(MyLog::new()));
    log::set_logger(leaked).unwrap();
    log::set_max_level(LevelFilter::Debug);
    log_panics::init();

    let mut ctx = Context::new();

    let state = game_loop::State::new(&mut ctx);
    ctx.start(state);
}
