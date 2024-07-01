use std::sync::{Arc, OnceLock};

use crate::{
    abstract_tree::{AbstractNode, AbstractTree},
    context::Context,
    lexer::lex,
    parser,
};
use chumsky::prelude::*;

pub fn std_full_compiled() -> [AbstractTree; 5] {
    [
        std_core_compiled().clone(),
        std_fs_compiled().clone(),
        std_io_compiled().clone(),
        std_json_compiled().clone(),
        std_thread_compiled().clone(),
    ]
}

pub const STD_CORE: &str = include_str!("../../std/core.ds");
pub const STD_FS: &str = include_str!("../../std/fs.ds");
pub const STD_IO: &str = include_str!("../../std/io.ds");
pub const STD_JSON: &str = include_str!("../../std/json.ds");
pub const STD_THREAD: &str = include_str!("../../std/thread.ds");

static CORE_CONTEXT: OnceLock<Context> = OnceLock::new();

pub fn core_context<'a>() -> &'a Context {
    CORE_CONTEXT.get_or_init(|| {
        let context = Context::new(None);
        let std_core = std_core_compiled().clone();

        std_core
            .define_types(&context)
            .expect("Failed to define types for std.core");
        std_core
            .validate(&context, true)
            .expect("Failed to validate std.core");
        std_core
            .evaluate(&context, true)
            .expect("Failed to evaluate std.core");

        context
    })
}

static CORE_SOURCE: OnceLock<(Arc<str>, Arc<str>)> = OnceLock::new();

pub fn core_source<'a>() -> &'a (Arc<str>, Arc<str>) {
    CORE_SOURCE.get_or_init(|| (Arc::from("std/core.ds"), Arc::from(STD_CORE)))
}

static STD_SOURCES: OnceLock<[(Arc<str>, Arc<str>); 4]> = OnceLock::new();

pub fn std_sources<'a>() -> &'a [(Arc<str>, Arc<str>); 4] {
    STD_SOURCES.get_or_init(|| {
        [
            (Arc::from("std/fs.ds"), Arc::from(STD_FS)),
            (Arc::from("std/io.ds"), Arc::from(STD_IO)),
            (Arc::from("std/json.ds"), Arc::from(STD_JSON)),
            (Arc::from("std/thread.ds"), Arc::from(STD_THREAD)),
        ]
    })
}

static STD_CORE_COMPILED: OnceLock<AbstractTree> = OnceLock::new();

pub fn std_core_compiled<'a>() -> &'a AbstractTree {
    STD_CORE_COMPILED.get_or_init(|| {
        let tokens = lex(STD_CORE).expect("Failed to lex");
        let abstract_tree = parser(true)
            .parse(tokens.spanned((tokens.len()..tokens.len()).into()))
            .into_result()
            .expect("Failed to parse");

        abstract_tree
    })
}

static STD_FS_COMPILED: OnceLock<AbstractTree> = OnceLock::new();

pub fn std_fs_compiled<'a>() -> &'a AbstractTree {
    STD_FS_COMPILED.get_or_init(|| {
        let tokens = lex(STD_FS).expect("Failed to lex");
        let abstract_tree = parser(true)
            .parse(tokens.spanned((tokens.len()..tokens.len()).into()))
            .into_result()
            .expect("Failed to parse");

        abstract_tree
    })
}

static STD_IO_COMPILED: OnceLock<AbstractTree> = OnceLock::new();

pub fn std_io_compiled<'a>() -> &'a AbstractTree {
    STD_IO_COMPILED.get_or_init(|| {
        let tokens = lex(STD_IO).expect("Failed to lex");
        let abstract_tree = parser(true)
            .parse(tokens.spanned((tokens.len()..tokens.len()).into()))
            .into_result()
            .expect("Failed to parse");

        abstract_tree
    })
}

static STD_JSON_COMPILED: OnceLock<AbstractTree> = OnceLock::new();

pub fn std_json_compiled<'a>() -> &'a AbstractTree {
    STD_JSON_COMPILED.get_or_init(|| {
        let tokens = lex(STD_JSON).expect("Failed to lex");
        let abstract_tree = parser(true)
            .parse(tokens.spanned((tokens.len()..tokens.len()).into()))
            .into_result()
            .expect("Failed to parse");

        abstract_tree
    })
}

static STD_THREAD_COMPILED: OnceLock<AbstractTree> = OnceLock::new();

pub fn std_thread_compiled<'a>() -> &'a AbstractTree {
    STD_THREAD_COMPILED.get_or_init(|| {
        let tokens = lex(STD_THREAD).expect("Failed to lex");
        let abstract_tree = parser(true)
            .parse(tokens.spanned((tokens.len()..tokens.len()).into()))
            .into_result()
            .expect("Failed to parse");

        abstract_tree
    })
}
