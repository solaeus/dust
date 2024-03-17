use std::{
    cmp::Ordering,
    collections::BTreeMap,
    fmt::{self, Display, Formatter},
    ops::Range,
    sync::{Arc, OnceLock},
};

use stanza::{
    renderer::{console::Console, Renderer},
    style::{HAlign, MinWidth, Styles},
    table::Table,
};

use crate::{
    abstract_tree::{AbstractTree, Action, Block, Identifier, Positioned, Type},
    context::Context,
    error::{RuntimeError, ValidationError},
};

#[derive(Clone, Debug, PartialEq)]
pub struct Value(Arc<ValueInner>);

impl Value {
    pub fn inner(&self) -> &Arc<ValueInner> {
        &self.0
    }

    pub fn boolean(boolean: bool) -> Self {
        Value(Arc::new(ValueInner::Boolean(boolean)))
    }

    pub fn float(float: f64) -> Self {
        Value(Arc::new(ValueInner::Float(float)))
    }

    pub fn integer(integer: i64) -> Self {
        Value(Arc::new(ValueInner::Integer(integer)))
    }

    pub fn list(list: Vec<Value>) -> Self {
        Value(Arc::new(ValueInner::List(list)))
    }

    pub fn map(map: BTreeMap<Identifier, Value>) -> Self {
        Value(Arc::new(ValueInner::Map(map)))
    }

    pub fn range(range: Range<i64>) -> Self {
        Value(Arc::new(ValueInner::Range(range)))
    }

    pub fn string(string: String) -> Self {
        Value(Arc::new(ValueInner::String(string)))
    }

    pub fn function(
        parameters: Vec<(Identifier, Positioned<Type>)>,
        return_type: Positioned<Type>,
        body: Positioned<Block>,
    ) -> Self {
        Value(Arc::new(ValueInner::Function(Function::Parsed(
            ParsedFunction {
                parameters,
                return_type,
                body,
            },
        ))))
    }

    pub fn built_in_function(function: BuiltInFunction) -> Self {
        Value(Arc::new(ValueInner::Function(Function::BuiltIn(function))))
    }

    pub fn r#type(&self) -> Type {
        match self.0.as_ref() {
            ValueInner::Boolean(_) => Type::Boolean,
            ValueInner::Float(_) => Type::Float,
            ValueInner::Integer(_) => Type::Integer,
            ValueInner::List(values) => {
                let mut types = Vec::with_capacity(values.len());

                for value in values {
                    types.push(value.r#type());
                }

                Type::ListExact(types)
            }
            ValueInner::Map(_) => Type::Map,
            ValueInner::Range(_) => Type::Range,
            ValueInner::String(_) => Type::String,
            ValueInner::Function(_) => todo!(),
        }
    }

    pub fn as_boolean(&self) -> Result<bool, ValidationError> {
        if let ValueInner::Boolean(boolean) = self.0.as_ref() {
            return Ok(*boolean);
        }

        Err(ValidationError::ExpectedBoolean)
    }

    pub fn as_number(&self) -> Result<bool, ValidationError> {
        if let ValueInner::Boolean(boolean) = self.0.as_ref() {
            return Ok(*boolean);
        }

        Err(ValidationError::ExpectedBoolean)
    }

    pub fn as_function(&self) -> Result<&Function, ValidationError> {
        if let ValueInner::Function(function) = self.0.as_ref() {
            return Ok(function);
        }

        Err(ValidationError::ExpectedFunction)
    }

    pub fn as_list(&self) -> Option<&Vec<Value>> {
        if let ValueInner::List(list) = self.inner().as_ref() {
            Some(list)
        } else {
            None
        }
    }

    pub fn as_integer(&self) -> Option<i64> {
        if let ValueInner::Integer(integer) = self.inner().as_ref() {
            Some(*integer)
        } else {
            None
        }
    }

    pub fn add(&self, other: &Self) -> Result<Value, ValidationError> {
        match (self.inner().as_ref(), other.inner().as_ref()) {
            (ValueInner::Integer(left), ValueInner::Integer(right)) => {
                let sum = left.saturating_add(*right);

                Ok(Value::integer(sum))
            }
            (ValueInner::Float(left), ValueInner::Float(right)) => {
                let sum = left + right;

                Ok(Value::float(sum))
            }
            (ValueInner::Float(left), ValueInner::Integer(right)) => {
                let sum = left + *right as f64;

                Ok(Value::float(sum))
            }
            (ValueInner::Integer(left), ValueInner::Float(right)) => {
                let sum = *left as f64 + right;

                Ok(Value::float(sum))
            }
            _ => Err(ValidationError::ExpectedIntegerOrFloat),
        }
    }

    pub fn subtract(&self, other: &Self) -> Result<Value, ValidationError> {
        match (self.inner().as_ref(), other.inner().as_ref()) {
            (ValueInner::Integer(left), ValueInner::Integer(right)) => {
                let sum = left.saturating_sub(*right);

                Ok(Value::integer(sum))
            }
            (ValueInner::Float(left), ValueInner::Float(right)) => {
                let sum = left - right;

                Ok(Value::float(sum))
            }
            (ValueInner::Float(left), ValueInner::Integer(right)) => {
                let sum = left - *right as f64;

                Ok(Value::float(sum))
            }
            (ValueInner::Integer(left), ValueInner::Float(right)) => {
                let sum = *left as f64 - right;

                Ok(Value::float(sum))
            }
            _ => Err(ValidationError::ExpectedIntegerOrFloat),
        }
    }
}

impl Display for Value {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        fn create_table() -> Table {
            Table::with_styles(Styles::default().with(HAlign::Centred).with(MinWidth(3)))
        }

        match self.inner().as_ref() {
            ValueInner::Boolean(boolean) => write!(f, "{boolean}"),
            ValueInner::Float(float) => write!(f, "{float}"),
            ValueInner::Integer(integer) => write!(f, "{integer}"),
            ValueInner::List(list) => {
                let mut table = create_table();

                for value in list {
                    table = table.with_row([value.to_string()]);
                }

                write!(f, "{}", Console::default().render(&table))
            }
            ValueInner::Map(map) => {
                let mut table = create_table();

                for (identifier, value) in map {
                    table = table.with_row([identifier.as_str(), &value.to_string()]);
                }

                write!(f, "{}", Console::default().render(&table))
            }
            ValueInner::Range(_) => todo!(),
            ValueInner::String(string) => write!(f, "{string}"),
            ValueInner::Function(Function::Parsed(ParsedFunction {
                parameters,
                return_type,
                body,
            })) => {
                write!(f, "(")?;

                for (identifier, r#type) in parameters {
                    write!(f, "{identifier}: {}", r#type.node)?;
                }

                write!(f, "): {} {:?}", return_type.node, body.node)
            }
            ValueInner::Function(Function::BuiltIn(built_in_function)) => {
                write!(f, "{built_in_function}")
            }
        }
    }
}

impl Eq for Value {}

impl PartialOrd for Value {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Value {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.0.as_ref().cmp(other.0.as_ref())
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum ValueInner {
    Boolean(bool),
    Float(f64),
    Function(Function),
    Integer(i64),
    List(Vec<Value>),
    Map(BTreeMap<Identifier, Value>),
    Range(Range<i64>),
    String(String),
}

impl Eq for ValueInner {}

impl PartialOrd for ValueInner {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for ValueInner {
    fn cmp(&self, other: &Self) -> Ordering {
        use ValueInner::*;

        match (self, other) {
            (Boolean(left), Boolean(right)) => left.cmp(right),
            (Boolean(_), _) => Ordering::Greater,
            (Float(left), Float(right)) => left.total_cmp(right),
            (Float(_), _) => Ordering::Greater,
            (Integer(left), Integer(right)) => left.cmp(right),
            (Integer(_), _) => Ordering::Greater,
            (List(left), List(right)) => left.cmp(right),
            (List(_), _) => Ordering::Greater,
            (Map(left), Map(right)) => left.cmp(right),
            (Map(_), _) => Ordering::Greater,
            (Range(left), Range(right)) => {
                let start_cmp = left.start.cmp(&right.start);

                if start_cmp.is_eq() {
                    left.end.cmp(&right.end)
                } else {
                    start_cmp
                }
            }
            (Range(_), _) => Ordering::Greater,
            (String(left), String(right)) => left.cmp(right),
            (String(_), _) => Ordering::Greater,
            (Function(left), Function(right)) => left.cmp(right),
            (Function(_), _) => Ordering::Greater,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
pub enum Function {
    Parsed(ParsedFunction),
    BuiltIn(BuiltInFunction),
}

impl Function {
    pub fn call(self, arguments: Vec<Value>, context: Context) -> Result<Action, RuntimeError> {
        let action = match self {
            Function::Parsed(ParsedFunction {
                parameters, body, ..
            }) => {
                for ((identifier, _), value) in parameters.into_iter().zip(arguments.into_iter()) {
                    context.set_value(identifier.clone(), value)?;
                }

                body.node.run(&context)?
            }
            Function::BuiltIn(built_in_function) => built_in_function.call(arguments, &context)?,
        };

        Ok(action)
    }
}

#[derive(Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
pub struct ParsedFunction {
    parameters: Vec<(Identifier, Positioned<Type>)>,
    return_type: Positioned<Type>,
    body: Positioned<Block>,
}

#[derive(Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
pub enum BuiltInFunction {
    Output,
}

impl BuiltInFunction {
    pub fn output() -> Value {
        static OUTPUT: OnceLock<Value> = OnceLock::new();

        OUTPUT
            .get_or_init(|| Value::built_in_function(BuiltInFunction::Output))
            .clone()
    }

    pub fn r#type(&self) -> Type {
        match self {
            BuiltInFunction::Output => Type::Function {
                parameter_types: vec![Type::Any],
                return_type: Box::new(Type::None),
            },
        }
    }

    pub fn call(&self, arguments: Vec<Value>, _context: &Context) -> Result<Action, RuntimeError> {
        match self {
            BuiltInFunction::Output => {
                println!("{}", arguments[0]);

                Ok(Action::None)
            }
        }
    }
}

impl Display for BuiltInFunction {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            BuiltInFunction::Output => write!(f, "(to_output : any) : none rust_magic();"),
        }
    }
}
