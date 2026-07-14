use std::fmt::{self, Display, Formatter};

use indexmap::IndexSet;
use rustc_hash::FxBuildHasher;

use crate::{
    compiler::context::{declarations::DeclarationId, types::TypeId},
    prototype::Prototype,
};

#[derive(Debug, Default)]
pub struct Prototypes {
    prototypes: Vec<Prototype>,
    compilation_stack: Vec<PrototypeId>,
    monomorphization_cache: IndexSet<(DeclarationId, TypeId::SmallVec), FxBuildHasher>,
}

impl Prototypes {
    pub fn into_prototypes(self) -> Vec<Prototype> {
        self.prototypes
    }

    pub fn pop_from_compilation_stack(&mut self) -> Option<PrototypeId> {
        self.compilation_stack.pop()
    }

    pub fn get_monomorphized_function(
        &self,
        prototype_id: PrototypeId,
    ) -> &(DeclarationId, TypeId::SmallVec) {
        &self.monomorphization_cache[prototype_id.0 as usize]
    }

    pub fn set_prototype(&mut self, prototype_id: PrototypeId, prototype: Prototype) {
        self.prototypes[prototype_id.0 as usize] = prototype;
    }

    pub fn monomorphize_function_to_prototype(
        &mut self,
        declaration_id: DeclarationId,
        type_arguments: TypeId::SmallVec,
    ) -> PrototypeId {
        let cache_key = (declaration_id, type_arguments);

        if let Some(index) = self.monomorphization_cache.get_index_of(&cache_key) {
            PrototypeId(index as u16)
        } else {
            let prototype_id = PrototypeId(self.prototypes.len() as u16);

            self.prototypes.push(Prototype::placeholder());
            self.monomorphization_cache.insert(cache_key);
            self.compilation_stack.push(prototype_id);

            prototype_id
        }
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord, Hash)]
pub struct PrototypeId(#[cfg(test)] pub(crate) u16, #[cfg(not(test))] u16);

impl PrototypeId {
    pub(crate) const MAIN: Self = Self(0);

    pub fn index(self) -> u16 {
        self.0
    }
}

impl Display for PrototypeId {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "proto_{}", self.0)
    }
}
