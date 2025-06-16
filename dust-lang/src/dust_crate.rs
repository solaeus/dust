use std::marker::PhantomData;

use serde::{Deserialize, Serialize};

use crate::Module;

#[derive(Debug, Clone, Serialize)]
pub enum DustCrate<'a, C> {
    Library(Module<'a, C>),
    Program(Box<Program<C>>),
}

impl<'a, C> DustCrate<'a, C> {
    pub fn library(module: Module<'a, C>) -> Self {
        Self::Library(module)
    }

    pub fn program(main_chunk: C, cell_count: u16) -> Self {
        Self::Program(Box::new(Program {
            main_chunk,
            cell_count,
        }))
    }
}

impl<'de, C> Deserialize<'de> for DustCrate<'de, C>
where
    C: 'de + Deserialize<'de>,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct DustCrateVisitor<C> {
            _marker: PhantomData<C>,
        }

        impl<'de, C> serde::de::Visitor<'de> for DustCrateVisitor<C>
        where
            C: 'de + Deserialize<'de>,
        {
            type Value = DustCrate<'de, C>;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("a DustCrate enum")
            }

            fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
            where
                A: serde::de::SeqAccess<'de>,
            {
                if let Some(module) = seq.next_element()? {
                    Ok(DustCrate::Library(module))
                } else if let Some(progrma) = seq.next_element()? {
                    Ok(DustCrate::Program(progrma))
                } else {
                    Err(serde::de::Error::invalid_length(0, &self))
                }
            }
        }

        deserializer.deserialize_enum(
            "DustCrate",
            &["Library", "Program"],
            DustCrateVisitor {
                _marker: PhantomData,
            },
        )
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Program<C> {
    pub main_chunk: C,
    pub cell_count: u16,
}
