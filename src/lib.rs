use std::{
    collections::BTreeMap,
    ops::Range,
    sync::{Arc, OnceLock, PoisonError, RwLock},
};

use chumsky::{prelude::*, Parser};

pub static NONE: OnceLock<Value> = OnceLock::new();

pub enum BuiltInValue {
    None,
}

impl BuiltInValue {
    pub fn get(&self) -> Value {
        match self {
            BuiltInValue::None => NONE.get_or_init(|| {
                Value::r#enum(EnumInstance {
                    type_name: Identifier("Option".to_string()),
                    variant: Identifier("None".to_string()),
                })
            }),
        }
        .clone()
    }
}

pub enum Error<'src> {
    Parse(Vec<Rich<'src, char>>),
    Runtime(RuntimeError),
}

impl<'src> From<Vec<Rich<'src, char>>> for Error<'src> {
    fn from(errors: Vec<Rich<'src, char>>) -> Self {
        Error::Parse(errors)
    }
}

impl<'src> From<RuntimeError> for Error<'src> {
    fn from(error: RuntimeError) -> Self {
        Error::Runtime(error)
    }
}

pub enum RuntimeError {
    RwLockPoison(RwLockPoisonError),
}

impl From<RwLockPoisonError> for RuntimeError {
    fn from(error: RwLockPoisonError) -> Self {
        RuntimeError::RwLockPoison(error)
    }
}

pub struct RwLockPoisonError;

impl<T> From<PoisonError<T>> for RwLockPoisonError {
    fn from(_: PoisonError<T>) -> Self {
        RwLockPoisonError
    }
}

pub struct Context {
    inner: Arc<RwLock<BTreeMap<Identifier, Value>>>,
}

impl Context {
    pub fn get(&self, identifier: &Identifier) -> Result<Option<Value>, RwLockPoisonError> {
        let value = self.inner.read()?.get(&identifier).cloned();

        Ok(value)
    }

    pub fn set(&self, identifier: Identifier, value: Value) -> Result<(), RwLockPoisonError> {
        self.inner.write()?.insert(identifier, value);

        Ok(())
    }
}

pub trait AbstractTree {
    fn run(self, context: &Context) -> Result<Value, RuntimeError>;
}

pub struct Interpreter<P> {
    parser: P,
    context: Context,
}

impl<'src, P> Interpreter<P>
where
    P: Parser<'src, &'src str, Statement, extra::Err<Rich<'src, char>>>,
{
    pub fn run(&self, source: &'src str) -> Result<Value, Error<'src>> {
        let final_value = self
            .parser
            .parse(source)
            .into_result()?
            .run(&self.context)?;

        Ok(final_value)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum Statement {
    Assignment(Assignment),
    Expression(Expression),
}

impl AbstractTree for Statement {
    fn run(self, context: &Context) -> Result<Value, RuntimeError> {
        match self {
            Statement::Assignment(assignment) => assignment.run(context),
            Statement::Expression(expression) => expression.run(context),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum Expression {
    Identifier(Identifier),
    Logic(Logic),
    Value(Value),
}

impl AbstractTree for Expression {
    fn run(self, context: &Context) -> Result<Value, RuntimeError> {
        match self {
            Expression::Identifier(identifier) => identifier.run(context),
            Expression::Logic(logic) => logic.run(context),
            Expression::Value(value) => value.run(context),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
pub struct Identifier(String);

impl AbstractTree for Identifier {
    fn run(self, context: &Context) -> Result<Value, RuntimeError> {
        let value = context
            .get(&self)?
            .unwrap_or_else(|| BuiltInValue::None.get())
            .clone();

        Ok(value)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Assignment {
    identifier: Identifier,
    value: Value,
}

impl AbstractTree for Assignment {
    fn run(self, context: &Context) -> Result<Value, RuntimeError> {
        context.set(self.identifier, self.value)?;

        Ok(BuiltInValue::None.get().clone())
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Logic {
    left: LogicExpression,
    operator: LogicOperator,
    right: LogicExpression,
}

impl AbstractTree for Logic {
    fn run(self, _: &Context) -> Result<Value, RuntimeError> {
        todo!()
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum LogicOperator {
    Equal,
    NotEqual,
    Greater,
    Less,
    GreaterOrEqual,
    LessOrEqual,
    And,
    Or,
}

#[derive(Clone, Debug, PartialEq)]
pub enum LogicExpression {
    Identifier(Identifier),
    Logic(Box<Logic>),
    Value(Value),
}

#[derive(Clone, Debug, PartialEq)]
pub struct Value(Arc<ValueInner>);

impl Value {
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

    pub fn r#enum(r#enum: EnumInstance) -> Self {
        Value(Arc::new(ValueInner::Enum(r#enum)))
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum ValueInner {
    Boolean(bool),
    Float(f64),
    Integer(i64),
    List(Vec<Value>),
    Map(BTreeMap<Identifier, Value>),
    Range(Range<i64>),
    String(String),
    Enum(EnumInstance),
}

#[derive(Clone, Debug, PartialEq)]
pub struct EnumInstance {
    type_name: Identifier,
    variant: Identifier,
}

impl AbstractTree for Value {
    fn run(self, _: &Context) -> Result<Value, RuntimeError> {
        Ok(self)
    }
}

pub fn parser<'src>() -> impl Parser<'src, &'src str, Statement, extra::Err<Rich<'src, char>>> {
    let operator = |text: &'src str| just(text).padded();

    let value = recursive(|value| {
        let boolean = just("true")
            .or(just("false"))
            .map(|s: &str| Value::boolean(s.parse().unwrap()));

        let float_numeric = just('-')
            .or_not()
            .then(text::int(10))
            .then(just('.').then(text::digits(10)))
            .to_slice()
            .map(|text: &str| Value::float(text.parse().unwrap()));

        let float_other = choice((just("Infinity"), just("-Infinity"), just("NaN")))
            .map(|text| Value::float(text.parse().unwrap()));

        let float = choice((float_numeric, float_other));

        let integer = just('-')
            .or_not()
            .then(text::int(10).padded())
            .to_slice()
            .map(|text: &str| {
                let integer = text.parse::<i64>().unwrap();

                Value::integer(integer)
            });

        let delimited_string = |delimiter| {
            just(delimiter)
                .ignore_then(none_of(delimiter).repeated())
                .then_ignore(just(delimiter))
                .to_slice()
                .map(|text: &str| Value::string(text[1..text.len() - 1].to_string()))
        };

        let string = choice((
            delimited_string('\''),
            delimited_string('"'),
            delimited_string('`'),
        ));

        let list = value
            .clone()
            .separated_by(just(',').padded())
            .allow_trailing()
            .collect()
            .padded()
            .delimited_by(just('['), just(']'))
            .map(|values| Value::list(values));

        choice((boolean, float, integer, string, list))
    });

    let identifier = text::ident().map(|text: &str| Identifier(text.to_string()));

    let assignment = identifier
        .then_ignore(operator("="))
        .then(value.clone())
        .map(|(identifier, value)| Assignment { identifier, value });

    let logic = recursive(|logic| {
        choice((
            value.clone().map(|value| LogicExpression::Value(value)),
            identifier.map(|identifier| LogicExpression::Identifier(identifier)),
            logic
                .clone()
                .map(|logic| LogicExpression::Logic(Box::new(logic))),
        ))
        .then(choice((
            operator("==").map(|_| LogicOperator::Equal),
            operator("!=").map(|_| LogicOperator::NotEqual),
            operator(">").map(|_| LogicOperator::Greater),
            operator("<").map(|_| LogicOperator::Less),
            operator(">=").map(|_| LogicOperator::GreaterOrEqual),
            operator("<=").map(|_| LogicOperator::LessOrEqual),
            operator("&&").map(|_| LogicOperator::And),
            operator("||").map(|_| LogicOperator::Or),
        )))
        .then(choice((
            value.clone().map(|value| LogicExpression::Value(value)),
            identifier.map(|identifier| LogicExpression::Identifier(identifier)),
            logic.map(|logic| LogicExpression::Logic(Box::new(logic))),
        )))
        .map(|((left, operator), right)| Logic {
            left,
            operator,
            right,
        })
    });

    let expression = choice((
        logic.map(|logic| Expression::Logic(logic)),
        value.map(|value| Expression::Value(value)),
        identifier.map(|identifier| Expression::Identifier(identifier)),
    ));

    let statement = choice((
        assignment.map(|assignment| Statement::Assignment(assignment)),
        expression.map(|expression| Statement::Expression(expression)),
    ));

    statement
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_identifier() {
        assert_eq!(
            parser().parse("x").unwrap(),
            Statement::Expression(Expression::Identifier(Identifier("x".to_string())))
        );
        assert_eq!(
            parser().parse("foobar").unwrap(),
            Statement::Expression(Expression::Identifier(Identifier("foobar".to_string())))
        );
        assert_eq!(
            parser().parse("HELLO").unwrap(),
            Statement::Expression(Expression::Identifier(Identifier("HELLO".to_string())))
        );
    }

    #[test]
    fn parse_assignment() {
        assert_eq!(
            parser().parse("foobar = 1").unwrap(),
            Statement::Assignment(Assignment {
                identifier: Identifier("foobar".to_string()),
                value: Value::integer(1)
            })
        );
    }

    #[test]
    fn parse_logic() {
        assert_eq!(
            parser().parse("x == 1").unwrap(),
            Statement::Expression(Expression::Logic(Logic {
                left: LogicExpression::Identifier(Identifier("x".to_string())),
                operator: LogicOperator::Equal,
                right: LogicExpression::Value(Value::integer(1))
            }))
        );
    }

    #[test]
    fn parse_list() {
        assert_eq!(
            parser().parse("[]").unwrap(),
            Statement::Expression(Expression::Value(Value::list(vec![])))
        );
        assert_eq!(
            parser().parse("[42]").unwrap(),
            Statement::Expression(Expression::Value(Value::list(vec![Value::integer(42)])))
        );
        assert_eq!(
            parser().parse("[42, 'foo', \"bar\", [1, 2, 3,]]").unwrap(),
            Statement::Expression(Expression::Value(Value::list(vec![
                Value::integer(42),
                Value::string("foo".to_string()),
                Value::string("bar".to_string()),
                Value::list(vec![
                    Value::integer(1),
                    Value::integer(2),
                    Value::integer(3),
                ])
            ])))
        );
    }

    #[test]
    fn parse_true() {
        assert_eq!(
            parser().parse("true").unwrap(),
            Statement::Expression(Expression::Value(Value::boolean(true)))
        );
    }

    #[test]
    fn parse_false() {
        assert_eq!(
            parser().parse("false").unwrap(),
            Statement::Expression(Expression::Value(Value::boolean(false)))
        );
    }

    #[test]
    fn parse_positive_float() {
        assert_eq!(
            parser().parse("0.0").unwrap(),
            Statement::Expression(Expression::Value(Value::float(0.0)))
        );
        assert_eq!(
            parser().parse("42.0").unwrap(),
            Statement::Expression(Expression::Value(Value::float(42.0)))
        );

        let max_float = f64::MAX.to_string() + ".0";

        assert_eq!(
            parser().parse(&max_float).unwrap(),
            Statement::Expression(Expression::Value(Value::float(f64::MAX)))
        );

        let min_positive_float = f64::MIN_POSITIVE.to_string();

        assert_eq!(
            parser().parse(&min_positive_float).unwrap(),
            Statement::Expression(Expression::Value(Value::float(f64::MIN_POSITIVE)))
        );
    }

    #[test]
    fn parse_negative_float() {
        assert_eq!(
            parser().parse("-0.0").unwrap(),
            Statement::Expression(Expression::Value(Value::float(-0.0)))
        );
        assert_eq!(
            parser().parse("-42.0").unwrap(),
            Statement::Expression(Expression::Value(Value::float(-42.0)))
        );

        let min_float = f64::MIN.to_string() + ".0";

        assert_eq!(
            parser().parse(&min_float).unwrap(),
            Statement::Expression(Expression::Value(Value::float(f64::MIN)))
        );

        let max_negative_float = format!("-{}", f64::MIN_POSITIVE);

        assert_eq!(
            parser().parse(&max_negative_float).unwrap(),
            Statement::Expression(Expression::Value(Value::float(-f64::MIN_POSITIVE)))
        );
    }

    #[test]
    fn parse_other_float() {
        assert_eq!(
            parser().parse("Infinity").unwrap(),
            Statement::Expression(Expression::Value(Value::float(f64::INFINITY)))
        );
        assert_eq!(
            parser().parse("-Infinity").unwrap(),
            Statement::Expression(Expression::Value(Value::float(f64::NEG_INFINITY)))
        );

        if let Statement::Expression(Expression::Value(Value(value_inner))) =
            parser().parse("NaN").unwrap()
        {
            if let ValueInner::Float(float) = value_inner.as_ref() {
                return assert!(float.is_nan());
            }
        }

        panic!("Expected a float.")
    }

    #[test]
    fn parse_positive_integer() {
        for i in 0..10 {
            let source = i.to_string();
            let result = parser().parse(&source);

            assert_eq!(
                result.unwrap(),
                Statement::Expression(Expression::Value(Value::integer(i)))
            )
        }

        assert_eq!(
            parser().parse("42").unwrap(),
            Statement::Expression(Expression::Value(Value::integer(42)))
        );

        let maximum_integer = i64::MAX.to_string();

        assert_eq!(
            parser().parse(&maximum_integer).unwrap(),
            Statement::Expression(Expression::Value(Value::integer(i64::MAX)))
        );
    }

    #[test]
    fn parse_negative_integer() {
        for i in -9..1 {
            let source = i.to_string();
            let result = parser().parse(&source);

            assert_eq!(
                result.unwrap(),
                Statement::Expression(Expression::Value(Value::integer(i)))
            )
        }

        assert_eq!(
            parser().parse("-42").unwrap(),
            Statement::Expression(Expression::Value(Value::integer(-42)))
        );

        let minimum_integer = i64::MIN.to_string();

        assert_eq!(
            parser().parse(&minimum_integer).unwrap(),
            Statement::Expression(Expression::Value(Value::integer(i64::MIN)))
        );
    }

    #[test]
    fn double_quoted_string() {
        assert_eq!(
            parser().parse("\"\"").unwrap(),
            Statement::Expression(Expression::Value(Value::string("".to_string())))
        );
        assert_eq!(
            parser().parse("\"42\"").unwrap(),
            Statement::Expression(Expression::Value(Value::string("42".to_string())))
        );
        assert_eq!(
            parser().parse("\"foobar\"").unwrap(),
            Statement::Expression(Expression::Value(Value::string("foobar".to_string())))
        );
    }

    #[test]
    fn single_quoted_string() {
        assert_eq!(
            parser().parse("''").unwrap(),
            Statement::Expression(Expression::Value(Value::string("".to_string())))
        );
        assert_eq!(
            parser().parse("'42'").unwrap(),
            Statement::Expression(Expression::Value(Value::string("42".to_string())))
        );
        assert_eq!(
            parser().parse("'foobar'").unwrap(),
            Statement::Expression(Expression::Value(Value::string("foobar".to_string())))
        );
    }

    #[test]
    fn grave_quoted_string() {
        assert_eq!(
            parser().parse("``").unwrap(),
            Statement::Expression(Expression::Value(Value::string("".to_string())))
        );
        assert_eq!(
            parser().parse("`42`").unwrap(),
            Statement::Expression(Expression::Value(Value::string("42".to_string())))
        );
        assert_eq!(
            parser().parse("`foobar`").unwrap(),
            Statement::Expression(Expression::Value(Value::string("foobar".to_string())))
        );
    }
}
