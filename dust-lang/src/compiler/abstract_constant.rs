use crate::{chunk::ConstantId, syntax_tree::SyntaxId};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AbstractConstant {
    Raw {
        expression: SyntaxId,
    },
    Folded {
        left_expression: SyntaxId,
        right_expression: SyntaxId,
    },
    LeftFold {
        left_constant: ConstantId,
        right_expression: SyntaxId,
    },
    RightFold {
        left_expression: SyntaxId,
        right_constant: ConstantId,
    },
    DoubleFold {
        left_constant: ConstantId,
        right_constant: ConstantId,
    },
}

impl AbstractConstant {
    pub fn fold(self, other: Self) -> Self {
        match (self, other) {
            (
                AbstractConstant::Raw { expression: left },
                AbstractConstant::Raw { expression: right },
            ) => AbstractConstant::Folded {
                left_expression: left,
                right_expression: right,
            },
            _ => todo!(),
        }
    }
}
