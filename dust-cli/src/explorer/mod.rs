use dust_compiler::{
    compiler::context::Context, instruction::OperandType, program::Program, source::Source,
    syntax::Syntax,
};

pub struct Explorer<'a> {
    program: &'a Program,
    source: &'a Source<'a>,
    syntax: &'a Syntax,
    constant_tags: &'a [OperandType],
    context: &'a Context,
}

impl<'a> Explorer<'a> {
    pub fn new(
        program: &'a Program,
        source: &'a Source,
        syntax: &'a Syntax,
        constant_tags: &'a [OperandType],
        context: &'a Context,
    ) -> Self {
        Self {
            program,
            source,
            syntax,
            constant_tags,
            context,
        }
    }
}
