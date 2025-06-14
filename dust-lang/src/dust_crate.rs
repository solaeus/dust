use serde::{Deserialize, Serialize};

use crate::{Chunk, Module};

#[derive(Debug, Clone, Serialize)]
pub enum DustCrate<'a> {
    Library(Module<'a>),
    Program(Program<'a>),
}

impl<'a> DustCrate<'a> {
    pub fn library(module: Module<'a>) -> Self {
        Self::Library(module)
    }

    pub fn program(main_chunk: Chunk, main_module: Module<'a>, cell_count: u16) -> Self {
        Self::Program(Program {
            main_chunk,
            main_module,
            cell_count,
        })
    }
}

impl<'a, 'de: 'a> Deserialize<'de> for DustCrate<'a> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct DustCrateVisitor;

        impl<'de> serde::de::Visitor<'de> for DustCrateVisitor {
            type Value = DustCrate<'de>;

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

        deserializer.deserialize_enum("DustCrate", &["Library", "Program"], DustCrateVisitor)
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct Program<'a> {
    pub main_chunk: Chunk,
    pub main_module: Module<'a>,
    pub cell_count: u16,
}

impl<'a, 'de: 'a> Deserialize<'de> for Program<'a> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let (main_chunk, main_module, cell_count) = Deserialize::deserialize(deserializer)?;

        Ok(Program {
            main_chunk,
            main_module,
            cell_count,
        })
    }
}
