use std::collections::HashMap;
use std::fmt;

use protos::protobuf::{self, Struct};

use crate::value::fmt_helpers::DebugValue;

#[derive(Clone, PartialEq)]
pub struct QueryStats {
    fields: HashMap<String, protobuf::Value>,
}

impl QueryStats {
    #[inline]
    pub(crate) fn from_struct(struc: Struct) -> Self {
        Self {
            fields: struc.fields,
        }
    }
}

impl fmt::Debug for QueryStats {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut dbg = f.debug_struct("QueryStats");

        for (key, value) in self.fields.iter() {
            if let Some(ref kind) = value.kind {
                dbg.field(key, &DebugValue(kind));
            }
        }

        dbg.finish()
    }
}
