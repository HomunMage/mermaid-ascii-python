// test_dummy.rs — Zero-content dependency for standalone test compilation.
//
// Importing `use test_dummy` in a .hom test file sets `has_rs_dep = true`
// in the homunc resolver, which switches sema to `skip_undef` mode.
// This is needed when a test calls functions from .rs companion files
// (e.g. path_state.rs, grid_data.rs) that the sema checker cannot inspect.
//
// This file intentionally has no declarations.
