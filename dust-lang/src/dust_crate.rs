use serde::{Deserialize, Serialize};

use crate::{Chunk, Module};

#[derive(Debug, Clone, Serialize)]
pub struct DustCrate<'a> {
    pub main_chunk: Chunk,
    pub main_module: Module<'a>,
    pub cell_count: usize,
}

impl<'a> DustCrate<'a> {
    pub fn new(main_chunk: Chunk, main_module: Module<'a>) -> Self {
        Self {
            main_chunk,
            main_module,
            cell_count: 0,
        }
    }
}

impl<'de> Deserialize<'de> for DustCrate<'de> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct DustCrateVisitor;

        impl<'de> serde::de::Visitor<'de> for DustCrateVisitor {
            type Value = DustCrate<'de>;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("a DustCrate struct")
            }

            fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
            where
                A: serde::de::SeqAccess<'de>,
            {
                let main_chunk = seq
                    .next_element()?
                    .ok_or_else(|| serde::de::Error::invalid_length(0, &self))?;
                let main_module = seq
                    .next_element()?
                    .ok_or_else(|| serde::de::Error::invalid_length(1, &self))?;
                let cell_count = seq
                    .next_element()?
                    .ok_or_else(|| serde::de::Error::invalid_length(2, &self))?;

                Ok(DustCrate {
                    main_chunk,
                    main_module,
                    cell_count,
                })
            }
        }

        deserializer.deserialize_struct(
            "DustCrate",
            &["main_chunk", "main_module", "cell_count"],
            DustCrateVisitor,
        )
    }
}
