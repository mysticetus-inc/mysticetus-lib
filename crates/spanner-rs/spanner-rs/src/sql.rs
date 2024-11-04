use std::collections::HashMap;

use protos::{protobuf, spanner};

use crate::IntoSpanner;
use crate::convert::SpannerEncode;
use crate::ty::{SpannerType, Type};

#[derive(Debug, Clone, PartialEq)]
pub struct Params {
    params: HashMap<String, protobuf::Value>,
    types: HashMap<String, spanner::Type>,
}

impl Params {
    #[inline]
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            params: HashMap::with_capacity(capacity),
            types: HashMap::with_capacity(capacity),
        }
    }

    fn insert_inner(&mut self, name: String, ty: &Type, value: crate::Value) {
        if !value.is_null() {
            self.types.insert(name.clone(), ty.into_proto());
        }

        self.params.insert(name, value.into_protobuf());
    }

    pub fn insert<N, T>(&mut self, name: N, value: T) -> &mut Self
    where
        N: Into<String>,
        T: IntoSpanner,
    {
        self.insert_inner(name.into(), T::TYPE, value.into_value());
        self
    }

    pub fn encode_insert<N, T>(&mut self, name: N, value: T) -> Result<&mut Self, T::Error>
    where
        N: Into<String>,
        T: SpannerEncode,
    {
        self.insert_inner(
            name.into(),
            <T::SpannerType as SpannerType>::TYPE,
            value.encode()?.into_value(),
        );
        Ok(self)
    }

    #[inline]
    pub fn new() -> Self {
        Self::with_capacity(0)
    }

    #[inline]
    pub(crate) fn into_parts(self) -> (Option<protobuf::Struct>, HashMap<String, spanner::Type>) {
        let s = if self.params.is_empty() {
            None
        } else {
            Some(protobuf::Struct {
                fields: self.params,
            })
        };

        (s, self.types)
    }
}

/*

pub struct Where {
    name: String,
    op: WhereOp,
}

pub enum WhereOp {
    Equals(protos::protobuf::Value),
}

pub struct OrderBy {
    field: String,
    desc: bool,
}

pub struct SimpleSqlQuery<T> {
    params: Option<Params>,
    limit: Option<u32>,
    order_by: Option<OrderBy>,
    _marker: PhantomData<T>,
}

impl<T: Table> SimpleSqlQuery<T> {
    pub fn new() -> Self {
        Self {
            params: None,
            limit: None,
            order_by: None,
            _marker: PhantomData,
        }
    }

    pub fn with_param_capacity(params: usize) -> Self {
        Self {
            params: Some(Params::with_capacity(params)),
            limit: None,
            order_by: None,
            _marker: PhantomData,
        }
    }

    pub fn take(&mut self) -> Self {
        std::mem::replace(self, Self::new())
    }
}
*/
