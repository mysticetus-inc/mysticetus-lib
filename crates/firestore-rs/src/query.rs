use std::collections::HashMap;
use std::pin::Pin;
use std::task::{Context, Poll, ready};

use futures::{Stream, StreamExt};
use protos::firestore::run_query_request::{ConsistencySelector, QueryType};
use protos::firestore::run_query_response::ContinuationSelector;
use protos::firestore::structured_query::composite_filter::Operator as CompositeOperator;
use protos::firestore::structured_query::field_filter::Operator as FieldOperator;
use protos::firestore::structured_query::filter::FilterType;
use protos::firestore::structured_query::unary_filter::{OperandType, Operator as UnaryOperator};
use protos::firestore::structured_query::{
    CollectionSelector, CompositeFilter, Direction, FieldFilter, FieldReference, Filter, Order,
    Projection, UnaryFilter,
};
use protos::firestore::value::ValueType;
use protos::firestore::{
    ArrayValue, Cursor, Document, MapValue, RunQueryRequest, RunQueryResponse, StructuredQuery,
    Value,
};
use protos::protobuf::Int32Value;

use crate::client::FirestoreClient;
use crate::de::deserialize_doc_fields;
use crate::ser::{NullOverwrite, OmitNulls, ValueSerializer};

pub struct FieldFilterBuilder<'a> {
    field: FieldReference,
    builder: QueryBuilder<'a>,
}

pub struct IdFilterBuilder<'a> {
    builder: QueryBuilder<'a>,
}

macro_rules! impl_id_field_fns {
    ($(($fn_name:ident, $op_variant:ident)),* $(,)?) => {
        $(
            pub fn $fn_name<S>(mut self, id: S) -> QueryBuilder<'a>
            where
                S: crate::PathComponent
            {
                let mut doc_ref = format!(
                    "{}/{}/",
                    self.builder.parent,
                    self.builder.from.collection_id
                );

                id.append_to_path(&mut doc_ref);

                self.builder.add_field_filter(
                    FieldReference { field_path: "__name__".into() },
                    FieldOperator::$op_variant,
                    Value { value_type: Some(ValueType::ReferenceValue(doc_ref))}
                );
                self.builder
            }
        )*
    };
}

impl<'a> IdFilterBuilder<'a> {
    impl_id_field_fns! {
        (less_than, LessThan),
        (less_than_or_eq, LessThanOrEqual),
        (greater_than, GreaterThan),
        (greater_than_or_eq, GreaterThanOrEqual),
        (equals, Equal),
        (not_equals, NotEqual),
    }
}

pub struct ValueWrapper(Value);

fn json_to_value_type(json: serde_json::Value) -> ValueType {
    match json {
        serde_json::Value::Null => ValueType::NullValue(0),
        serde_json::Value::Bool(boolean) => ValueType::BooleanValue(boolean),
        serde_json::Value::Number(number) => {
            if let Some(int) = number.as_i64() {
                ValueType::IntegerValue(int)
            } else if let Some(uint) = number.as_u64() {
                ValueType::IntegerValue(uint as i64)
            } else if let Some(float) = number.as_f64() {
                ValueType::DoubleValue(float)
            } else {
                unreachable!("serde_json::Number only has 3 variants")
            }
        }
        serde_json::Value::String(string) => ValueType::StringValue(string),
        serde_json::Value::Array(array) => {
            let values = array
                .into_iter()
                .map(|json| Value {
                    value_type: Some(json_to_value_type(json)),
                })
                .collect::<Vec<Value>>();

            ValueType::ArrayValue(ArrayValue { values })
        }
        serde_json::Value::Object(map) => {
            let fields = map
                .into_iter()
                .map(|(key, value)| {
                    (
                        key,
                        Value {
                            value_type: Some(json_to_value_type(value)),
                        },
                    )
                })
                .collect::<HashMap<String, Value>>();

            ValueType::MapValue(MapValue { fields })
        }
    }
}

impl<T> From<T> for ValueWrapper
where
    T: Into<serde_json::Value>,
{
    fn from(val: T) -> Self {
        Self(Value {
            value_type: Some(json_to_value_type(val.into())),
        })
    }
}

macro_rules! impl_unary {
    ($(($fn_name:ident, $op_variant:ident)),* $(,)?) => {
        $(
            #[allow(clippy::wrong_self_convention)] // convention clashes with builder pattern
            pub fn $fn_name(self) -> QueryBuilder<'a> {
                let FieldFilterBuilder { mut builder, field } = self;
                builder.add_unary_filter(field, UnaryOperator::$op_variant);
                builder
            }
        )*
    };
}

macro_rules! impl_field {
    ($(($fn_name:ident, $op_variant:ident)),* $(,)?) => {
        $(
            pub fn $fn_name<T>(self, value: T) -> QueryBuilder<'a>
            where
                T: Into<ValueWrapper>,
            {
                let value = value.into().0;
                let Self { mut builder, field } = self;
                builder.add_field_filter(field, FieldOperator::$op_variant, value);
                builder
            }
        )*
    };
}

macro_rules! impl_array_fields {
    ($(($fn_name:ident, $op_variant:ident, $null_strat:ty)),* $(,)?) => {
        $(
            pub fn $fn_name<V, const N: usize>(self, values: [&V; N]) -> crate::Result<QueryBuilder<'a>>
            where
                V: serde::Serialize,
            {
                assert!(N < 11, "cannot use more than 10 array values in an array query");
                let serialized = ValueSerializer::<$null_strat>::NEW.seq(&values)?;

                let Self { mut builder, field } = self;
                builder.add_field_filter(field, FieldOperator::$op_variant, serialized);
                Ok(builder)
            }
        )*
    };
}

impl<'a> FieldFilterBuilder<'a> {
    impl_unary! {
        (is_null, IsNull),
        (is_not_null, IsNotNull),
        (is_nan, IsNan),
        (is_not_nan, IsNotNan),
    }

    impl_field! {
        (less_than, LessThan),
        (less_than_or_eq, LessThanOrEqual),
        (greater_than, GreaterThan),
        (greater_than_or_eq, GreaterThanOrEqual),
        (equals, Equal),
        (not_equals, NotEqual),
        (array_contains, ArrayContains),
        (array_contains_any, ArrayContainsAny),
    }

    impl_array_fields! {
        (one_of, In, NullOverwrite),
        (one_of_omit_nulls, In, OmitNulls),
        (not_one_of, NotIn, NullOverwrite),
        (not_one_of_omit_nulls, NotIn, OmitNulls),
    }
}

pub struct QueryBuilder<'a> {
    client: &'a mut FirestoreClient,
    parent: String,
    select: Option<Vec<FieldReference>>,
    from: CollectionSelector,
    filter: Option<Filter>,
    order_by: Vec<Order>,
    start_at: Option<Cursor>,
    end_at: Option<Cursor>,
    offset: u32,
    limit: Option<u32>,
}

impl<'a> QueryBuilder<'a> {
    pub(crate) fn collection_scoped(
        client: &'a mut FirestoreClient,
        parent: String,
        collection_id: String,
    ) -> Self {
        Self {
            client,
            parent,
            select: None,
            from: CollectionSelector {
                collection_id,
                all_descendants: false,
            },
            filter: None,
            order_by: vec![],
            start_at: None,
            end_at: None,
            offset: 0,
            limit: None,
        }
    }

    pub fn where_field<S>(self, field: S) -> FieldFilterBuilder<'a>
    where
        S: AsRef<str>,
    {
        let field_path = crate::ser::escape_field_path(field.as_ref());

        FieldFilterBuilder {
            builder: self,
            field: FieldReference { field_path },
        }
    }

    pub fn where_id(self) -> IdFilterBuilder<'a> {
        IdFilterBuilder { builder: self }
    }

    pub fn limit(mut self, limit: u32) -> Self {
        self.limit = Some(limit);
        self
    }

    #[inline]
    fn order_by_inner(mut self, field_path: &str, direction: Direction) -> Self {
        let field_path = crate::ser::escape_field_path(field_path);

        self.order_by.push(Order {
            field: Some(FieldReference { field_path }),
            direction: direction as i32,
        });

        self
    }

    fn start_at_inner<S>(mut self, value: S, before: bool) -> crate::Result<Self>
    where
        S: serde::Serialize,
    {
        let values = match value.serialize(crate::ser::ValueSerializer::default())? {
            Some(value) => vec![value],
            None => vec![],
        };

        self.start_at = Some(Cursor { values, before });
        Ok(self)
    }

    pub fn start_at<S>(self, value: S) -> crate::Result<Self>
    where
        S: serde::Serialize,
    {
        self.start_at_inner(value, true)
    }

    pub fn start_after<S>(self, value: S) -> crate::Result<Self>
    where
        S: serde::Serialize,
    {
        self.start_at_inner(value, false)
    }

    fn end_at_inner<S>(mut self, value: S, before: bool) -> crate::Result<Self>
    where
        S: serde::Serialize,
    {
        let values = match value.serialize(crate::ser::ValueSerializer::default())? {
            Some(value) => vec![value],
            None => vec![],
        };

        self.end_at = Some(Cursor { values, before });
        Ok(self)
    }

    pub fn end_at<S>(self, value: S) -> crate::Result<Self>
    where
        S: serde::Serialize,
    {
        self.end_at_inner(value, true)
    }

    pub fn end_after<S>(self, value: S) -> crate::Result<Self>
    where
        S: serde::Serialize,
    {
        self.end_at_inner(value, false)
    }

    pub fn order_by<S>(self, field: S) -> Self
    where
        S: AsRef<str>,
    {
        self.order_by_inner(field.as_ref(), Direction::Ascending)
    }

    pub fn order_by_desc<S>(self, field: S) -> Self
    where
        S: AsRef<str>,
    {
        self.order_by_inner(field.as_ref(), Direction::Descending)
    }

    fn take_filter(&mut self) -> Option<FilterType> {
        self.filter.take().and_then(|filter| filter.filter_type)
    }

    fn add_filter(&mut self, filter: Filter) {
        // Passing in a composite filter not yet supported
        assert!(!matches!(
            filter.filter_type,
            Some(FilterType::CompositeFilter(_))
        ));

        let existing = self.take_filter();

        self.filter = match existing {
            Some(FilterType::CompositeFilter(mut composite)) => {
                composite.filters.push(filter);

                Some(Filter {
                    filter_type: Some(FilterType::CompositeFilter(composite)),
                })
            }
            Some(filter_type) => Some(Filter {
                filter_type: Some(FilterType::CompositeFilter(CompositeFilter {
                    op: CompositeOperator::And as i32,
                    filters: vec![
                        Filter {
                            filter_type: Some(filter_type),
                        },
                        filter,
                    ],
                })),
            }),
            None => Some(filter),
        };
    }

    fn add_unary_filter(&mut self, field: FieldReference, op: UnaryOperator) {
        self.add_filter(Filter {
            filter_type: Some(FilterType::UnaryFilter(UnaryFilter {
                operand_type: Some(OperandType::Field(field)),
                op: op as i32,
            })),
        })
    }

    fn add_field_filter(&mut self, field: FieldReference, op: FieldOperator, value: Value) {
        self.add_filter(Filter {
            filter_type: Some(FilterType::FieldFilter(FieldFilter {
                field: Some(field),
                op: op as i32,
                value: Some(value),
            })),
        })
    }

    fn into_query(self) -> (&'a mut FirestoreClient, String, StructuredQuery) {
        let query = StructuredQuery {
            find_nearest: None,
            select: self.select.map(|fields| Projection { fields }),
            from: vec![self.from],
            r#where: self.filter,
            order_by: self.order_by,
            start_at: self.start_at,
            end_at: self.end_at,
            offset: self.offset as i32,
            limit: self.limit.map(|lim| Int32Value { value: lim as i32 }),
        };

        (self.client, self.parent, query)
    }

    pub async fn first<D>(mut self) -> crate::Result<Option<D>>
    where
        D: serde::de::DeserializeOwned,
    {
        self.limit = Some(1);
        let mut stream = self.run().await?;
        // futures::pin_mut!(stream);

        match stream.next().await {
            Some(Ok(doc)) => Ok(Some(doc)),
            Some(Err(err)) => Err(err),
            None => Ok(None),
        }
    }

    async fn run_raw_inner(
        self,
        consistency_selector: Option<ConsistencySelector>,
    ) -> crate::Result<RawQueryStream> {
        let limit = self.limit.map(|uint| uint as usize);
        let (client, parent, query) = self.into_query();

        let request = RunQueryRequest {
            query_type: Some(QueryType::StructuredQuery(query)),
            parent,
            consistency_selector,
            explain_options: None,
        };

        let stream = client.get().run_query(request).await?.into_inner();

        Ok(RawQueryStream {
            stream: Some(stream),
            limit,
        })
    }

    pub async fn run_raw(self) -> crate::Result<RawQueryStream> {
        self.run_raw_inner(None).await
    }

    pub async fn run<D>(self) -> crate::Result<QueryStream<D>>
    where
        D: serde::de::DeserializeOwned,
    {
        let stream = self.run_raw().await?;
        Ok(QueryStream {
            stream,
            _marker: std::marker::PhantomData,
        })
    }

    pub async fn run_to_completion<D>(self) -> crate::Result<Vec<D>>
    where
        D: serde::de::DeserializeOwned,
    {
        let mut query_stream = self.run().await?;
        let (lower, upper) = query_stream.size_hint();

        let mut results = Vec::with_capacity(upper.unwrap_or(lower));

        while let Some(result) = query_stream.next().await {
            if let Some(item) = result? {
                results.push(item);
            }
        }

        Ok(results)
    }
}

pin_project_lite::pin_project! {
    #[derive(Debug)]
    pub struct RawQueryStream {
        limit: Option<usize>,
        #[pin]
        stream: Option<tonic::Streaming<RunQueryResponse>>,
    }
}

impl RawQueryStream {
    pub fn is_completed(&self) -> bool {
        self.stream.is_none()
    }

    pub async fn run_to_completion(self) -> crate::Result<Vec<Document>> {
        let Some(mut stream) = self.stream else {
            return Ok(vec![]);
        };

        let (low, high) = stream.size_hint();

        let cap = match (high, self.limit) {
            (Some(high), Some(limit)) => limit.min(high),
            (None, Some(cap)) | (Some(cap), None) => cap,
            (None, None) => low,
        };

        let mut buf = Vec::with_capacity(cap);

        while let Some(result) = stream.next().await {
            if let Some(doc) = result?.document {
                buf.push(doc);
            }
        }

        Ok(buf)
    }
}

impl Stream for RawQueryStream {
    type Item = crate::Result<Document>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let mut this = self.project();

        loop {
            let Some(stream) = this.stream.as_mut().as_pin_mut() else {
                return Poll::Ready(None);
            };

            let chunk = match ready!(stream.poll_next(cx)) {
                Some(result) => result?,
                None => {
                    this.stream.set(None);
                    return Poll::Ready(None);
                }
            };

            // register another wake up right away, since the stream should be done at this point.
            // Dont stream.set(None) here to make sure the actual stream shuts down cleanly.
            if chunk.continuation_selector == Some(ContinuationSelector::Done(true)) {
                cx.waker().wake_by_ref();
            }

            if let Some(doc) = chunk.document {
                return Poll::Ready(Some(Ok(doc)));
            }
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let Some((low, stream_high)) = self.stream.as_ref().map(|stream| stream.size_hint()) else {
            return (0, Some(0));
        };

        match (stream_high, self.limit) {
            (Some(high), Some(limit)) => (low, Some(high.min(limit))),
            (Some(either), _) | (_, Some(either)) => (low, Some(either)),
            _ => (low, None),
        }
    }
}

pin_project_lite::pin_project! {
    #[derive(Debug)]
    pub struct QueryStream<D> {
        #[pin]
        stream: RawQueryStream,
        _marker: std::marker::PhantomData<D>,
    }
}

impl<D> QueryStream<D> {
    pub fn is_completed(&self) -> bool {
        self.stream.is_completed()
    }
}

impl<D> QueryStream<D>
where
    D: serde::de::DeserializeOwned,
{
    pub async fn run_to_completion(mut self) -> crate::Result<Vec<D>> {
        let (low, high) = self.stream.size_hint();
        let mut buf = Vec::with_capacity(high.unwrap_or(low));

        while let Some(result) = self.stream.next().await {
            let doc = result?;
            let deser = deserialize_doc_fields(doc.fields)?;
            buf.push(deser);
        }

        Ok(buf)
    }
}

impl<D> Stream for QueryStream<D>
where
    D: serde::de::DeserializeOwned,
{
    type Item = crate::Result<D>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        match ready!(self.project().stream.poll_next(cx)) {
            Some(Ok(document)) => {
                let deser_result = deserialize_doc_fields(document.fields).map_err(Into::into);

                Poll::Ready(Some(deser_result))
            }
            Some(Err(status)) => Poll::Ready(Some(Err(status))),
            None => Poll::Ready(None),
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.stream.size_hint()
    }
}
