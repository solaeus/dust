use crate::{BuiltInFunction, Error, List, Map, Result, Value};

pub struct Type;

impl BuiltInFunction for Type {
    fn name(&self) -> &'static str {
        "type"
    }

    fn run(&self, arguments: &[Value], context: &Map) -> Result<Value> {
        Error::expect_argument_amount(self, 1, arguments.len())?;

        if arguments.len() == 1 {
            let type_definition = arguments.first().unwrap().r#type(context)?;
            let type_text = type_definition.to_string();
            let text_without_brackets = &type_text[1..type_text.len() - 1];

            Ok(Value::String(text_without_brackets.to_string()))
        } else {
            let mut answers = Vec::new();

            for value in arguments {
                let type_definition = value.r#type(context)?;
                let type_text = type_definition.to_string();
                let text_without_brackets = &type_text[1..type_text.len() - 1];
                let text_as_value = Value::String(text_without_brackets.to_string());

                answers.push(text_as_value);
            }

            Ok(Value::List(List::with_items(answers)))
        }
    }

    fn type_definition(&self) -> crate::TypeDefinition {
        todo!()
    }
}
