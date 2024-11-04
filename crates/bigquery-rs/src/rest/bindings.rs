// @generated
//! A data platform for customers to create, manage, share and query data.

/// The Base URL for this service.
pub const BASE_URL: &str = "https://bigquery.googleapis.com/bigquery/v2/";

#[derive(
    ::serde::Deserialize,
    ::serde::Serialize,
    ::typed_builder::TypedBuilder,
    Debug,
    PartialEq,
    Clone,
    Default,
)]
#[serde(rename_all = "camelCase")]
pub struct BigtableColumn {
    /// [Optional] If this is set, only the latest version of value in this column are exposed.
    /// 'onlyReadLatest' can also be set at the column family level. However, the setting at this
    /// level takes precedence if 'onlyReadLatest' is set at both levels.
    #[builder(setter(into))]
    #[serde(default)]
    pub only_read_latest: ::std::option::Option<bool>,
    /// [Optional] The encoding of the values when the type is not STRING. Acceptable encoding
    /// values are: TEXT - indicates values are alphanumeric text strings. BINARY - indicates
    /// values are encoded using HBase Bytes.toBytes family of functions. 'encoding' can also
    /// be set at the column family level. However, the setting at this level takes precedence
    /// if 'encoding' is set at both levels.
    #[builder(setter(into))]
    #[serde(default)]
    pub encoding: ::std::option::Option<::std::string::String>,
    /// [Required] Qualifier of the column. Columns in the parent column family that has this exact
    /// qualifier are exposed as . field. If the qualifier is valid UTF-8 string, it can be
    /// specified in the qualifier_string field. Otherwise, a base-64 encoded value must be set
    /// to qualifier_encoded. The column field name is the same as the column qualifier.
    /// However, if the qualifier is not a valid BigQuery field identifier i.e. does not match
    /// [a-zA-Z][a-zA-Z0-9_]*, a valid identifier must be provided as field_name.
    #[builder(setter(into))]
    #[serde(default)]
    pub qualifier_encoded: ::bytes::Bytes,
    #[builder(setter(into))]
    #[serde(default)]
    pub qualifier_string: ::std::string::String,
    /// [Optional] The type to convert the value in cells of this column. The values are expected
    /// to be encoded using HBase Bytes.toBytes function when using the BINARY encoding value.
    /// Following BigQuery types are allowed (case-sensitive) - BYTES STRING INTEGER FLOAT
    /// BOOLEAN Default type is BYTES. 'type' can also be set at the column family level.
    /// However, the setting at this level takes precedence if 'type' is set at both levels.
    #[builder(setter(into))]
    #[serde(rename = "type")]
    #[serde(default)]
    pub ty: ::std::option::Option<::std::string::String>,
    /// [Optional] If the qualifier is not a valid BigQuery field identifier i.e. does not match
    /// [a-zA-Z][a-zA-Z0-9_]*, a valid identifier must be provided as the column field name and is
    /// used as field name in queries.
    #[builder(setter(into))]
    #[serde(default)]
    pub field_name: ::std::option::Option<::std::string::String>,
}

#[derive(
    ::serde::Deserialize,
    ::serde::Serialize,
    ::typed_builder::TypedBuilder,
    Debug,
    PartialEq,
    Clone,
    Default,
)]
#[serde(rename_all = "camelCase")]
pub struct BigtableOptions {
    /// [Optional] If field is true, then the column families that are not specified in
    /// columnFamilies list are not exposed in the table schema. Otherwise, they are read with
    /// BYTES type values. The default value is false.
    #[builder(setter(into))]
    #[serde(default)]
    pub ignore_unspecified_column_families: ::std::option::Option<bool>,
    /// [Optional] If field is true, then the rowkey column families will be read and converted to
    /// string. Otherwise they are read with BYTES type values and users need to manually cast them
    /// with CAST if necessary. The default value is false.
    #[builder(setter(into))]
    #[serde(default)]
    pub read_rowkey_as_string: ::std::option::Option<bool>,
    /// [Optional] List of column families to expose in the table schema along with their types.
    /// This list restricts the column families that can be referenced in queries and specifies
    /// their value types. You can use this list to do type conversions - see the 'type' field
    /// for more details. If you leave this list empty, all column families are present in the
    /// table schema and their values are read as BYTES. During a query only the column
    /// families referenced in that query are read from Bigtable.
    #[builder(setter(into))]
    #[serde(default)]
    pub column_families: ::std::vec::Vec<BigtableColumnFamily>,
}

/// [Output-only, Beta] Training options used by this training run. These options are mutable for
/// subsequent training runs. Default values are explicitly stored for options not specified in the
/// input query of the first training run. For subsequent training runs, any option not explicitly
/// specified in the input query will be copied from the previous training run.
#[derive(
    ::serde::Deserialize,
    ::serde::Serialize,
    ::typed_builder::TypedBuilder,
    Debug,
    PartialEq,
    Clone,
    Default,
)]
#[serde(rename_all = "camelCase")]
pub struct TrainingOptions {
    #[builder(setter(into))]
    #[serde(default)]
    pub l_2_reg: f64,
    #[builder(setter(into))]
    #[serde(default)]
    pub early_stop: bool,
    #[builder(setter(into))]
    #[serde(default)]
    pub l_1_reg: f64,
    #[builder(setter(into))]
    #[serde(default)]
    pub min_rel_progress: f64,
    #[builder(setter(into))]
    #[serde(default)]
    pub learn_rate_strategy: ::std::string::String,
    #[builder(setter(into))]
    #[serde(default)]
    pub line_search_init_learn_rate: f64,
    #[builder(setter(into))]
    #[serde(default)]
    pub learn_rate: f64,
    #[builder(setter(into))]
    #[serde(with = "with::int64")]
    #[serde(default)]
    pub max_iteration: i64,
    #[builder(setter(into))]
    #[serde(default)]
    pub warm_start: bool,
}

#[derive(
    ::serde::Deserialize,
    ::serde::Serialize,
    ::typed_builder::TypedBuilder,
    Debug,
    PartialEq,
    Clone,
    Default,
)]
#[serde(rename_all = "camelCase")]
pub struct BqmlTrainingRun {
    /// [Output-only, Beta] Training options used by this training run. These options are mutable
    /// for subsequent training runs. Default values are explicitly stored for options not
    /// specified in the input query of the first training run. For subsequent training runs,
    /// any option not explicitly specified in the input query will be copied from the previous
    /// training run.
    #[builder(setter(into))]
    #[serde(default)]
    pub training_options: TrainingOptions,
    /// [Output-only, Beta] List of each iteration results.
    #[builder(setter(into))]
    #[serde(default)]
    pub iteration_results: ::std::vec::Vec<BqmlIterationResult>,
    /// [Output-only, Beta] Different state applicable for a training run. IN PROGRESS: Training
    /// run is in progress. FAILED: Training run ended due to a non-retryable failure.
    /// SUCCEEDED: Training run successfully completed. CANCELLED: Training run cancelled by
    /// the user.
    #[builder(setter(into))]
    #[serde(default)]
    pub state: ::std::option::Option<::std::string::String>,
    /// [Output-only, Beta] Training run start time in milliseconds since the epoch.
    #[builder(setter(into))]
    #[serde(default)]
    pub start_time: ::std::option::Option<::timestamp::Timestamp>,
}

/// BigQuery-specific metadata about a location. This will be set on
/// google.cloud.location.Location.metadata in Cloud Location API responses.
#[derive(
    ::serde::Deserialize,
    ::serde::Serialize,
    ::typed_builder::TypedBuilder,
    Debug,
    PartialEq,
    Clone,
    Default,
)]
#[serde(rename_all = "camelCase")]
pub struct LocationMetadata {
    /// The legacy BigQuery location ID, e.g. “EU” for the “europe” location. This is for any
    /// API consumers that need the legacy “US” and “EU” locations.
    #[builder(setter(into))]
    #[serde(default)]
    pub legacy_location_id: ::std::string::String,
}

#[derive(
    ::serde::Deserialize,
    ::serde::Serialize,
    ::typed_builder::TypedBuilder,
    Debug,
    PartialEq,
    Clone,
    Default,
)]
#[serde(rename_all = "camelCase")]
pub struct JobReference {
    /// [Required] The ID of the job. The ID must contain only letters (a-z, A-Z), numbers (0-9),
    /// underscores (_), or dashes (-). The maximum length is 1,024 characters.
    #[builder(setter(into))]
    #[serde(default)]
    pub job_id: ::std::string::String,
    /// The geographic location of the job. See details at
    /// https://cloud.google.com/bigquery/docs/locations#specifying_your_location.
    #[builder(setter(into))]
    #[serde(default)]
    pub location: ::std::string::String,
    /// [Required] The ID of the project containing this job.
    #[builder(setter(into))]
    #[serde(default)]
    pub project_id: ::std::string::String,
}

/// Range of a double hyperparameter.
#[derive(
    ::serde::Deserialize,
    ::serde::Serialize,
    ::typed_builder::TypedBuilder,
    Debug,
    PartialEq,
    Clone,
    Default,
)]
#[serde(rename_all = "camelCase")]
pub struct DoubleRange {
    /// Max value of the double parameter.
    #[builder(setter(into))]
    #[serde(default)]
    pub max: f64,
    /// Min value of the double parameter.
    #[builder(setter(into))]
    #[serde(default)]
    pub min: f64,
}

/// Information about a single cluster for clustering model.
#[derive(
    ::serde::Deserialize,
    ::serde::Serialize,
    ::typed_builder::TypedBuilder,
    Debug,
    PartialEq,
    Clone,
    Default,
)]
#[serde(rename_all = "camelCase")]
pub struct ClusterInfo {
    /// Cluster radius, the average distance from centroid to each point assigned to the cluster.
    #[builder(setter(into))]
    #[serde(default)]
    pub cluster_radius: f64,
    /// Centroid id.
    #[builder(setter(into))]
    #[serde(with = "with::int64")]
    #[serde(default)]
    pub centroid_id: i64,
    /// Cluster size, the total number of points assigned to the cluster.
    #[builder(setter(into))]
    #[serde(with = "with::int64")]
    #[serde(default)]
    pub cluster_size: i64,
}

/// Model evaluation metrics for dimensionality reduction models.
#[derive(
    ::serde::Deserialize,
    ::serde::Serialize,
    ::typed_builder::TypedBuilder,
    Debug,
    PartialEq,
    Clone,
    Default,
)]
#[serde(rename_all = "camelCase")]
pub struct DimensionalityReductionMetrics {
    /// Total percentage of variance explained by the selected principal components.
    #[builder(setter(into))]
    #[serde(default)]
    pub total_explained_variance_ratio: f64,
}

/// [Output-only] Endpoints generated for the Spark job.
pub type Endpoints = ::std::collections::HashMap<::std::string::String, ::std::string::String>;

#[derive(
    ::serde::Deserialize,
    ::serde::Serialize,
    ::typed_builder::TypedBuilder,
    Debug,
    PartialEq,
    Clone,
    Default,
)]
#[serde(rename_all = "camelCase")]
pub struct SparkStatistics {
    /// [Output-only] Endpoints generated for the Spark job.
    #[builder(setter(into))]
    #[serde(default)]
    pub endpoints: Endpoints,
    /// [Output-only] Logging info is used to generate a link to Cloud Logging.
    #[builder(setter(into))]
    #[serde(default)]
    pub logging_info: ::std::option::Option<SparkLoggingInfo>,
    /// [Output-only] Location where the Spark job is executed.
    #[builder(setter(into))]
    #[serde(default)]
    pub spark_job_location: ::std::option::Option<::std::string::String>,
    /// [Output-only] Spark job id if a Spark job is created successfully.
    #[builder(setter(into))]
    #[serde(default)]
    pub spark_job_id: ::std::option::Option<::std::string::String>,
}

/// Information about a single training query run for the model.
#[derive(
    ::serde::Deserialize, ::serde::Serialize, ::typed_builder::TypedBuilder, Debug, PartialEq, Clone,
)]
#[serde(rename_all = "camelCase")]
pub struct TrainingRun {
    /// Output only. Global explanation contains the explanation of top features on the model
    /// level. Applies to both regression and classification models.
    #[builder(setter(into))]
    #[serde(default)]
    pub model_level_global_explanation: GlobalExplanation,
    /// Output only. The start time of this training run, in milliseconds since epoch.
    #[builder(setter(into))]
    #[serde(with = "with::int64")]
    #[serde(default)]
    pub training_start_time: i64,
    /// Output only. The model version in the [Vertex AI Model
    /// Registry](https://cloud.google.com/vertex-ai/docs/model-registry/introduction) for this training
    /// run.
    #[builder(setter(into))]
    #[serde(default)]
    pub vertex_ai_model_version: ::std::string::String,
    /// The model id in the [Vertex AI Model
    /// Registry](https://cloud.google.com/vertex-ai/docs/model-registry/introduction) for this training
    /// run.
    #[builder(setter(into))]
    #[serde(default)]
    pub vertex_ai_model_id: ::std::string::String,
    /// Output only. Data split result of the training run. Only set when the input data is
    /// actually split.
    #[builder(setter(into))]
    pub data_split_result: DataSplitResult,
    /// Output only. The evaluation metrics over training/eval data that were computed at the end
    /// of training.
    #[builder(setter(into))]
    pub evaluation_metrics: EvaluationMetrics,
    /// Output only. Output of each iteration run, results.size() <= max_iterations.
    #[builder(setter(into))]
    #[serde(default)]
    pub results: ::std::vec::Vec<IterationResult>,
    /// Output only. Options that were used for this training run, includes user specified and
    /// default options that were used.
    #[builder(setter(into))]
    #[serde(default)]
    pub training_options: TrainingOptions,
    /// Output only. The start time of this training run.
    #[builder(setter(into))]
    #[serde(default)]
    pub start_time: ::std::string::String,
    /// Output only. Global explanation contains the explanation of top features on the class
    /// level. Applies to classification models only.
    #[builder(setter(into))]
    #[serde(default)]
    pub class_level_global_explanations: ::std::vec::Vec<GlobalExplanation>,
}

/// Data split result. This contains references to the training and evaluation data tables that were
/// used to train the model.
#[derive(
    ::serde::Deserialize,
    ::serde::Serialize,
    ::typed_builder::TypedBuilder,
    Debug,
    PartialEq,
    Clone,
    Default,
)]
#[serde(rename_all = "camelCase")]
pub struct DataSplitResult {
    /// Table reference of the training data after split.
    #[builder(setter(into))]
    #[serde(default)]
    pub training_table: TableReference,
    /// Table reference of the evaluation data after split.
    #[builder(setter(into))]
    #[serde(default)]
    pub evaluation_table: TableReference,
    /// Table reference of the test data after split.
    #[builder(setter(into))]
    #[serde(default)]
    pub test_table: TableReference,
}

/// Represents access on a subset of rows on the specified table, defined by its filter predicate.
/// Access to the subset of rows is controlled by its IAM policy.
#[derive(
    ::serde::Deserialize,
    ::serde::Serialize,
    ::typed_builder::TypedBuilder,
    Debug,
    PartialEq,
    Clone,
    Default,
)]
#[serde(rename_all = "camelCase")]
pub struct RowAccessPolicy {
    /// Output only. The time when this row access policy was created, in milliseconds since the
    /// epoch.
    #[builder(setter(into))]
    #[serde(default)]
    pub creation_time: ::std::string::String,
    /// Output only. The time when this row access policy was last modified, in milliseconds since
    /// the epoch.
    #[builder(setter(into))]
    #[serde(default)]
    pub last_modified_time: ::std::string::String,
    /// Required. Reference describing the ID of this row access policy.
    #[builder(setter(into))]
    #[serde(default)]
    pub row_access_policy_reference: RowAccessPolicyReference,
    /// Required. A SQL boolean expression that represents the rows defined by this row access
    /// policy, similar to the boolean expression in a WHERE clause of a SELECT query on a
    /// table. References to other tables, routines, and temporary functions are not supported.
    /// Examples: region="EU" date_field = CAST('2019-9-27' as DATE) nullable_field is not NULL
    /// numeric_field BETWEEN 1.0 AND 5.0
    #[builder(setter(into))]
    #[serde(default)]
    pub filter_predicate: ::std::string::String,
    /// Output only. A hash of this resource.
    #[builder(setter(into))]
    #[serde(default)]
    pub etag: ::std::string::String,
}

/// ARIMA model fitting metrics.
#[derive(
    ::serde::Deserialize,
    ::serde::Serialize,
    ::typed_builder::TypedBuilder,
    Debug,
    PartialEq,
    Clone,
    Default,
)]
#[serde(rename_all = "camelCase")]
pub struct ArimaFittingMetrics {
    /// Variance.
    #[builder(setter(into))]
    #[serde(default)]
    pub variance: f64,
    /// Log-likelihood.
    #[builder(setter(into))]
    #[serde(default)]
    pub log_likelihood: f64,
    /// AIC.
    #[builder(setter(into))]
    #[serde(default)]
    pub aic: f64,
}

#[derive(
    ::serde::Deserialize,
    ::serde::Serialize,
    ::typed_builder::TypedBuilder,
    Debug,
    PartialEq,
    Clone,
    Default,
)]
#[serde(rename_all = "camelCase")]
pub struct AvroOptions {
    /// [Optional] If sourceFormat is set to "AVRO", indicates whether to interpret logical types
    /// as the corresponding BigQuery data type (for example, TIMESTAMP), instead of using the
    /// raw type (for example, INTEGER).
    #[builder(setter(into))]
    #[serde(default)]
    pub use_avro_logical_types: ::std::option::Option<bool>,
}

/// Search space for int array.
#[derive(
    ::serde::Deserialize,
    ::serde::Serialize,
    ::typed_builder::TypedBuilder,
    Debug,
    PartialEq,
    Clone,
    Default,
)]
#[serde(rename_all = "camelCase")]
pub struct IntArrayHparamSearchSpace {
    /// Candidates for the int array parameter.
    #[builder(setter(into))]
    #[serde(default)]
    pub candidates: ::std::vec::Vec<IntArray>,
}

/// Confusion matrix for multi-class classification models.
#[derive(
    ::serde::Deserialize,
    ::serde::Serialize,
    ::typed_builder::TypedBuilder,
    Debug,
    PartialEq,
    Clone,
    Default,
)]
#[serde(rename_all = "camelCase")]
pub struct ConfusionMatrix {
    /// One row per actual label.
    #[builder(setter(into))]
    #[serde(default)]
    pub rows: ::std::vec::Vec<Row>,
    /// Confidence threshold used when computing the entries of the confusion matrix.
    #[builder(setter(into))]
    #[serde(default)]
    pub confidence_threshold: f64,
}

/// Evaluation metrics of a model. These are either computed on all training data or just the eval
/// data based on whether eval data was used during training. These are not present for imported
/// models.
#[derive(
    ::serde::Deserialize, ::serde::Serialize, ::typed_builder::TypedBuilder, Debug, PartialEq, Clone,
)]
#[serde(rename_all = "camelCase")]
pub struct EvaluationMetrics {
    /// Populated for multi-class classification/classifier models.
    #[builder(setter(into))]
    pub multi_class_classification_metrics: MultiClassClassificationMetrics,
    /// Populated for implicit feedback type matrix factorization models.
    #[builder(setter(into))]
    #[serde(default)]
    pub ranking_metrics: RankingMetrics,
    /// Populated for clustering models.
    #[builder(setter(into))]
    #[serde(default)]
    pub clustering_metrics: ClusteringMetrics,
    /// Evaluation metrics when the model is a dimensionality reduction model, which currently
    /// includes PCA.
    #[builder(setter(into))]
    #[serde(default)]
    pub dimensionality_reduction_metrics: DimensionalityReductionMetrics,
    /// Populated for ARIMA models.
    #[builder(setter(into))]
    #[serde(default)]
    pub arima_forecasting_metrics: ArimaForecastingMetrics,
    /// Populated for binary classification/classifier models.
    #[builder(setter(into))]
    pub binary_classification_metrics: BinaryClassificationMetrics,
    /// Populated for regression models and explicit feedback type matrix factorization models.
    #[builder(setter(into))]
    #[serde(default)]
    pub regression_metrics: RegressionMetrics,
}

#[derive(
    ::serde::Deserialize,
    ::serde::Serialize,
    ::typed_builder::TypedBuilder,
    Debug,
    PartialEq,
    Clone,
    Default,
)]
#[serde(rename_all = "camelCase")]
pub struct ReservationUsage {
    /// [Output only] Slot-milliseconds the job spent in the given reservation.
    #[builder(setter(into))]
    #[serde(with = "with::int64")]
    #[serde(default)]
    pub slot_ms: i64,
    /// [Output only] Reservation name or "unreserved" for on-demand resources usage.
    #[builder(setter(into))]
    #[serde(default)]
    pub name: ::std::string::String,
}

#[derive(
    ::serde::Deserialize,
    ::serde::Serialize,
    ::typed_builder::TypedBuilder,
    Debug,
    PartialEq,
    Clone,
    Default,
)]
#[serde(rename_all = "camelCase")]
pub struct JobStatistics2 {
    /// The DDL operation performed, possibly dependent on the pre-existence of the DDL target.
    /// Possible values (new values might be added in the future): "CREATE": The query created
    /// the DDL target. "SKIP": No-op. Example cases: the query is CREATE TABLE IF NOT EXISTS
    /// while the table already exists, or the query is DROP TABLE IF EXISTS while the table
    /// does not exist. "REPLACE": The query replaced the DDL target. Example case: the query
    /// is CREATE OR REPLACE TABLE, and the table already exists. "DROP": The query deleted the
    /// DDL target.
    #[builder(setter(into))]
    #[serde(default)]
    pub ddl_operation_performed: ::std::string::String,
    /// [Output only] Referenced routines (persistent user-defined functions and stored procedures)
    /// for the job.
    #[builder(setter(into))]
    #[serde(default)]
    pub referenced_routines: ::std::vec::Vec<RoutineReference>,
    /// The type of query statement, if valid. Possible values (new values might be added in the
    /// future): "SELECT": SELECT query. "INSERT": INSERT query; see
    /// https://cloud.google.com/bigquery/docs/reference/standard-sql/data-manipulation-language.
    /// "UPDATE": UPDATE query; see
    /// https://cloud.google.com/bigquery/docs/reference/standard-sql/data-manipulation-language.
    /// "DELETE": DELETE query; see
    /// https://cloud.google.com/bigquery/docs/reference/standard-sql/data-manipulation-language.
    /// "MERGE": MERGE query; see
    /// https://cloud.google.com/bigquery/docs/reference/standard-sql/data-manipulation-language.
    /// "ALTER_TABLE": ALTER TABLE query. "ALTER_VIEW": ALTER VIEW query. "ASSERT": ASSERT
    /// condition AS 'description'. "CREATE_FUNCTION": CREATE FUNCTION query. "CREATE_MODEL":
    /// CREATE [OR REPLACE] MODEL ... AS SELECT ... . "CREATE_PROCEDURE": CREATE PROCEDURE
    /// query. "CREATE_TABLE": CREATE [OR REPLACE] TABLE without AS SELECT.
    /// "CREATE_TABLE_AS_SELECT": CREATE [OR REPLACE] TABLE ... AS SELECT ... . "CREATE_VIEW":
    /// CREATE [OR REPLACE] VIEW ... AS SELECT ... . "DROP_FUNCTION" : DROP FUNCTION query.
    /// "DROP_PROCEDURE": DROP PROCEDURE query. "DROP_TABLE": DROP TABLE query. "DROP_VIEW":
    /// DROP VIEW query.
    #[builder(setter(into))]
    #[serde(default)]
    pub statement_type: ::std::string::String,
    /// [Output-only] Total bytes transferred for cross-cloud queries such as Cross Cloud Transfer
    /// and CREATE TABLE AS SELECT (CTAS).
    #[builder(setter(into))]
    #[serde(with = "with::int64::option")]
    #[serde(default)]
    pub transferred_bytes: ::std::option::Option<i64>,
    /// [Output only] Job resource usage breakdown by reservation.
    #[builder(setter(into))]
    #[serde(default)]
    pub reservation_usage: ::std::vec::Vec<ReservationUsage>,
    /// [Output only] The DDL target dataset. Present only for CREATE/ALTER/DROP SCHEMA queries.
    #[builder(setter(into))]
    #[serde(default)]
    pub ddl_target_dataset: DatasetReference,
    /// Standard SQL only: list of undeclared query parameters detected during a dry run
    /// validation.
    #[builder(setter(into))]
    #[serde(default)]
    pub undeclared_query_parameters: ::std::vec::Vec<QueryParameter>,
    /// [Output only] Whether the query result was fetched from the query cache.
    #[builder(setter(into))]
    #[serde(default)]
    pub cache_hit: bool,
    /// [Output only] Detailed statistics for DML statements Present only for DML statements
    /// INSERT, UPDATE, DELETE or TRUNCATE.
    #[builder(setter(into))]
    #[serde(default)]
    pub dml_stats: DmlStatistics,
    /// [Output only, Beta] Deprecated; do not use.
    #[builder(setter(into))]
    #[serde(with = "with::int64")]
    #[serde(default)]
    pub model_training_expected_total_iteration: i64,
    /// [Output only] Total number of partitions processed from all partitioned tables referenced
    /// in the job.
    #[builder(setter(into))]
    #[serde(with = "with::int64")]
    #[serde(default)]
    pub total_partitions_processed: i64,
    /// [Output only] [Preview] The number of row access policies affected by a DDL statement.
    /// Present only for DROP ALL ROW ACCESS POLICIES queries.
    #[builder(setter(into))]
    #[serde(with = "with::int64")]
    #[serde(default)]
    pub ddl_affected_row_access_policy_count: i64,
    /// [Output only] [Beta] Describes a timeline of job execution.
    #[builder(setter(into))]
    #[serde(default)]
    pub timeline: ::std::vec::Vec<QueryTimelineSample>,
    /// [Output only] The schema of the results. Present only for successful dry run of non-legacy
    /// SQL queries.
    #[builder(setter(into))]
    #[serde(default)]
    pub schema: TableSchema,
    /// [Output only] The number of rows affected by a DML statement. Present only for DML
    /// statements INSERT, UPDATE or DELETE.
    #[builder(setter(into))]
    #[serde(with = "with::int64")]
    #[serde(default)]
    pub num_dml_affected_rows: i64,
    /// [Output only, Beta] Information about create model query job progress.
    #[builder(setter(into))]
    #[serde(default)]
    pub model_training: BigQueryModelTraining,
    /// The DDL target routine. Present only for CREATE/DROP FUNCTION/PROCEDURE queries.
    #[builder(setter(into))]
    #[serde(default)]
    pub ddl_target_routine: RoutineReference,
    /// [Output only] For dry-run jobs, totalBytesProcessed is an estimate and this field specifies
    /// the accuracy of the estimate. Possible values can be: UNKNOWN: accuracy of the estimate
    /// is unknown. PRECISE: estimate is precise. LOWER_BOUND: estimate is lower bound of what
    /// the query would cost. UPPER_BOUND: estimate is upper bound of what the query would
    /// cost.
    #[builder(setter(into))]
    #[serde(default)]
    pub total_bytes_processed_accuracy: ::std::string::String,
    /// [Output only] Total bytes billed for the job.
    #[builder(setter(into))]
    #[serde(with = "with::int64")]
    #[serde(default)]
    pub total_bytes_billed: i64,
    /// [Output only] Billing tier for the job.
    #[builder(setter(into))]
    #[serde(default)]
    pub billing_tier: i64,
    /// [Output only] Statistics of a BigQuery ML training job.
    #[builder(setter(into))]
    #[serde(default)]
    pub ml_statistics: MlStatistics,
    /// [Output only] Statistics of a Spark procedure job.
    #[builder(setter(into))]
    #[serde(default)]
    pub spark_statistics: SparkStatistics,
    /// [Output only] Slot-milliseconds for the job.
    #[builder(setter(into))]
    #[serde(with = "with::int64")]
    #[serde(default)]
    pub total_slot_ms: i64,
    /// [Output only] Total bytes processed for the job.
    #[builder(setter(into))]
    #[serde(with = "with::int64")]
    #[serde(default)]
    pub total_bytes_processed: i64,
    /// [Output only] The original estimate of bytes processed for the job.
    #[builder(setter(into))]
    #[serde(with = "with::int64")]
    #[serde(default)]
    pub estimated_bytes_processed: i64,
    /// [Output only] Describes execution plan for the query.
    #[builder(setter(into))]
    #[serde(default)]
    pub query_plan: ::std::vec::Vec<ExplainQueryStage>,
    /// [Output only] Search query specific statistics.
    #[builder(setter(into))]
    #[serde(default)]
    pub search_statistics: SearchStatistics,
    /// [Output only] Referenced tables for the job. Queries that reference more than 50 tables
    /// will not have a complete list.
    #[builder(setter(into))]
    #[serde(default)]
    pub referenced_tables: ::std::vec::Vec<TableReference>,
    /// [Output only] The DDL destination table. Present only for ALTER TABLE RENAME TO queries.
    /// Note that ddl_target_table is used just for its type information.
    #[builder(setter(into))]
    #[serde(default)]
    pub ddl_destination_table: TableReference,
    /// [Output only] [Preview] The DDL target row access policy. Present only for CREATE/DROP ROW
    /// ACCESS POLICY queries.
    #[builder(setter(into))]
    #[serde(default)]
    pub ddl_target_row_access_policy: RowAccessPolicyReference,
    /// [Output only] The DDL target table. Present only for CREATE/DROP TABLE/VIEW and DROP ALL
    /// ROW ACCESS POLICIES queries.
    #[builder(setter(into))]
    #[serde(default)]
    pub ddl_target_table: TableReference,
    /// BI Engine specific Statistics. [Output only] BI Engine specific Statistics.
    #[builder(setter(into))]
    #[serde(default)]
    pub bi_engine_statistics: BiEngineStatistics,
    /// [Output only, Beta] Deprecated; do not use.
    #[builder(setter(into))]
    #[serde(default)]
    pub model_training_current_iteration: i64,
}

/// Represents a textual expression in the Common Expression Language (CEL) syntax. CEL is a C-like
/// expression language. The syntax and semantics of CEL are documented at
/// https://github.com/google/cel-spec. Example (Comparison): title: "Summary size limit"
/// description: "Determines if a summary is less than 100 chars" expression:
/// "document.summary.size() < 100" Example (Equality): title: "Requestor is owner" description:
/// "Determines if requestor is the document owner" expression: "document.owner ==
/// request.auth.claims.email" Example (Logic): title: "Public documents" description: "Determine
/// whether the document should be publicly visible" expression: "document.type != 'private' &&
/// document.type != 'internal'" Example (Data Manipulation): title: "Notification string"
/// description: "Create a notification string with a timestamp." expression: "'New message received
/// at ' + string(document.create_time)" The exact variables and functions that may be referenced
/// within an expression are determined by the service that evaluates it. See the service
/// documentation for additional information.
#[derive(
    ::serde::Deserialize,
    ::serde::Serialize,
    ::typed_builder::TypedBuilder,
    Debug,
    PartialEq,
    Clone,
    Default,
)]
#[serde(rename_all = "camelCase")]
pub struct Expr {
    /// Optional. Description of the expression. This is a longer text which describes the
    /// expression, e.g. when hovered over it in a UI.
    #[builder(setter(into))]
    #[serde(default)]
    pub description: ::std::option::Option<::std::string::String>,
    /// Optional. Title for the expression, i.e. a short string describing its purpose. This can be
    /// used e.g. in UIs which allow to enter the expression.
    #[builder(setter(into))]
    #[serde(default)]
    pub title: ::std::option::Option<::std::string::String>,
    /// Optional. String indicating the location of the expression for error reporting, e.g. a file
    /// name and a position in the file.
    #[builder(setter(into))]
    #[serde(default)]
    pub location: ::std::option::Option<::std::string::String>,
    /// Textual representation of an expression in Common Expression Language syntax.
    #[builder(setter(into))]
    #[serde(default)]
    pub expression: ::std::string::String,
}

#[derive(::serde::Deserialize, ::serde::Serialize, Debug, PartialEq, Clone)]
pub enum TargetTypes {
    /// This entry applies to views in the dataset.
    #[serde(rename = "VIEWS")]
    Views,
}

#[derive(
    ::serde::Deserialize,
    ::serde::Serialize,
    ::typed_builder::TypedBuilder,
    Debug,
    PartialEq,
    Clone,
    Default,
)]
#[serde(rename_all = "camelCase")]
pub struct DatasetAccessEntry {
    #[builder(setter(into))]
    #[serde(default)]
    pub target_types: ::std::vec::Vec<TargetTypes>,
    /// [Required] The dataset this entry applies to.
    #[builder(setter(into))]
    #[serde(default)]
    pub dataset: DatasetReference,
}

/// [TrustedTester] [Required] Defines the ranges for range partitioning.
#[derive(
    ::serde::Deserialize,
    ::serde::Serialize,
    ::typed_builder::TypedBuilder,
    Debug,
    PartialEq,
    Clone,
    Default,
)]
#[serde(rename_all = "camelCase")]
pub struct Range {
    /// [TrustedTester] [Required] The start of range partitioning, inclusive.
    #[builder(setter(into))]
    #[serde(with = "with::int64")]
    #[serde(default)]
    pub start: i64,
    /// [TrustedTester] [Required] The end of range partitioning, exclusive.
    #[builder(setter(into))]
    #[serde(with = "with::int64")]
    #[serde(default)]
    pub end: i64,
    /// [TrustedTester] [Required] The width of each interval.
    #[builder(setter(into))]
    #[serde(with = "with::int64")]
    #[serde(default)]
    pub interval: i64,
}

#[derive(
    ::serde::Deserialize,
    ::serde::Serialize,
    ::typed_builder::TypedBuilder,
    Debug,
    PartialEq,
    Clone,
    Default,
)]
#[serde(rename_all = "camelCase")]
pub struct RangePartitioning {
    /// [TrustedTester] [Required] The table is partitioned by this field. The field must be a
    /// top-level NULLABLE/REQUIRED field. The only supported type is INTEGER/INT64.
    #[builder(setter(into))]
    #[serde(default)]
    pub field: ::std::string::String,
    /// [TrustedTester] [Required] Defines the ranges for range partitioning.
    #[builder(setter(into))]
    #[serde(default)]
    pub range: Range,
}

#[derive(
    ::serde::Deserialize,
    ::serde::Serialize,
    ::typed_builder::TypedBuilder,
    Debug,
    PartialEq,
    Clone,
    Default,
)]
#[serde(rename_all = "camelCase")]
pub struct GetQueryResultsResponse {
    /// Whether the query has completed or not. If rows or totalRows are present, this will always
    /// be true. If this is false, totalRows will not be available.
    #[builder(setter(into))]
    #[serde(default)]
    pub job_complete: bool,
    /// The schema of the results. Present only when the query completes successfully.
    #[builder(setter(into))]
    #[serde(default)]
    pub schema: TableSchema,
    /// The total number of rows in the complete query result set, which can be more than the
    /// number of rows in this single page of results. Present only when the query completes
    /// successfully.
    #[builder(setter(into))]
    #[serde(with = "with::uint64")]
    #[serde(default)]
    pub total_rows: u64,
    /// A hash of this response.
    #[builder(setter(into))]
    #[serde(default)]
    pub etag: ::std::string::String,
    /// The total number of bytes processed for this query.
    #[builder(setter(into))]
    #[serde(with = "with::int64")]
    #[serde(default)]
    pub total_bytes_processed: i64,
    /// [Output-only] The first errors or warnings encountered during the running of the job. The
    /// final message includes the number of errors that caused the process to stop. Errors
    /// here do not necessarily mean that the job has completed or was unsuccessful.
    #[builder(setter(into))]
    #[serde(default)]
    pub errors: ::std::vec::Vec<ErrorProto>,
    /// [Output-only] The number of rows affected by a DML statement. Present only for DML
    /// statements INSERT, UPDATE or DELETE.
    #[builder(setter(into))]
    #[serde(with = "with::int64::option")]
    #[serde(default)]
    pub num_dml_affected_rows: ::std::option::Option<i64>,
    /// Whether the query result was fetched from the query cache.
    #[builder(setter(into))]
    #[serde(default)]
    pub cache_hit: bool,
    /// A token used for paging results.
    #[builder(setter(into))]
    #[serde(default)]
    pub page_token: ::std::string::String,
    /// Reference to the BigQuery Job that was created to run the query. This field will be present
    /// even if the original request timed out, in which case GetQueryResults can be used to
    /// read the results once the query has completed. Since this API only returns the first
    /// page of results, subsequent pages can be fetched via the same mechanism
    /// (GetQueryResults).
    #[builder(setter(into))]
    #[serde(default)]
    pub job_reference: JobReference,
    /// The resource type of the response.
    #[builder(setter(into))]
    #[serde(default)]
    pub kind: ::std::string::String,
    /// An object with as many results as can be contained within the maximum permitted reply size.
    /// To get any additional rows, you can call GetQueryResults and specify the jobReference
    /// returned above. Present only when the query completes successfully.
    #[builder(setter(into))]
    #[serde(default)]
    pub rows: ::std::vec::Vec<TableRow>,
}

/// An Identity and Access Management (IAM) policy, which specifies access controls for Google Cloud
/// resources. A `Policy` is a collection of `bindings`. A `binding` binds one or more `members`, or
/// principals, to a single `role`. Principals can be user accounts, service accounts, Google
/// groups, and domains (such as G Suite). A `role` is a named list of permissions; each `role` can
/// be an IAM predefined role or a user-created custom role. For some types of Google Cloud
/// resources, a `binding` can also specify a `condition`, which is a logical expression that allows
/// access to a resource only if the expression evaluates to `true`. A condition can add constraints
/// based on attributes of the request, the resource, or both. To learn which resources support
/// conditions in their IAM policies, see the [IAM
/// documentation](https://cloud.google.com/iam/help/conditions/resource-policies). **JSON
/// example:** { "bindings": [ { "role": "roles/resourcemanager.organizationAdmin", "members": [
/// "user:mike@example.com", "group:admins@example.com", "domain:google.com",
/// "serviceAccount:my-project-id@appspot.gserviceaccount.com" ] }, { "role":
/// "roles/resourcemanager.organizationViewer", "members": [ "user:eve@example.com" ], "condition":
/// { "title": "expirable access", "description": "Does not grant access after Sep 2020",
/// "expression": "request.time < timestamp('2020-10-01T00:00:00.000Z')", } } ], "etag":
/// "BwWWja0YfJA=", "version": 3 } **YAML example:** bindings: - members: - user:mike@example.com -
/// group:admins@example.com - domain:google.com -
/// serviceAccount:my-project-id@appspot.gserviceaccount.com role:
/// roles/resourcemanager.organizationAdmin - members: - user:eve@example.com role:
/// roles/resourcemanager.organizationViewer condition: title: expirable access description: Does
/// not grant access after Sep 2020 expression: request.time < timestamp('2020-10-01T00:00:00.000Z')
/// etag: BwWWja0YfJA= version: 3 For a description of IAM and its features, see the [IAM
/// documentation](https://cloud.google.com/iam/docs/).
#[derive(
    ::serde::Deserialize,
    ::serde::Serialize,
    ::typed_builder::TypedBuilder,
    Debug,
    PartialEq,
    Clone,
    Default,
)]
#[serde(rename_all = "camelCase")]
pub struct Policy {
    /// Specifies the format of the policy. Valid values are `0`, `1`, and `3`. Requests that
    /// specify an invalid value are rejected. Any operation that affects conditional role
    /// bindings must specify version `3`. This requirement applies to the following
    /// operations: * Getting a policy that includes a conditional role binding * Adding a
    /// conditional role binding to a policy * Changing a conditional role binding in a policy
    /// * Removing any role binding, with or without a condition, from a policy that includes
    /// conditions **Important:** If you use IAM Conditions, you must include the `etag` field
    /// whenever you call `setIamPolicy`. If you omit this field, then IAM allows you to
    /// overwrite a version `3` policy with a version `1` policy, and all of the conditions in
    /// the version `3` policy are lost. If a policy does not include any conditions,
    /// operations on that policy may specify any valid version or leave the field unset. To learn
    /// which resources support conditions in their IAM policies, see the [IAM
    /// documentation](https://cloud.google.com/iam/help/conditions/resource-policies).
    #[builder(setter(into))]
    #[serde(default)]
    pub version: i64,
    /// Specifies cloud audit logging configuration for this policy.
    #[builder(setter(into))]
    #[serde(default)]
    pub audit_configs: ::std::vec::Vec<AuditConfig>,
    /// Associates a list of `members`, or principals, with a `role`. Optionally, may specify a
    /// `condition` that determines how and when the `bindings` are applied. Each of the `bindings`
    /// must contain at least one principal. The `bindings` in a `Policy` can refer to up to
    /// 1,500 principals; up to 250 of these principals can be Google groups. Each occurrence
    /// of a principal counts towards these limits. For example, if the `bindings` grant 50
    /// different roles to `user:alice@example.com`, and not to any other principal, then you
    /// can add another 1,450 principals to the `bindings` in the `Policy`.
    #[builder(setter(into))]
    #[serde(default)]
    pub bindings: ::std::vec::Vec<Binding>,
    /// `etag` is used for optimistic concurrency control as a way to help prevent simultaneous
    /// updates of a policy from overwriting each other. It is strongly suggested that systems
    /// make use of the `etag` in the read-modify-write cycle to perform policy updates in
    /// order to avoid race conditions: An `etag` is returned in the response to
    /// `getIamPolicy`, and systems are expected to put that etag in the request to
    /// `setIamPolicy` to ensure that their change will be applied to the same version of the
    /// policy. **Important:** If you use IAM Conditions, you must include the `etag` field
    /// whenever you call `setIamPolicy`. If you omit this field, then IAM allows you to
    /// overwrite a version `3` policy with a version `1` policy, and all of the conditions in the
    /// version `3` policy are lost.
    #[builder(setter(into))]
    #[serde(default)]
    pub etag: ::bytes::Bytes,
}

#[derive(
    ::serde::Deserialize,
    ::serde::Serialize,
    ::typed_builder::TypedBuilder,
    Debug,
    PartialEq,
    Clone,
    Default,
)]
#[serde(rename_all = "camelCase")]
pub struct GetServiceAccountResponse {
    /// The service account email address.
    #[builder(setter(into))]
    #[serde(default)]
    pub email: ::std::string::String,
    /// The resource type of the response.
    #[builder(setter(into))]
    #[serde(default)]
    pub kind: ::std::string::String,
}

/// The labels associated with this job. You can use these to organize and group your jobs. Label
/// keys and values can be no longer than 63 characters, can only contain lowercase letters, numeric
/// characters, underscores and dashes. International characters are allowed. Label values are
/// optional. Label keys must start with a letter and each label in the list must have a different
/// key.
pub type Labels = ::std::collections::HashMap<::std::string::String, ::std::string::String>;

#[derive(
    ::serde::Deserialize,
    ::serde::Serialize,
    ::typed_builder::TypedBuilder,
    Debug,
    PartialEq,
    Clone,
    Default,
)]
#[serde(rename_all = "camelCase")]
pub struct QueryRequest {
    /// [Optional] Specifies the default datasetId and projectId to assume for any unqualified
    /// table names in the query. If not set, all table names in the query string must be
    /// qualified in the format 'datasetId.tableId'.
    #[builder(setter(into))]
    #[serde(default)]
    pub default_dataset: ::std::option::Option<DatasetReference>,
    /// [Optional] Limits the bytes billed for this job. Queries that will have bytes billed beyond
    /// this limit will fail (without incurring a charge). If unspecified, this will be set to
    /// your project default.
    #[builder(setter(into))]
    #[serde(with = "with::int64::option")]
    #[serde(default)]
    pub maximum_bytes_billed: ::std::option::Option<i64>,
    /// If true, creates a new session, where session id will be a server generated random id. If
    /// false, runs query with an existing session_id passed in ConnectionProperty, otherwise
    /// runs query in non-session mode.
    #[builder(setter(into))]
    #[serde(default)]
    pub create_session: bool,
    /// [Optional] Whether to look for the result in the query cache. The query cache is a
    /// best-effort cache that will be flushed whenever tables in the query are modified. The
    /// default value is true.
    #[builder(setter(into))]
    #[serde(default)]
    pub use_query_cache: ::std::option::Option<bool>,
    /// Specifies whether to use BigQuery's legacy SQL dialect for this query. The default value is
    /// true. If set to false, the query will use BigQuery's standard SQL:
    /// https://cloud.google.com/bigquery/sql-reference/ When useLegacySql is set to false, the value of
    /// flattenResults is ignored; query will be run as if flattenResults is false.
    #[builder(setter(into))]
    #[serde(default)]
    pub use_legacy_sql: bool,
    /// [Required] A query string, following the BigQuery query syntax, of the query to execute.
    /// Example: "SELECT count(f1) FROM [myProjectId:myDatasetId.myTableId]".
    #[builder(setter(into))]
    #[serde(default)]
    pub query: ::std::string::String,
    /// The resource type of the request.
    #[builder(setter(into))]
    #[serde(default)]
    pub kind: ::std::string::String,
    /// Connection properties.
    #[builder(setter(into))]
    #[serde(default)]
    pub connection_properties: ::std::vec::Vec<ConnectionProperty>,
    /// Standard SQL only. Set to POSITIONAL to use positional (?) query parameters or to NAMED to
    /// use named (@myparam) query parameters in this query.
    #[builder(setter(into))]
    #[serde(default)]
    pub parameter_mode: ::std::string::String,
    /// A unique user provided identifier to ensure idempotent behavior for queries. Note that this
    /// is different from the job_id. It has the following properties: 1. It is case-sensitive,
    /// limited to up to 36 ASCII characters. A UUID is recommended. 2. Read only queries can
    /// ignore this token since they are nullipotent by definition. 3. For the purposes of
    /// idempotency ensured by the request_id, a request is considered duplicate of another
    /// only if they have the same request_id and are actually duplicates. When determining
    /// whether a request is a duplicate of the previous request, all parameters in the request
    /// that may affect the behavior are considered. For example, query, connection_properties,
    /// query_parameters, use_legacy_sql are parameters that affect the result and are
    /// considered when determining whether a request is a duplicate, but properties like
    /// timeout_ms don't affect the result and are thus not considered. Dry run query requests are
    /// never considered duplicate of another request. 4. When a duplicate mutating query
    /// request is detected, it returns: a. the results of the mutation if it completes
    /// successfully within the timeout. b. the running operation if it is still in progress at
    /// the end of the timeout. 5. Its lifetime is limited to 15 minutes. In other words, if
    /// two requests are sent with the same request_id, but more than 15 minutes apart,
    /// idempotency is not guaranteed.
    #[builder(setter(into))]
    #[serde(default)]
    pub request_id: ::std::string::String,
    /// [Optional] The maximum number of rows of data to return per page of results. Setting this
    /// flag to a small value such as 1000 and then paging through results might improve
    /// reliability when the query result set is large. In addition to this limit, responses
    /// are also limited to 10 MB. By default, there is no maximum row count, and only the byte
    /// limit applies.
    #[builder(setter(into))]
    #[serde(default)]
    pub max_results: ::std::option::Option<i64>,
    /// [Deprecated] This property is deprecated.
    #[builder(setter(into))]
    #[serde(default)]
    pub preserve_nulls: bool,
    /// The geographic location where the job should run. See details at
    /// https://cloud.google.com/bigquery/docs/locations#specifying_your_location.
    #[builder(setter(into))]
    #[serde(default)]
    pub location: ::std::string::String,
    /// Query parameters for Standard SQL queries.
    #[builder(setter(into))]
    #[serde(default)]
    pub query_parameters: ::std::vec::Vec<QueryParameter>,
    /// The labels associated with this job. You can use these to organize and group your jobs.
    /// Label keys and values can be no longer than 63 characters, can only contain lowercase
    /// letters, numeric characters, underscores and dashes. International characters are
    /// allowed. Label values are optional. Label keys must start with a letter and each label
    /// in the list must have a different key.
    #[builder(setter(into))]
    #[serde(default)]
    pub labels: Labels,
    /// [Optional] How long to wait for the query to complete, in milliseconds, before the request
    /// times out and returns. Note that this is only a timeout for the request, not the query.
    /// If the query takes longer to run than the timeout value, the call returns without any
    /// results and with the 'jobComplete' flag set to false. You can call GetQueryResults() to
    /// wait for the query to complete and read the results. The default value is 10000
    /// milliseconds (10 seconds).
    #[builder(setter(into))]
    #[serde(default)]
    pub timeout_ms: ::std::option::Option<i64>,
    /// [Optional] If set to true, BigQuery doesn't run the job. Instead, if the query is valid,
    /// BigQuery returns statistics about the job such as how many bytes would be processed. If the
    /// query is invalid, an error returns. The default value is false.
    #[builder(setter(into))]
    #[serde(default)]
    pub dry_run: ::std::option::Option<bool>,
}

#[derive(
    ::serde::Deserialize,
    ::serde::Serialize,
    ::typed_builder::TypedBuilder,
    Debug,
    PartialEq,
    Clone,
    Default,
)]
#[serde(rename_all = "camelCase")]
pub struct BigQueryModelTraining {
    /// [Output-only, Beta] Index of current ML training iteration. Updated during create model
    /// query job to show job progress.
    #[builder(setter(into))]
    #[serde(default)]
    pub current_iteration: ::std::option::Option<i64>,
    /// [Output-only, Beta] Expected number of iterations for the create model query job specified
    /// as num_iterations in the input query. The actual total number of iterations may be less
    /// than this number due to early stop.
    #[builder(setter(into))]
    #[serde(with = "with::int64::option")]
    #[serde(default)]
    pub expected_total_iterations: ::std::option::Option<i64>,
}

#[derive(
    ::serde::Deserialize,
    ::serde::Serialize,
    ::typed_builder::TypedBuilder,
    Debug,
    PartialEq,
    Clone,
    Default,
)]
#[serde(rename_all = "camelCase")]
pub struct ModelReference {
    /// [Required] The ID of the model. The ID must contain only letters (a-z, A-Z), numbers (0-9),
    /// or underscores (_). The maximum length is 1,024 characters.
    #[builder(setter(into))]
    #[serde(default)]
    pub model_id: ::std::string::String,
    /// [Required] The ID of the project containing this model.
    #[builder(setter(into))]
    #[serde(default)]
    pub project_id: ::std::string::String,
    /// [Required] The ID of the dataset containing this model.
    #[builder(setter(into))]
    #[serde(default)]
    pub dataset_id: ::std::string::String,
}

#[derive(
    ::serde::Deserialize,
    ::serde::Serialize,
    ::typed_builder::TypedBuilder,
    Debug,
    PartialEq,
    Clone,
    Default,
)]
#[serde(rename_all = "camelCase")]
pub struct Tags {
    /// [Required] Friendly short name of the tag value, e.g. "production".
    #[builder(setter(into))]
    #[serde(default)]
    pub tag_value: ::std::string::String,
    /// [Required] The namespaced friendly name of the tag key, e.g. "12345/environment" where
    /// 12345 is org id.
    #[builder(setter(into))]
    #[serde(default)]
    pub tag_key: ::std::string::String,
}

#[derive(
    ::serde::Deserialize, ::serde::Serialize, ::typed_builder::TypedBuilder, Debug, PartialEq, Clone,
)]
#[serde(rename_all = "camelCase")]
pub struct Access {
    /// [Pick one] A special group to grant access to. Possible values include: projectOwners:
    /// Owners of the enclosing project. projectReaders: Readers of the enclosing project.
    /// projectWriters: Writers of the enclosing project. allAuthenticatedUsers: All
    /// authenticated BigQuery users. Maps to similarly-named IAM members.
    #[builder(setter(into))]
    #[serde(default)]
    pub special_group: ::std::string::String,
    /// [Pick one] A routine from a different dataset to grant access to. Queries executed against
    /// that routine will have read access to views/tables/routines in this dataset. Only UDF
    /// is supported for now. The role field is not required when this field is set. If that
    /// routine is updated by any user, access to the routine needs to be granted again via an
    /// update operation.
    #[builder(setter(into))]
    #[serde(default)]
    pub routine: RoutineReference,
    /// [Pick one] A grant authorizing all resources of a particular type in a particular dataset
    /// access to this dataset. Only views are supported for now. The role field is not
    /// required when this field is set. If that dataset is deleted and re-created, its access
    /// needs to be granted again via an update operation.
    #[builder(setter(into))]
    pub dataset: DatasetAccessEntry,
    /// [Pick one] An email address of a user to grant access to. For example: fred@example.com.
    /// Maps to IAM policy member "user:EMAIL" or "serviceAccount:EMAIL".
    #[builder(setter(into))]
    #[serde(default)]
    pub user_by_email: ::std::string::String,
    /// [Pick one] A view from a different dataset to grant access to. Queries executed against
    /// that view will have read access to tables in this dataset. The role field is not
    /// required when this field is set. If that view is updated by any user, access to the
    /// view needs to be granted again via an update operation.
    #[builder(setter(into))]
    #[serde(default)]
    pub view: TableReference,
    /// [Pick one] An email address of a Google Group to grant access to. Maps to IAM policy member
    /// "group:GROUP".
    #[builder(setter(into))]
    #[serde(default)]
    pub group_by_email: ::std::string::String,
    /// [Required] An IAM role ID that should be granted to the user, group, or domain specified in
    /// this access entry. The following legacy mappings will be applied: OWNER
    /// roles/bigquery.dataOwner WRITER roles/bigquery.dataEditor READER
    /// roles/bigquery.dataViewer This field will accept any of the above formats, but will
    /// return only the legacy format. For example, if you set this field to "roles/bigquery.
    /// dataOwner", it will be returned back as "OWNER".
    #[builder(setter(into))]
    #[serde(default)]
    pub role: ::std::string::String,
    /// [Pick one] Some other type of member that appears in the IAM Policy but isn't a user,
    /// group, domain, or special group.
    #[builder(setter(into))]
    #[serde(default)]
    pub iam_member: ::std::string::String,
    /// [Pick one] A domain to grant access to. Any users signed in with the domain specified will
    /// be granted the specified access. Example: "example.com". Maps to IAM policy member
    /// "domain:DOMAIN".
    #[builder(setter(into))]
    #[serde(default)]
    pub domain: ::std::string::String,
}

#[derive(
    ::serde::Deserialize,
    ::serde::Serialize,
    ::typed_builder::TypedBuilder,
    Debug,
    PartialEq,
    Clone,
    Default,
)]
#[serde(rename_all = "camelCase")]
pub struct Dataset {
    /// [Output-only] The resource type.
    #[builder(setter(into))]
    #[serde(default)]
    pub kind: ::std::option::Option<::std::string::String>,
    /// [Output-only] Reserved for future use.
    #[builder(setter(into))]
    #[serde(default)]
    pub satisfies_pzs: ::std::option::Option<bool>,
    /// [Output-only] The fully-qualified unique name of the dataset in the format
    /// projectId:datasetId. The dataset name without the project name is given in the
    /// datasetId field. When creating a new dataset, leave this field blank, and instead
    /// specify the datasetId field.
    #[builder(setter(into))]
    #[serde(default)]
    pub id: ::std::option::Option<::std::string::String>,
    /// [Optional] Indicates if table names are case insensitive in the dataset.
    #[builder(setter(into))]
    #[serde(default)]
    pub is_case_insensitive: ::std::option::Option<bool>,
    /// [Optional] Number of hours for the max time travel for all tables in the dataset.
    #[builder(setter(into))]
    #[serde(with = "with::int64::option")]
    #[serde(default)]
    pub max_time_travel_hours: ::std::option::Option<i64>,
    /// [Optional]The tags associated with this dataset. Tag keys are globally unique.
    #[builder(setter(into))]
    #[serde(default)]
    pub tags: ::std::vec::Vec<Tags>,
    /// [Output-only] A URL that can be used to access the resource again. You can use this URL in
    /// Get or Update requests to the resource.
    #[builder(setter(into))]
    #[serde(default)]
    pub self_link: ::std::option::Option<::std::string::String>,
    /// [Output-only] The default collation of the dataset.
    #[builder(setter(into))]
    #[serde(default)]
    pub default_collation: ::std::option::Option<::std::string::String>,
    /// The geographic location where the dataset should reside. The default value is US. See
    /// details at https://cloud.google.com/bigquery/docs/locations.
    #[builder(setter(into))]
    #[serde(default)]
    pub location: ::std::string::String,
    /// The labels associated with this dataset. You can use these to organize and group your
    /// datasets. You can set this property when inserting or updating a dataset. See Creating
    /// and Updating Dataset Labels for more information.
    #[builder(setter(into))]
    #[serde(default)]
    pub labels: Labels,
    /// [Optional] The default lifetime of all tables in the dataset, in milliseconds. The minimum
    /// value is 3600000 milliseconds (one hour). Once this property is set, all newly-created
    /// tables in the dataset will have an expirationTime property set to the creation time
    /// plus the value in this property, and changing the value will only affect new tables,
    /// not existing ones. When the expirationTime for a given table is reached, that table
    /// will be deleted automatically. If a table's expirationTime is modified or removed
    /// before the table expires, or if you provide an explicit expirationTime when creating a
    /// table, that value takes precedence over the default expiration time indicated by this
    /// property.
    #[builder(setter(into))]
    #[serde(with = "with::int64::option")]
    #[serde(default)]
    pub default_table_expiration_ms: ::std::option::Option<i64>,
    /// [Optional] An array of objects that define dataset access for one or more entities. You can
    /// set this property when inserting or updating a dataset in order to control who is
    /// allowed to access the data. If unspecified at dataset creation time, BigQuery adds
    /// default dataset access for the following entities: access.specialGroup: projectReaders;
    /// access.role: READER; access.specialGroup: projectWriters; access.role: WRITER;
    /// access.specialGroup: projectOwners; access.role: OWNER; access.userByEmail: [dataset
    /// creator email]; access.role: OWNER;
    #[builder(setter(into))]
    #[serde(default)]
    pub access: ::std::vec::Vec<Access>,
    #[builder(setter(into))]
    #[serde(default)]
    pub default_encryption_configuration: EncryptionConfiguration,
    /// [Output-only] The time when this dataset was created, in milliseconds since the epoch.
    #[builder(setter(into))]
    #[serde(with = "with::int64::option")]
    #[serde(default)]
    pub creation_time: ::std::option::Option<i64>,
    /// [Required] A reference that identifies the dataset.
    #[builder(setter(into))]
    #[serde(default)]
    pub dataset_reference: DatasetReference,
    /// [Optional] The default partition expiration for all partitioned tables in the dataset, in
    /// milliseconds. Once this property is set, all newly-created partitioned tables in the
    /// dataset will have an expirationMs property in the timePartitioning settings set to this
    /// value, and changing the value will only affect new tables, not existing ones. The
    /// storage in a partition will have an expiration time of its partition time plus this
    /// value. Setting this property overrides the use of defaultTableExpirationMs for
    /// partitioned tables: only one of defaultTableExpirationMs and
    /// defaultPartitionExpirationMs will be used for any new partitioned table. If you provide
    /// an explicit timePartitioning.expirationMs when creating or updating a partitioned
    /// table, that value takes precedence over the default partition expiration time indicated
    /// by this property.
    #[builder(setter(into))]
    #[serde(with = "with::int64::option")]
    #[serde(default)]
    pub default_partition_expiration_ms: ::std::option::Option<i64>,
    /// [Output-only] A hash of the resource.
    #[builder(setter(into))]
    #[serde(default)]
    pub etag: ::std::option::Option<::std::string::String>,
    /// [Output-only] The date when this dataset or any of its tables was last modified, in
    /// milliseconds since the epoch.
    #[builder(setter(into))]
    #[serde(with = "with::int64::option")]
    #[serde(default)]
    pub last_modified_time: ::std::option::Option<i64>,
    /// [Optional] A user-friendly description of the dataset.
    #[builder(setter(into))]
    #[serde(default)]
    pub description: ::std::option::Option<::std::string::String>,
    /// [Optional] A descriptive name for the dataset.
    #[builder(setter(into))]
    #[serde(default)]
    pub friendly_name: ::std::option::Option<::std::string::String>,
}

#[derive(
    ::serde::Deserialize,
    ::serde::Serialize,
    ::typed_builder::TypedBuilder,
    Debug,
    PartialEq,
    Clone,
    Default,
)]
#[serde(rename_all = "camelCase")]
pub struct InsertErrors {
    /// The index of the row that error applies to.
    #[builder(setter(into))]
    #[serde(default)]
    pub index: i64,
    /// Error information for the row indicated by the index property.
    #[builder(setter(into))]
    #[serde(default)]
    pub errors: ::std::vec::Vec<ErrorProto>,
}

#[derive(
    ::serde::Deserialize,
    ::serde::Serialize,
    ::typed_builder::TypedBuilder,
    Debug,
    PartialEq,
    Clone,
    Default,
)]
#[serde(rename_all = "camelCase")]
pub struct TableDataInsertAllResponse {
    /// An array of errors for rows that were not inserted.
    #[builder(setter(into))]
    #[serde(default)]
    pub insert_errors: ::std::vec::Vec<InsertErrors>,
    /// The resource type of the response.
    #[builder(setter(into))]
    #[serde(default)]
    pub kind: ::std::string::String,
}

/// Optional. Defaults to FIXED_TYPE.
#[derive(::serde::Deserialize, ::serde::Serialize, Debug, PartialEq, Clone)]
pub enum ArgumentKind {
    /// The argument is a variable with fully specified type, which can be a struct or an array,
    /// but not a table.
    #[serde(rename = "FIXED_TYPE")]
    FixedType,
    /// The argument is any type, including struct or array, but not a table. To be added:
    /// FIXED_TABLE, ANY_TABLE
    #[serde(rename = "ANY_TYPE")]
    AnyType,
}

/// Optional. Specifies whether the argument is input or output. Can be set for procedures only.
#[derive(::serde::Deserialize, ::serde::Serialize, Debug, PartialEq, Clone)]
pub enum Mode {
    /// The argument is input-only.
    #[serde(rename = "IN")]
    In,
    /// The argument is output-only.
    #[serde(rename = "OUT")]
    Out,
    /// The argument is both an input and an output.
    #[serde(rename = "INOUT")]
    Inout,
}

/// Input/output argument of a function or a stored procedure.
#[derive(
    ::serde::Deserialize, ::serde::Serialize, ::typed_builder::TypedBuilder, Debug, PartialEq, Clone,
)]
#[serde(rename_all = "camelCase")]
pub struct Argument {
    /// Optional. The name of this argument. Can be absent for function return argument.
    #[builder(setter(into))]
    #[serde(default)]
    pub name: ::std::option::Option<::std::string::String>,
    /// Optional. Defaults to FIXED_TYPE.
    #[builder(setter(into))]
    pub argument_kind: ArgumentKind,
    /// Required unless argument_kind = ANY_TYPE.
    #[builder(setter(into))]
    pub data_type: StandardSqlDataType,
    /// Optional. Specifies whether the argument is input or output. Can be set for procedures
    /// only.
    #[builder(setter(into))]
    pub mode: Mode,
}

/// Search space for a double hyperparameter.
#[derive(
    ::serde::Deserialize,
    ::serde::Serialize,
    ::typed_builder::TypedBuilder,
    Debug,
    PartialEq,
    Clone,
    Default,
)]
#[serde(rename_all = "camelCase")]
pub struct DoubleHparamSearchSpace {
    /// Candidates of the double hyperparameter.
    #[builder(setter(into))]
    #[serde(default)]
    pub candidates: DoubleCandidates,
    /// Range of the double hyperparameter.
    #[builder(setter(into))]
    #[serde(default)]
    pub range: DoubleRange,
}

/// Evaluation metrics for clustering models.
#[derive(
    ::serde::Deserialize,
    ::serde::Serialize,
    ::typed_builder::TypedBuilder,
    Debug,
    PartialEq,
    Clone,
    Default,
)]
#[serde(rename_all = "camelCase")]
pub struct ClusteringMetrics {
    /// Mean of squared distances between each sample to its cluster centroid.
    #[builder(setter(into))]
    #[serde(default)]
    pub mean_squared_distance: f64,
    /// Davies-Bouldin index.
    #[builder(setter(into))]
    #[serde(default)]
    pub davies_bouldin_index: f64,
    /// Information for all clusters.
    #[builder(setter(into))]
    #[serde(default)]
    pub clusters: ::std::vec::Vec<Cluster>,
}

/// [Optional] The categories attached to this field, used for field-level access control.
#[derive(
    ::serde::Deserialize,
    ::serde::Serialize,
    ::typed_builder::TypedBuilder,
    Debug,
    PartialEq,
    Clone,
    Default,
)]
#[serde(rename_all = "camelCase")]
pub struct Categories {
    /// A list of category resource names. For example, "projects/1/taxonomies/2/categories/3". At
    /// most 5 categories are allowed.
    #[builder(setter(into))]
    #[serde(default)]
    pub names: ::std::vec::Vec<::std::string::String>,
}

#[derive(
    ::serde::Deserialize,
    ::serde::Serialize,
    ::typed_builder::TypedBuilder,
    Debug,
    PartialEq,
    Clone,
    Default,
)]
#[serde(rename_all = "camelCase")]
pub struct PolicyTags {
    /// A list of category resource names. For example,
    /// "projects/1/location/eu/taxonomies/2/policyTags/3". At most 1 policy tag is allowed.
    #[builder(setter(into))]
    #[serde(default)]
    pub names: ::std::vec::Vec<::std::string::String>,
}

#[derive(
    ::serde::Deserialize,
    ::serde::Serialize,
    ::typed_builder::TypedBuilder,
    Debug,
    PartialEq,
    Clone,
    Default,
)]
#[serde(rename_all = "camelCase")]
pub struct TableFieldSchema {
    /// [Optional] The categories attached to this field, used for field-level access control.
    #[builder(setter(into))]
    #[serde(default)]
    pub categories: Categories,
    /// [Required] The field data type. Possible values include STRING, BYTES, INTEGER, INT64 (same
    /// as INTEGER), FLOAT, FLOAT64 (same as FLOAT), NUMERIC, BIGNUMERIC, BOOLEAN, BOOL (same
    /// as BOOLEAN), TIMESTAMP, DATE, TIME, DATETIME, INTERVAL, RECORD (where RECORD indicates
    /// that the field contains a nested schema) or STRUCT (same as RECORD).
    #[builder(setter(into))]
    #[serde(rename = "type")]
    #[serde(default)]
    pub ty: ::std::string::String,
    /// [Optional] See documentation for precision.
    #[builder(setter(into))]
    #[serde(with = "with::int64::option")]
    #[serde(default)]
    pub scale: ::std::option::Option<i64>,
    /// [Optional] The field description. The maximum length is 1,024 characters.
    #[builder(setter(into))]
    #[serde(default)]
    pub description: ::std::option::Option<::std::string::String>,
    /// Optional. A SQL expression to specify the default value for this field. It can only be set
    /// for top level fields (columns). You can use struct or array expression to specify
    /// default value for the entire struct or array. The valid SQL expressions are: - Literals
    /// for all data types, including STRUCT and ARRAY. - Following functions: -
    /// CURRENT_TIMESTAMP - CURRENT_TIME - CURRENT_DATE - CURRENT_DATETIME - GENERATE_UUID -
    /// RAND - SESSION_USER - ST_GEOGPOINT - Struct or array composed with the above allowed
    /// functions, for example, [CURRENT_DATE(), DATE '2020-01-01']
    #[builder(setter(into))]
    #[serde(default)]
    pub default_value_expression: ::std::option::Option<::std::string::String>,
    /// [Optional] The field mode. Possible values include NULLABLE, REQUIRED and REPEATED. The
    /// default value is NULLABLE.
    #[builder(setter(into))]
    #[serde(default)]
    pub mode: ::std::option::Option<::std::string::String>,
    /// Optional. Collation specification of the field. It only can be set on string type field.
    #[builder(setter(into))]
    #[serde(default)]
    pub collation: ::std::option::Option<::std::string::String>,
    /// [Optional] Precision (maximum number of total digits in base 10) and scale (maximum number
    /// of digits in the fractional part in base 10) constraints for values of this field for
    /// NUMERIC or BIGNUMERIC. It is invalid to set precision or scale if type ≠ "NUMERIC" and
    /// ≠ "BIGNUMERIC". If precision and scale are not specified, no value range constraint is
    /// imposed on this field insofar as values are permitted by the type. Values of this
    /// NUMERIC or BIGNUMERIC field must be in this range when: - Precision (P) and scale (S)
    /// are specified: [-10P-S + 10-S, 10P-S - 10-S] - Precision (P) is specified but not scale
    /// (and thus scale is interpreted to be equal to zero): [-10P + 1, 10P - 1]. Acceptable
    /// values for precision and scale if both are specified: - If type = "NUMERIC": 1 ≤
    /// precision - scale ≤ 29 and 0 ≤ scale ≤ 9. - If type = "BIGNUMERIC": 1 ≤ precision -
    /// scale ≤ 38 and 0 ≤ scale ≤ 38. Acceptable values for precision if only precision is
    /// specified but not scale (and thus scale is interpreted to be equal to zero): - If
    /// type = "NUMERIC": 1 ≤ precision ≤ 29. - If type = "BIGNUMERIC": 1 ≤ precision ≤ 38. If
    /// scale is specified but not precision, then it is invalid.
    #[builder(setter(into))]
    #[serde(with = "with::int64::option")]
    #[serde(default)]
    pub precision: ::std::option::Option<i64>,
    /// [Optional] Maximum length of values of this field for STRINGS or BYTES. If max_length is
    /// not specified, no maximum length constraint is imposed on this field. If type =
    /// "STRING", then max_length represents the maximum UTF-8 length of strings in this field.
    /// If type = "BYTES", then max_length represents the maximum number of bytes in this
    /// field. It is invalid to set this field if type ≠ "STRING" and ≠ "BYTES".
    #[builder(setter(into))]
    #[serde(with = "with::int64::option")]
    #[serde(default)]
    pub max_length: ::std::option::Option<i64>,
    /// [Optional] Describes the nested schema fields if the type property is set to RECORD.
    #[builder(setter(into))]
    #[serde(default)]
    pub fields: ::std::vec::Vec<TableFieldSchema>,
    /// [Required] The field name. The name must contain only letters (a-z, A-Z), numbers (0-9), or
    /// underscores (_), and must start with a letter or underscore. The maximum length is 300
    /// characters.
    #[builder(setter(into))]
    #[serde(default)]
    pub name: ::std::string::String,
    #[builder(setter(into))]
    #[serde(default)]
    pub policy_tags: PolicyTags,
}

#[derive(
    ::serde::Deserialize,
    ::serde::Serialize,
    ::typed_builder::TypedBuilder,
    Debug,
    PartialEq,
    Clone,
    Default,
)]
#[serde(rename_all = "camelCase")]
pub struct Projects {
    /// An opaque ID of this project.
    #[builder(setter(into))]
    #[serde(default)]
    pub id: ::std::string::String,
    /// The resource type.
    #[builder(setter(into))]
    #[serde(default)]
    pub kind: ::std::string::String,
    /// A descriptive name for this project.
    #[builder(setter(into))]
    #[serde(default)]
    pub friendly_name: ::std::string::String,
    /// A unique reference to this project.
    #[builder(setter(into))]
    #[serde(default)]
    pub project_reference: ProjectReference,
    /// The numeric ID of this project.
    #[builder(setter(into))]
    #[serde(with = "with::uint64")]
    #[serde(default)]
    pub numeric_id: u64,
}

#[derive(
    ::serde::Deserialize,
    ::serde::Serialize,
    ::typed_builder::TypedBuilder,
    Debug,
    PartialEq,
    Clone,
    Default,
)]
#[serde(rename_all = "camelCase")]
pub struct ProjectList {
    /// The total number of projects in the list.
    #[builder(setter(into))]
    #[serde(default)]
    pub total_items: i64,
    /// A hash of the page of results
    #[builder(setter(into))]
    #[serde(default)]
    pub etag: ::std::string::String,
    /// A token to request the next page of results.
    #[builder(setter(into))]
    #[serde(default)]
    pub next_page_token: ::std::string::String,
    /// Projects to which you have at least READ access.
    #[builder(setter(into))]
    #[serde(default)]
    pub projects: ::std::vec::Vec<Projects>,
    /// The type of list.
    #[builder(setter(into))]
    #[serde(default)]
    pub kind: ::std::string::String,
}

/// A single row in the confusion matrix.
#[derive(
    ::serde::Deserialize,
    ::serde::Serialize,
    ::typed_builder::TypedBuilder,
    Debug,
    PartialEq,
    Clone,
    Default,
)]
#[serde(rename_all = "camelCase")]
pub struct Row {
    /// Info describing predicted label distribution.
    #[builder(setter(into))]
    #[serde(default)]
    pub entries: ::std::vec::Vec<Entry>,
    /// The original label of this row.
    #[builder(setter(into))]
    #[serde(default)]
    pub actual_label: ::std::string::String,
}

#[derive(
    ::serde::Deserialize,
    ::serde::Serialize,
    ::typed_builder::TypedBuilder,
    Debug,
    PartialEq,
    Clone,
    Default,
)]
#[serde(rename_all = "camelCase")]
pub struct ConnectionProperty {
    /// [Required] Name of the connection property to set.
    #[builder(setter(into))]
    #[serde(default)]
    pub key: ::std::string::String,
    /// [Required] Value of the connection property.
    #[builder(setter(into))]
    #[serde(default)]
    pub value: ::std::string::String,
}

/// Discrete candidates of an int hyperparameter.
#[derive(
    ::serde::Deserialize,
    ::serde::Serialize,
    ::typed_builder::TypedBuilder,
    Debug,
    PartialEq,
    Clone,
    Default,
)]
#[serde(rename_all = "camelCase")]
pub struct IntCandidates {
    /// Candidates for the int parameter in increasing order.
    #[builder(setter(into))]
    #[serde(default)]
    pub candidates: ::std::vec::Vec<i64>,
}

/// Request message for `SetIamPolicy` method.
#[derive(
    ::serde::Deserialize,
    ::serde::Serialize,
    ::typed_builder::TypedBuilder,
    Debug,
    PartialEq,
    Clone,
    Default,
)]
#[serde(rename_all = "camelCase")]
pub struct SetIamPolicyRequest {
    /// REQUIRED: The complete policy to be applied to the `resource`. The size of the policy is
    /// limited to a few 10s of KB. An empty policy is a valid policy but certain Google Cloud
    /// services (such as Projects) might reject them.
    #[builder(setter(into))]
    #[serde(default)]
    pub policy: Policy,
    /// OPTIONAL: A FieldMask specifying which fields of the policy to modify. Only the fields in
    /// the mask will be modified. If no mask is provided, the following default mask is used:
    /// `paths: "bindings, etag"`
    #[builder(setter(into))]
    #[serde(default)]
    pub update_mask: ::std::option::Option<::std::string::String>,
}

/// Arima coefficients.
#[derive(
    ::serde::Deserialize,
    ::serde::Serialize,
    ::typed_builder::TypedBuilder,
    Debug,
    PartialEq,
    Clone,
    Default,
)]
#[serde(rename_all = "camelCase")]
pub struct ArimaCoefficients {
    /// Intercept coefficient, just a double not an array.
    #[builder(setter(into))]
    #[serde(default)]
    pub intercept_coefficient: f64,
    /// Moving-average coefficients, an array of double.
    #[builder(setter(into))]
    #[serde(default)]
    pub moving_average_coefficients: ::std::vec::Vec<f64>,
    /// Auto-regressive coefficients, an array of double.
    #[builder(setter(into))]
    #[serde(default)]
    pub auto_regressive_coefficients: ::std::vec::Vec<f64>,
}

/// The status of the trial.
#[derive(::serde::Deserialize, ::serde::Serialize, Debug, PartialEq, Clone)]
pub enum Status {
    /// Scheduled but not started.
    #[serde(rename = "NOT_STARTED")]
    NotStarted,
    /// Running state.
    #[serde(rename = "RUNNING")]
    Running,
    /// The trial succeeded.
    #[serde(rename = "SUCCEEDED")]
    Succeeded,
    /// The trial failed.
    #[serde(rename = "FAILED")]
    Failed,
    /// The trial is infeasible due to the invalid params.
    #[serde(rename = "INFEASIBLE")]
    Infeasible,
    /// Trial stopped early because it's not promising.
    #[serde(rename = "STOPPED_EARLY")]
    StoppedEarly,
}

/// Training info of a trial in [hyperparameter
/// tuning](/bigquery-ml/docs/reference/standard-sql/bigqueryml-syntax-hp-tuning-overview) models.
#[derive(
    ::serde::Deserialize, ::serde::Serialize, ::typed_builder::TypedBuilder, Debug, PartialEq, Clone,
)]
#[serde(rename_all = "camelCase")]
pub struct HparamTuningTrial {
    /// Ending time of the trial.
    #[builder(setter(into))]
    #[serde(with = "with::int64")]
    #[serde(default)]
    pub end_time_ms: i64,
    /// Evaluation metrics of this trial calculated on the test data. Empty in Job API.
    #[builder(setter(into))]
    pub evaluation_metrics: EvaluationMetrics,
    /// Loss computed on the training data at the end of trial.
    #[builder(setter(into))]
    #[serde(default)]
    pub training_loss: f64,
    /// The hyperprameters selected for this trial.
    #[builder(setter(into))]
    #[serde(default)]
    pub hparams: TrainingOptions,
    /// Starting time of the trial.
    #[builder(setter(into))]
    #[serde(with = "with::int64")]
    #[serde(default)]
    pub start_time_ms: i64,
    /// The status of the trial.
    #[builder(setter(into))]
    pub status: Status,
    /// Loss computed on the eval data at the end of trial.
    #[builder(setter(into))]
    #[serde(default)]
    pub eval_loss: f64,
    /// Hyperparameter tuning evaluation metrics of this trial calculated on the eval data. Unlike
    /// evaluation_metrics, only the fields corresponding to the hparam_tuning_objectives are set.
    #[builder(setter(into))]
    pub hparam_tuning_evaluation_metrics: EvaluationMetrics,
    /// 1-based index of the trial.
    #[builder(setter(into))]
    #[serde(with = "with::int64")]
    #[serde(default)]
    pub trial_id: i64,
    /// Error message for FAILED and INFEASIBLE trial.
    #[builder(setter(into))]
    #[serde(default)]
    pub error_message: ::std::string::String,
}

#[derive(
    ::serde::Deserialize,
    ::serde::Serialize,
    ::typed_builder::TypedBuilder,
    Debug,
    PartialEq,
    Clone,
    Default,
)]
#[serde(rename_all = "camelCase")]
pub struct JobStatistics5 {
    /// [Output-only] Number of logical bytes copied to the destination table.
    #[builder(setter(into))]
    #[serde(with = "with::int64::option")]
    #[serde(default)]
    pub copied_logical_bytes: ::std::option::Option<i64>,
    /// [Output-only] Number of rows copied to the destination table.
    #[builder(setter(into))]
    #[serde(with = "with::int64::option")]
    #[serde(default)]
    pub copied_rows: ::std::option::Option<i64>,
}

/// The log type that this config enables.
#[derive(::serde::Deserialize, ::serde::Serialize, Debug, PartialEq, Clone)]
pub enum LogType {
    /// Admin reads. Example: CloudIAM getIamPolicy
    #[serde(rename = "ADMIN_READ")]
    AdminRead,
    /// Data writes. Example: CloudSQL Users create
    #[serde(rename = "DATA_WRITE")]
    DataWrite,
    /// Data reads. Example: CloudSQL Users list
    #[serde(rename = "DATA_READ")]
    DataRead,
}

/// Provides the configuration for logging a type of permissions. Example: { "audit_log_configs": [
/// { "log_type": "DATA_READ", "exempted_members": [ "user:jose@example.com" ] }, { "log_type":
/// "DATA_WRITE" } ] } This enables 'DATA_READ' and 'DATA_WRITE' logging, while exempting
/// jose@example.com from DATA_READ logging.
#[derive(
    ::serde::Deserialize, ::serde::Serialize, ::typed_builder::TypedBuilder, Debug, PartialEq, Clone,
)]
#[serde(rename_all = "camelCase")]
pub struct AuditLogConfig {
    /// Specifies the identities that do not cause logging for this type of permission. Follows the
    /// same format of Binding.members.
    #[builder(setter(into))]
    #[serde(default)]
    pub exempted_members: ::std::vec::Vec<::std::string::String>,
    /// The log type that this config enables.
    #[builder(setter(into))]
    pub log_type: LogType,
}

#[derive(
    ::serde::Deserialize,
    ::serde::Serialize,
    ::typed_builder::TypedBuilder,
    Debug,
    PartialEq,
    Clone,
    Default,
)]
#[serde(rename_all = "camelCase")]
pub struct StructTypes {
    /// [Optional] The name of this field.
    #[builder(setter(into))]
    #[serde(default)]
    pub name: ::std::option::Option<::std::string::String>,
    /// [Optional] Human-oriented description of the field.
    #[builder(setter(into))]
    #[serde(default)]
    pub description: ::std::option::Option<::std::string::String>,
    /// [Required] The type of this field.
    #[builder(setter(into))]
    #[serde(rename = "type")]
    #[serde(default)]
    pub ty: QueryParameterType,
}

#[derive(
    ::serde::Deserialize,
    ::serde::Serialize,
    ::typed_builder::TypedBuilder,
    Debug,
    PartialEq,
    Clone,
    Default,
)]
#[serde(rename_all = "camelCase")]
pub struct QueryParameterType {
    /// [Optional] The types of the fields of this struct, in order, if this is a struct.
    #[builder(setter(into))]
    #[serde(default)]
    pub struct_types: ::std::vec::Vec<StructTypes>,
    /// [Required] The top level type of this field.
    #[builder(setter(into))]
    #[serde(rename = "type")]
    #[serde(default)]
    pub ty: ::std::string::String,
    /// [Optional] The type of the array's elements, if this is an array.
    #[builder(setter(into))]
    #[serde(default)]
    pub array_type: ::std::option::Option<::std::boxed::Box<QueryParameterType>>,
}

/// Evaluation metrics for regression and explicit feedback type matrix factorization models.
#[derive(
    ::serde::Deserialize,
    ::serde::Serialize,
    ::typed_builder::TypedBuilder,
    Debug,
    PartialEq,
    Clone,
    Default,
)]
#[serde(rename_all = "camelCase")]
pub struct RegressionMetrics {
    /// Mean squared log error.
    #[builder(setter(into))]
    #[serde(default)]
    pub mean_squared_log_error: f64,
    /// Mean absolute error.
    #[builder(setter(into))]
    #[serde(default)]
    pub mean_absolute_error: f64,
    /// Median absolute error.
    #[builder(setter(into))]
    #[serde(default)]
    pub median_absolute_error: f64,
    /// R^2 score. This corresponds to r2_score in ML.EVALUATE.
    #[builder(setter(into))]
    #[serde(default)]
    pub r_squared: f64,
    /// Mean squared error.
    #[builder(setter(into))]
    #[serde(default)]
    pub mean_squared_error: f64,
}

/// User-defined context as a set of key/value pairs, which will be sent as function invocation
/// context together with batched arguments in the requests to the remote service. The total number
/// of bytes of keys and values must be less than 8KB.
pub type UserDefinedContext =
    ::std::collections::HashMap<::std::string::String, ::std::string::String>;

/// Options for a remote user-defined function.
#[derive(
    ::serde::Deserialize,
    ::serde::Serialize,
    ::typed_builder::TypedBuilder,
    Debug,
    PartialEq,
    Clone,
    Default,
)]
#[serde(rename_all = "camelCase")]
pub struct RemoteFunctionOptions {
    /// Endpoint of the user-provided remote service, e.g.
    /// ```https://us-east1-my_gcf_project.cloudfunctions.net/remote_add```
    #[builder(setter(into))]
    #[serde(default)]
    pub endpoint: ::std::string::String,
    /// Fully qualified name of the user-provided connection object which holds the authentication
    /// information to send requests to the remote service. Format:
    /// ```"projects/{projectId}/locations/{locationId}/connections/{connectionId}"```
    #[builder(setter(into))]
    #[serde(default)]
    pub connection: ::std::string::String,
    /// User-defined context as a set of key/value pairs, which will be sent as function invocation
    /// context together with batched arguments in the requests to the remote service. The total
    /// number of bytes of keys and values must be less than 8KB.
    #[builder(setter(into))]
    #[serde(default)]
    pub user_defined_context: UserDefinedContext,
    /// Max number of rows in each batch sent to the remote service. If absent or if 0, BigQuery
    /// dynamically decides the number of rows in a batch.
    #[builder(setter(into))]
    #[serde(with = "with::int64")]
    #[serde(default)]
    pub max_batching_rows: i64,
}

#[derive(
    ::serde::Deserialize,
    ::serde::Serialize,
    ::typed_builder::TypedBuilder,
    Debug,
    PartialEq,
    Clone,
    Default,
)]
#[serde(rename_all = "camelCase")]
pub struct QueryTimelineSample {
    /// Cumulative slot-ms consumed by the query.
    #[builder(setter(into))]
    #[serde(with = "with::int64")]
    #[serde(default)]
    pub total_slot_ms: i64,
    /// Total parallel units of work completed by this query.
    #[builder(setter(into))]
    #[serde(with = "with::int64")]
    #[serde(default)]
    pub completed_units: i64,
    /// Milliseconds elapsed since the start of query execution.
    #[builder(setter(into))]
    #[serde(with = "with::int64")]
    #[serde(default)]
    pub elapsed_ms: i64,
    /// Total units of work remaining for the query. This number can be revised (increased or
    /// decreased) while the query is running.
    #[builder(setter(into))]
    #[serde(with = "with::int64")]
    #[serde(default)]
    pub pending_units: i64,
    /// Units of work that can be scheduled immediately. Providing additional slots for these units
    /// of work will speed up the query, provided no other query in the reservation needs
    /// additional slots.
    #[builder(setter(into))]
    #[serde(with = "with::int64")]
    #[serde(default)]
    pub estimated_runnable_units: i64,
    /// Total number of units currently being processed by workers. This does not correspond
    /// directly to slot usage. This is the largest value observed since the last sample.
    #[builder(setter(into))]
    #[serde(with = "with::int64")]
    #[serde(default)]
    pub active_units: i64,
}

/// Search space for string and enum.
#[derive(
    ::serde::Deserialize,
    ::serde::Serialize,
    ::typed_builder::TypedBuilder,
    Debug,
    PartialEq,
    Clone,
    Default,
)]
#[serde(rename_all = "camelCase")]
pub struct StringHparamSearchSpace {
    /// Canididates for the string or enum parameter in lower case.
    #[builder(setter(into))]
    #[serde(default)]
    pub candidates: ::std::vec::Vec<::std::string::String>,
}

#[derive(
    ::serde::Deserialize,
    ::serde::Serialize,
    ::typed_builder::TypedBuilder,
    Debug,
    PartialEq,
    Clone,
    Default,
)]
#[serde(rename_all = "camelCase")]
pub struct CsvOptions {
    /// [Optional] Indicates if BigQuery should accept rows that are missing trailing optional
    /// columns. If true, BigQuery treats missing trailing columns as null values. If false,
    /// records with missing trailing columns are treated as bad records, and if there are too
    /// many bad records, an invalid error is returned in the job result. The default value is
    /// false.
    #[builder(setter(into))]
    #[serde(default)]
    pub allow_jagged_rows: ::std::option::Option<bool>,
    /// [Optional] The separator for fields in a CSV file. BigQuery converts the string to
    /// ISO-8859-1 encoding, and then uses the first byte of the encoded string to split the
    /// data in its raw, binary state. BigQuery also supports the escape sequence "\t" to
    /// specify a tab separator. The default value is a comma (',').
    #[builder(setter(into))]
    #[serde(default)]
    pub field_delimiter: ::std::option::Option<::std::string::String>,
    /// [Optional] Indicates if BigQuery should allow quoted data sections that contain newline
    /// characters in a CSV file. The default value is false.
    #[builder(setter(into))]
    #[serde(default)]
    pub allow_quoted_newlines: ::std::option::Option<bool>,
    /// [Optional] An custom string that will represent a NULL value in CSV import data.
    #[builder(setter(into))]
    #[serde(default)]
    pub null_marker: ::std::option::Option<::std::string::String>,
    /// [Optional] The number of rows at the top of a CSV file that BigQuery will skip when reading
    /// the data. The default value is 0. This property is useful if you have header rows in
    /// the file that should be skipped. When autodetect is on, the behavior is the following:
    /// * skipLeadingRows unspecified - Autodetect tries to detect headers in the first row. If
    /// they are not detected, the row is read as data. Otherwise data is read starting from
    /// the second row. * skipLeadingRows is 0
    /// - Instructs autodetect that there are no headers and data should be read starting from the
    ///   first
    /// row. * skipLeadingRows = N > 0 - Autodetect skips N-1 rows and tries to detect headers in
    /// row N. If headers are not detected, row N is just skipped. Otherwise row N is used to
    /// extract column names for the detected schema.
    #[builder(setter(into))]
    #[serde(with = "with::int64::option")]
    #[serde(default)]
    pub skip_leading_rows: ::std::option::Option<i64>,
    /// [Optional] The character encoding of the data. The supported values are UTF-8 or
    /// ISO-8859-1. The default value is UTF-8. BigQuery decodes the data after the raw, binary
    /// data has been split using the values of the quote and fieldDelimiter properties.
    #[builder(setter(into))]
    #[serde(default)]
    pub encoding: ::std::option::Option<::std::string::String>,
    /// [Optional] Preserves the embedded ASCII control characters (the first 32 characters in the
    /// ASCII-table, from '\x00' to '\x1F') when loading from CSV. Only applicable to CSV, ignored
    /// for other formats.
    #[builder(setter(into))]
    #[serde(default)]
    pub preserve_ascii_control_characters: ::std::option::Option<bool>,
    /// [Optional] The value that is used to quote data sections in a CSV file. BigQuery converts
    /// the string to ISO-8859-1 encoding, and then uses the first byte of the encoded string
    /// to split the data in its raw, binary state. The default value is a double-quote ('"').
    /// If your data does not contain quoted sections, set the property value to an empty
    /// string. If your data contains quoted newline characters, you must also set the
    /// allowQuotedNewlines property to true.
    #[builder(setter(into))]
    #[serde(default)]
    pub quote: ::std::option::Option<::std::string::String>,
}

#[derive(
    ::serde::Deserialize,
    ::serde::Serialize,
    ::typed_builder::TypedBuilder,
    Debug,
    PartialEq,
    Clone,
    Default,
)]
#[serde(rename_all = "camelCase")]
pub struct BiEngineReason {
    /// [Output-only] High-level BI Engine reason for partial or disabled acceleration.
    #[builder(setter(into))]
    #[serde(default)]
    pub code: ::std::option::Option<::std::string::String>,
    /// [Output-only] Free form human-readable reason for partial or disabled acceleration.
    #[builder(setter(into))]
    #[serde(default)]
    pub message: ::std::option::Option<::std::string::String>,
}

/// A table type
#[derive(
    ::serde::Deserialize,
    ::serde::Serialize,
    ::typed_builder::TypedBuilder,
    Debug,
    PartialEq,
    Clone,
    Default,
)]
#[serde(rename_all = "camelCase")]
pub struct StandardSqlTableType {
    /// The columns in this table type
    #[builder(setter(into))]
    #[serde(default)]
    pub columns: ::std::vec::Vec<StandardSqlField>,
}

#[derive(
    ::serde::Deserialize,
    ::serde::Serialize,
    ::typed_builder::TypedBuilder,
    Debug,
    PartialEq,
    Clone,
    Default,
)]
#[serde(rename_all = "camelCase")]
pub struct ExternalDataConfiguration {
    /// [Optional] The maximum number of bad records that BigQuery can ignore when reading data. If
    /// the number of bad records exceeds this value, an invalid error is returned in the job
    /// result. This is only valid for CSV, JSON, and Google Sheets. The default value is 0,
    /// which requires that all records are valid. This setting is ignored for Google Cloud
    /// Bigtable, Google Cloud Datastore backups and Avro formats.
    #[builder(setter(into))]
    #[serde(default)]
    pub max_bad_records: ::std::option::Option<i64>,
    /// [Required] The data format. For CSV files, specify "CSV". For Google sheets, specify
    /// "GOOGLE_SHEETS". For newline-delimited JSON, specify "NEWLINE_DELIMITED_JSON". For Avro
    /// files, specify "AVRO". For Google Cloud Datastore backups, specify "DATASTORE_BACKUP".
    /// [Beta] For Google Cloud Bigtable, specify "BIGTABLE".
    #[builder(setter(into))]
    #[serde(default)]
    pub source_format: ::std::string::String,
    /// [Optional] Indicates if BigQuery should allow extra values that are not represented in the
    /// table schema. If true, the extra values are ignored. If false, records with extra
    /// columns are treated as bad records, and if there are too many bad records, an invalid
    /// error is returned in the job result. The default value is false. The sourceFormat
    /// property determines what BigQuery treats as an extra value: CSV: Trailing columns JSON:
    /// Named values that don't match any column names Google Cloud Bigtable: This setting is
    /// ignored. Google Cloud Datastore backups: This setting is ignored. Avro: This setting is
    /// ignored.
    #[builder(setter(into))]
    #[serde(default)]
    pub ignore_unknown_values: ::std::option::Option<bool>,
    /// [Optional] Provide a referencing file with the expected table schema. Enabled for the
    /// format: AVRO, PARQUET, ORC.
    #[builder(setter(into))]
    #[serde(default)]
    pub reference_file_schema_uri: ::std::option::Option<::std::string::String>,
    /// Additional properties to set if sourceFormat is set to CSV.
    #[builder(setter(into))]
    #[serde(default)]
    pub csv_options: CsvOptions,
    /// [Required] The fully-qualified URIs that point to your data in Google Cloud. For Google
    /// Cloud Storage URIs: Each URI can contain one '*' wildcard character and it must come
    /// after the 'bucket' name. Size limits related to load jobs apply to external data
    /// sources. For Google Cloud Bigtable URIs: Exactly one URI can be specified and it has be
    /// a fully specified and valid HTTPS URL for a Google Cloud Bigtable table. For Google
    /// Cloud Datastore backups, exactly one URI can be specified. Also, the '*' wildcard
    /// character is not allowed.
    #[builder(setter(into))]
    #[serde(default)]
    pub source_uris: ::std::vec::Vec<::std::string::String>,
    /// Additional properties to set if sourceFormat is set to Avro.
    #[builder(setter(into))]
    #[serde(default)]
    pub avro_options: AvroOptions,
    /// [Optional] Defines the list of possible SQL data types to which the source decimal values
    /// are converted. This list and the precision and the scale parameters of the decimal
    /// field determine the target type. In the order of NUMERIC, BIGNUMERIC, and STRING, a
    /// type is picked if it is in the specified list and if it supports the precision and the
    /// scale. STRING supports all precision and scale values. If none of the listed types
    /// supports the precision and the scale, the type supporting the widest range in the
    /// specified list is picked, and if a value exceeds the supported range when reading the
    /// data, an error will be thrown. Example: Suppose the value of this field is ["NUMERIC",
    /// "BIGNUMERIC"]. If (precision,scale) is: (38,9) -> NUMERIC; (39,9) -> BIGNUMERIC
    /// (NUMERIC cannot hold 30 integer digits); (38,10) -> BIGNUMERIC (NUMERIC cannot hold
    /// 10 fractional digits); (76,38) -> BIGNUMERIC; (77,38) -> BIGNUMERIC (error if value exeeds
    /// supported range). This field cannot contain duplicate types. The order of the types in this
    /// field is ignored. For example, ["BIGNUMERIC", "NUMERIC"] is the same as ["NUMERIC",
    /// "BIGNUMERIC"] and NUMERIC always takes precedence over BIGNUMERIC. Defaults to ["NUMERIC",
    /// "STRING"] for ORC and ["NUMERIC"] for the other file formats.
    #[builder(setter(into))]
    #[serde(default)]
    pub decimal_target_types: ::std::vec::Vec<::std::string::String>,
    /// Try to detect schema and format options automatically. Any option specified explicitly will
    /// be honored.
    #[builder(setter(into))]
    #[serde(default)]
    pub autodetect: bool,
    /// [Optional] The compression type of the data source. Possible values include GZIP and NONE.
    /// The default value is NONE. This setting is ignored for Google Cloud Bigtable, Google
    /// Cloud Datastore backups and Avro formats.
    #[builder(setter(into))]
    #[serde(default)]
    pub compression: ::std::option::Option<::std::string::String>,
    /// Additional properties to set if sourceFormat is set to Parquet.
    #[builder(setter(into))]
    #[serde(default)]
    pub parquet_options: ParquetOptions,
    /// [Optional, Trusted Tester] Connection for external data source.
    #[builder(setter(into))]
    #[serde(default)]
    pub connection_id: ::std::option::Option<::std::string::String>,
    /// [Optional] The schema for the data. Schema is required for CSV and JSON formats. Schema is
    /// disallowed for Google Cloud Bigtable, Cloud Datastore backups, and Avro formats.
    #[builder(setter(into))]
    #[serde(default)]
    pub schema: ::std::option::Option<TableSchema>,
    /// [Optional] Additional options if sourceFormat is set to BIGTABLE.
    #[builder(setter(into))]
    #[serde(default)]
    pub bigtable_options: BigtableOptions,
    /// [Optional] Options to configure hive partitioning support.
    #[builder(setter(into))]
    #[serde(default)]
    pub hive_partitioning_options: ::std::option::Option<HivePartitioningOptions>,
    /// [Optional] Additional options if sourceFormat is set to GOOGLE_SHEETS.
    #[builder(setter(into))]
    #[serde(default)]
    pub google_sheets_options: ::std::option::Option<GoogleSheetsOptions>,
}

/// Associates `members`, or principals, with a `role`.
#[derive(
    ::serde::Deserialize,
    ::serde::Serialize,
    ::typed_builder::TypedBuilder,
    Debug,
    PartialEq,
    Clone,
    Default,
)]
#[serde(rename_all = "camelCase")]
pub struct Binding {
    /// The condition that is associated with this binding. If the condition evaluates to `true`,
    /// then this binding applies to the current request. If the condition evaluates to
    /// `false`, then this binding does not apply to the current request. However, a different
    /// role binding might grant the same role to one or more of the principals in this
    /// binding. To learn which resources support conditions in their IAM policies, see the
    /// [IAM documentation](https://cloud.google.com/iam/help/conditions/resource-policies).
    #[builder(setter(into))]
    #[serde(default)]
    pub condition: Expr,
    /// Role that is assigned to the list of `members`, or principals. For example, `roles/viewer`,
    /// `roles/editor`, or `roles/owner`.
    #[builder(setter(into))]
    #[serde(default)]
    pub role: ::std::string::String,
    /// Specifies the principals requesting access for a Google Cloud resource. `members` can have
    /// the following values: * `allUsers`: A special identifier that represents anyone who is
    /// on the internet; with or without a Google account. * `allAuthenticatedUsers`: A special
    /// identifier that represents anyone who is authenticated with a Google account or a
    /// service account. Does not include identities that come from external identity providers
    /// (IdPs) through identity federation. * `user:{emailid}`: An email address that
    /// represents a specific Google account. For example, `alice@example.com` . *
    /// `serviceAccount:{emailid}`: An email address that represents a Google service account.
    /// For example, `my-other-app@appspot.gserviceaccount.com`. * `serviceAccount:{projectid}.
    /// svc.id.goog[{namespace}/{kubernetes-sa}]`: An identifier for a [Kubernetes service
    /// account](https://cloud.google.com/kubernetes-engine/docs/how-to/kubernetes-service-accounts).
    /// For example, `my-project.svc.id.goog[my-namespace/my-kubernetes-sa]`. * `group:{emailid}`:
    /// An email address that represents a Google group. For example, `admins@example.com`. *
    /// `deleted:user:{emailid}?uid={uniqueid}`: An email address (plus unique identifier)
    /// representing a user that has been recently deleted. For example,
    /// `alice@example.com?uid=123456789012345678901`. If the user is recovered, this value reverts
    /// to `user:{emailid}` and the recovered user retains the role in the binding. *
    /// `deleted:serviceAccount:{emailid}?uid={uniqueid}`: An email address (plus unique
    /// identifier) representing a service account that has been recently deleted. For example,
    /// `my-other-app@appspot.gserviceaccount.com?uid=123456789012345678901`. If the service
    /// account is undeleted, this value reverts to `serviceAccount:{emailid}` and the
    /// undeleted service account retains the role in the binding. *
    /// `deleted:group:{emailid}?uid={uniqueid}`: An email address (plus unique identifier)
    /// representing a Google group that has been recently deleted. For example, `admins@
    /// example.com?uid=123456789012345678901`. If the group is recovered, this value
    /// reverts to `group:{emailid}` and the recovered group retains the role in the binding. *
    /// `domain:{domain}`: The G Suite domain (primary) that represents all the users of that
    /// domain. For example, `google.com` or `example.com`.
    #[builder(setter(into))]
    #[serde(default)]
    pub members: ::std::vec::Vec<::std::string::String>,
}

/// Response message for `TestIamPermissions` method.
#[derive(
    ::serde::Deserialize,
    ::serde::Serialize,
    ::typed_builder::TypedBuilder,
    Debug,
    PartialEq,
    Clone,
    Default,
)]
#[serde(rename_all = "camelCase")]
pub struct TestIamPermissionsResponse {
    /// A subset of `TestPermissionsRequest.permissions` that the caller is allowed.
    #[builder(setter(into))]
    #[serde(default)]
    pub permissions: ::std::vec::Vec<::std::string::String>,
}

/// Encapsulates settings provided to GetIamPolicy.
#[derive(
    ::serde::Deserialize,
    ::serde::Serialize,
    ::typed_builder::TypedBuilder,
    Debug,
    PartialEq,
    Clone,
    Default,
)]
#[serde(rename_all = "camelCase")]
pub struct GetPolicyOptions {
    /// Optional. The maximum policy version that will be used to format the policy. Valid values
    /// are 0, 1, and 3. Requests specifying an invalid value will be rejected. Requests for
    /// policies with any conditional role bindings must specify version 3. Policies with no
    /// conditional role bindings may specify any valid value or leave the field unset. The
    /// policy in the response might use the policy version that you specified, or it might use
    /// a lower policy version. For example, if you specify version 3, but the policy has no
    /// conditional role bindings, the response uses version 1. To learn which resources
    /// support conditions in their IAM policies, see the [IAM documentation](https://cloud.google.com/iam/help/conditions/resource-policies).
    #[builder(setter(into))]
    #[serde(default)]
    pub requested_policy_version: ::std::option::Option<i64>,
}

/// Representative value of a single feature within the cluster.
#[derive(
    ::serde::Deserialize,
    ::serde::Serialize,
    ::typed_builder::TypedBuilder,
    Debug,
    PartialEq,
    Clone,
    Default,
)]
#[serde(rename_all = "camelCase")]
pub struct FeatureValue {
    /// The feature column name.
    #[builder(setter(into))]
    #[serde(default)]
    pub feature_column: ::std::string::String,
    /// The categorical feature value.
    #[builder(setter(into))]
    #[serde(default)]
    pub categorical_value: CategoricalValue,
    /// The numerical feature value. This is the centroid value for this feature.
    #[builder(setter(into))]
    #[serde(default)]
    pub numerical_value: f64,
}

/// Hyperparameter search spaces. These should be a subset of training_options.
#[derive(
    ::serde::Deserialize, ::serde::Serialize, ::typed_builder::TypedBuilder, Debug, PartialEq, Clone,
)]
#[serde(rename_all = "camelCase")]
pub struct HparamSearchSpaces {
    /// Dart normalization type for boosted tree models.
    #[builder(setter(into))]
    #[serde(default)]
    pub dart_normalize_type: StringHparamSearchSpace,
    /// Number of clusters for k-means.
    #[builder(setter(into))]
    #[serde(default)]
    pub num_clusters: IntHparamSearchSpace,
    /// Subsample ratio of columns when constructing each tree for boosted tree models.
    #[builder(setter(into))]
    pub colsample_bytree: DoubleHparamSearchSpace,
    /// Tree construction algorithm for boosted tree models.
    #[builder(setter(into))]
    #[serde(default)]
    pub tree_method: StringHparamSearchSpace,
    /// Minimum sum of instance weight needed in a child for boosted tree models.
    #[builder(setter(into))]
    #[serde(default)]
    pub min_tree_child_weight: IntHparamSearchSpace,
    /// Learning rate of training jobs.
    #[builder(setter(into))]
    pub learn_rate: DoubleHparamSearchSpace,
    /// Subsample the training data to grow tree to prevent overfitting for boosted tree models.
    #[builder(setter(into))]
    pub subsample: DoubleHparamSearchSpace,
    /// L1 regularization coefficient.
    #[builder(setter(into))]
    pub l_1_reg: DoubleHparamSearchSpace,
    /// Dropout probability for dnn model training and boosted tree models using dart booster.
    #[builder(setter(into))]
    pub dropout: DoubleHparamSearchSpace,
    /// Activation functions of neural network models.
    #[builder(setter(into))]
    #[serde(default)]
    pub activation_fn: StringHparamSearchSpace,
    /// Subsample ratio of columns for each level for boosted tree models.
    #[builder(setter(into))]
    pub colsample_bylevel: DoubleHparamSearchSpace,
    /// Optimizer of TF models.
    #[builder(setter(into))]
    #[serde(default)]
    pub optimizer: StringHparamSearchSpace,
    /// L2 regularization coefficient.
    #[builder(setter(into))]
    pub l_2_reg: DoubleHparamSearchSpace,
    /// Number of parallel trees for boosted tree models.
    #[builder(setter(into))]
    #[serde(default)]
    pub num_parallel_tree: IntHparamSearchSpace,
    /// Maximum depth of a tree for boosted tree models.
    #[builder(setter(into))]
    #[serde(default)]
    pub max_tree_depth: IntHparamSearchSpace,
    /// Subsample ratio of columns for each node(split) for boosted tree models.
    #[builder(setter(into))]
    pub colsample_bynode: DoubleHparamSearchSpace,
    /// Hidden units for neural network models.
    #[builder(setter(into))]
    #[serde(default)]
    pub hidden_units: IntArrayHparamSearchSpace,
    /// Number of latent factors to train on.
    #[builder(setter(into))]
    #[serde(default)]
    pub num_factors: IntHparamSearchSpace,
    /// Mini batch sample size.
    #[builder(setter(into))]
    #[serde(default)]
    pub batch_size: IntHparamSearchSpace,
    /// Minimum split loss for boosted tree models.
    #[builder(setter(into))]
    pub min_split_loss: DoubleHparamSearchSpace,
    /// Booster type for boosted tree models.
    #[builder(setter(into))]
    #[serde(default)]
    pub booster_type: StringHparamSearchSpace,
    /// Hyperparameter for matrix factoration when implicit feedback type is specified.
    #[builder(setter(into))]
    pub wals_alpha: DoubleHparamSearchSpace,
}

#[derive(
    ::serde::Deserialize,
    ::serde::Serialize,
    ::typed_builder::TypedBuilder,
    Debug,
    PartialEq,
    Clone,
    Default,
)]
#[serde(rename_all = "camelCase")]
pub struct HivePartitioningOptions {
    /// [Optional] If set to true, queries over this table require a partition filter that can be
    /// used for partition elimination to be specified. Note that this field should only be
    /// true when creating a permanent external table or querying a temporary external table.
    /// Hive-partitioned loads with requirePartitionFilter explicitly set to true will fail.
    #[builder(setter(into))]
    #[serde(default)]
    pub require_partition_filter: ::std::option::Option<bool>,
    /// [Optional] When hive partition detection is requested, a common prefix for all source uris
    /// should be supplied. The prefix must end immediately before the partition key encoding
    /// begins. For example, consider files following this data layout.
    /// gs://bucket/path_to_table/dt=2019-01-01/country=BR/id=7/file.avro
    /// gs://bucket/path_to_table/dt=2018-12-31/country=CA/id=3/file.avro When hive partitioning is
    /// requested with either AUTO or STRINGS detection, the common prefix can be either of
    /// gs://bucket/path_to_table or gs://bucket/path_to_table/ (trailing slash does not matter).
    #[builder(setter(into))]
    #[serde(default)]
    pub source_uri_prefix: ::std::option::Option<::std::string::String>,
    /// [Optional] When set, what mode of hive partitioning to use when reading data. The following
    /// modes are supported. (1) AUTO: automatically infer partition key name(s) and type(s). (2)
    /// STRINGS: automatically infer partition key name(s). All types are interpreted as strings.
    /// (3) CUSTOM: partition key schema is encoded in the source URI prefix. Not all storage
    /// formats support hive partitioning. Requesting hive partitioning on an unsupported
    /// format will lead to an error. Currently supported types include: AVRO, CSV, JSON, ORC
    /// and Parquet.
    #[builder(setter(into))]
    #[serde(default)]
    pub mode: ::std::option::Option<::std::string::String>,
}

#[derive(::serde::Deserialize, ::serde::Serialize, Debug, PartialEq, Clone)]
pub enum SeasonalPeriods {
    /// No seasonality
    #[serde(rename = "NO_SEASONALITY")]
    NoSeasonality,
    /// Daily period, 24 hours.
    #[serde(rename = "DAILY")]
    Daily,
    /// Weekly period, 7 days.
    #[serde(rename = "WEEKLY")]
    Weekly,
    /// Monthly period, 30 days or irregular.
    #[serde(rename = "MONTHLY")]
    Monthly,
    /// Quarterly period, 90 days or irregular.
    #[serde(rename = "QUARTERLY")]
    Quarterly,
    /// Yearly period, 365 days or irregular.
    #[serde(rename = "YEARLY")]
    Yearly,
}

/// Arima model information.
#[derive(
    ::serde::Deserialize,
    ::serde::Serialize,
    ::typed_builder::TypedBuilder,
    Debug,
    PartialEq,
    Clone,
    Default,
)]
#[serde(rename_all = "camelCase")]
pub struct ArimaModelInfo {
    /// If true, step_changes is a part of time series decomposition result.
    #[builder(setter(into))]
    #[serde(default)]
    pub has_step_changes: bool,
    /// Seasonal periods. Repeated because multiple periods are supported for one time series.
    #[builder(setter(into))]
    #[serde(default)]
    pub seasonal_periods: ::std::vec::Vec<SeasonalPeriods>,
    /// Whether Arima model fitted with drift or not. It is always false when d is not 1.
    #[builder(setter(into))]
    #[serde(default)]
    pub has_drift: bool,
    /// Arima fitting metrics.
    #[builder(setter(into))]
    #[serde(default)]
    pub arima_fitting_metrics: ArimaFittingMetrics,
    /// The tuple of time_series_ids identifying this time series. It will be one of the unique
    /// tuples of values present in the time_series_id_columns specified during ARIMA model
    /// training. Only present when time_series_id_columns training option was used and the
    /// order of values here are same as the order of time_series_id_columns.
    #[builder(setter(into))]
    #[serde(default)]
    pub time_series_ids: ::std::vec::Vec<::std::string::String>,
    /// The time_series_id value for this time series. It will be one of the unique values from the
    /// time_series_id_column specified during ARIMA model training. Only present when
    /// time_series_id_column training option was used.
    #[builder(setter(into))]
    #[serde(default)]
    pub time_series_id: ::std::string::String,
    /// If true, holiday_effect is a part of time series decomposition result.
    #[builder(setter(into))]
    #[serde(default)]
    pub has_holiday_effect: bool,
    /// Non-seasonal order.
    #[builder(setter(into))]
    #[serde(default)]
    pub non_seasonal_order: ArimaOrder,
    /// If true, spikes_and_dips is a part of time series decomposition result.
    #[builder(setter(into))]
    #[serde(default)]
    pub has_spikes_and_dips: bool,
    /// Arima coefficients.
    #[builder(setter(into))]
    #[serde(default)]
    pub arima_coefficients: ArimaCoefficients,
}

#[derive(
    ::serde::Deserialize,
    ::serde::Serialize,
    ::typed_builder::TypedBuilder,
    Debug,
    PartialEq,
    Clone,
    Default,
)]
#[serde(rename_all = "camelCase")]
pub struct TableCell {
    #[builder(setter(into))]
    #[serde(default)]
    pub v: ::serde_json::Value,
}

/// Represents a single JSON object.
pub type JsonObject = ::std::collections::HashMap<::std::string::String, ::serde_json::Value>;

#[derive(
    ::serde::Deserialize,
    ::serde::Serialize,
    ::typed_builder::TypedBuilder,
    Debug,
    PartialEq,
    Clone,
    Default,
)]
#[serde(rename_all = "camelCase")]
pub struct ProjectReference {
    /// [Required] ID of the project. Can be either the numeric ID or the assigned ID of the
    /// project.
    #[builder(setter(into))]
    #[serde(default)]
    pub project_id: ::std::string::String,
}

#[derive(
    ::serde::Deserialize, ::serde::Serialize, ::typed_builder::TypedBuilder, Debug, PartialEq, Clone,
)]
#[serde(rename_all = "camelCase")]
pub struct SnapshotDefinition {
    /// [Required] The time at which the base table was snapshot. This value is reported in the
    /// JSON response using RFC3339 format.
    #[builder(setter(into))]
    pub snapshot_time: ::timestamp::Timestamp,
    /// [Required] Reference describing the ID of the table that was snapshot.
    #[builder(setter(into))]
    #[serde(default)]
    pub base_table_reference: TableReference,
}

/// Evaluation metrics for multi-class classification/classifier models.
#[derive(
    ::serde::Deserialize,
    ::serde::Serialize,
    ::typed_builder::TypedBuilder,
    Debug,
    PartialEq,
    Clone,
    Default,
)]
#[serde(rename_all = "camelCase")]
pub struct MultiClassClassificationMetrics {
    /// Confusion matrix at different thresholds.
    #[builder(setter(into))]
    #[serde(default)]
    pub confusion_matrix_list: ::std::vec::Vec<ConfusionMatrix>,
    /// Aggregate classification metrics.
    #[builder(setter(into))]
    #[serde(default)]
    pub aggregate_classification_metrics: AggregateClassificationMetrics,
}

/// Message containing the information about one cluster.
#[derive(
    ::serde::Deserialize,
    ::serde::Serialize,
    ::typed_builder::TypedBuilder,
    Debug,
    PartialEq,
    Clone,
    Default,
)]
#[serde(rename_all = "camelCase")]
pub struct Cluster {
    /// Values of highly variant features for this cluster.
    #[builder(setter(into))]
    #[serde(default)]
    pub feature_values: ::std::vec::Vec<FeatureValue>,
    /// Centroid id.
    #[builder(setter(into))]
    #[serde(with = "with::int64")]
    #[serde(default)]
    pub centroid_id: i64,
    /// Count of training data rows that were assigned to this cluster.
    #[builder(setter(into))]
    #[serde(with = "with::int64")]
    #[serde(default)]
    pub count: i64,
}

#[derive(
    ::serde::Deserialize, ::serde::Serialize, ::typed_builder::TypedBuilder, Debug, PartialEq, Clone,
)]
#[serde(rename_all = "camelCase")]
pub struct Table {
    /// [Output-only] Number of physical bytes less than 90 days old. This data is not kept in real
    /// time, and might be delayed by a few seconds to a few minutes.
    #[builder(setter(into))]
    #[serde(with = "with::int64::option")]
    #[serde(default)]
    pub num_active_physical_bytes: ::std::option::Option<i64>,
    /// [Output-only] The geographic location where the table resides. This value is inherited from
    /// the dataset.
    #[builder(setter(into))]
    #[serde(default)]
    pub location: ::std::option::Option<::std::string::String>,
    /// [Optional] Materialized view definition.
    #[builder(setter(into))]
    #[serde(default)]
    pub materialized_view: ::std::option::Option<MaterializedViewDefinition>,
    /// [Output-only] The default collation of the table.
    #[builder(setter(into))]
    #[serde(default)]
    pub default_collation: ::std::option::Option<::std::string::String>,
    /// [Output-only] Total number of logical bytes in the table or materialized view.
    #[builder(setter(into))]
    #[serde(with = "with::int64::option")]
    #[serde(default)]
    pub num_total_logical_bytes: ::std::option::Option<i64>,
    /// [Output-only] Clone definition.
    #[builder(setter(into))]
    #[serde(default)]
    pub clone_definition: ::std::option::Option<CloneDefinition>,
    /// The labels associated with this table. You can use these to organize and group your tables.
    /// Label keys and values can be no longer than 63 characters, can only contain lowercase
    /// letters, numeric characters, underscores and dashes. International characters are
    /// allowed. Label values are optional. Label keys must start with a letter and each label
    /// in the list must have a different key.
    #[builder(setter(into))]
    #[serde(default)]
    pub labels: Labels,
    /// [Output-only] Number of physical bytes more than 90 days old. This data is not kept in real
    /// time, and might be delayed by a few seconds to a few minutes.
    #[builder(setter(into))]
    #[serde(with = "with::int64::option")]
    #[serde(default)]
    pub num_long_term_physical_bytes: ::std::option::Option<i64>,
    /// [Output-only] The number of partitions present in the table or materialized view. This data
    /// is not kept in real time, and might be delayed by a few seconds to a few minutes.
    #[builder(setter(into))]
    #[serde(with = "with::int64::option")]
    #[serde(default)]
    pub num_partitions: ::std::option::Option<i64>,
    /// [Output-only] [TrustedTester] The physical size of this table in bytes, excluding any data
    /// in the streaming buffer. This includes compression and storage used for time travel.
    #[builder(setter(into))]
    #[serde(with = "with::int64::option")]
    #[serde(default)]
    pub num_physical_bytes: ::std::option::Option<i64>,
    /// [Optional] A user-friendly description of this table.
    #[builder(setter(into))]
    #[serde(default)]
    pub description: ::std::option::Option<::std::string::String>,
    /// [Beta] Clustering specification for the table. Must be specified with partitioning, data in
    /// the table will be first partitioned and subsequently clustered.
    #[builder(setter(into))]
    #[serde(default)]
    pub clustering: Clustering,
    /// [Optional] The view definition.
    #[builder(setter(into))]
    #[serde(default)]
    pub view: ::std::option::Option<ViewDefinition>,
    /// [Output-only] The type of the resource.
    #[builder(setter(into))]
    #[serde(default)]
    pub kind: ::std::option::Option<::std::string::String>,
    /// [Output-only] Snapshot definition.
    #[builder(setter(into))]
    pub snapshot_definition: Option<SnapshotDefinition>,
    /// [Optional] If set to true, queries over this table require a partition filter that can be
    /// used for partition elimination to be specified.
    #[builder(setter(into))]
    #[serde(default)]
    pub require_partition_filter: ::std::option::Option<bool>,
    /// [Output-only] The size of this table in bytes, excluding any data in the streaming buffer.
    #[builder(setter(into))]
    #[serde(with = "with::int64::option")]
    #[serde(default)]
    pub num_bytes: ::std::option::Option<i64>,
    /// [Output-only] An opaque ID uniquely identifying the table.
    #[builder(setter(into))]
    #[serde(default)]
    pub id: ::std::option::Option<::std::string::String>,
    /// [TrustedTester] Range partitioning specification for this table. Only one of
    /// timePartitioning and rangePartitioning should be specified.
    #[builder(setter(into))]
    #[serde(default)]
    pub range_partitioning: RangePartitioning,
    /// [Output-only] A hash of the table metadata. Used to ensure there were no concurrent
    /// modifications to the resource when attempting an update. Not guaranteed to change when the
    /// table contents or the fields numRows, numBytes, numLongTermBytes or lastModifiedTime
    /// change.
    #[builder(setter(into))]
    #[serde(default)]
    pub etag: ::std::option::Option<::std::string::String>,
    /// [Output-only] The physical size of this table in bytes. This also includes storage used for
    /// time travel. This data is not kept in real time, and might be delayed by a few seconds
    /// to a few minutes.
    #[builder(setter(into))]
    #[serde(with = "with::int64::option")]
    #[serde(default)]
    pub num_total_physical_bytes: ::std::option::Option<i64>,
    /// [Output-only] Describes the table type. The following values are supported: TABLE: A normal
    /// BigQuery table. VIEW: A virtual table defined by a SQL query. SNAPSHOT: An immutable,
    /// read-only table that is a copy of another table. [TrustedTester] MATERIALIZED_VIEW: SQL
    /// query whose result is persisted. EXTERNAL: A table that references data stored in an
    /// external storage system, such as Google Cloud Storage. The default value is TABLE.
    #[builder(setter(into))]
    #[serde(rename = "type")]
    #[serde(default)]
    pub ty: ::std::option::Option<::std::string::String>,
    /// [Optional] The time when this table expires, in milliseconds since the epoch. If not
    /// present, the table will persist indefinitely. Expired tables will be deleted and their
    /// storage reclaimed. The defaultTableExpirationMs property of the encapsulating dataset
    /// can be used to set a default expirationTime on newly created tables.
    #[builder(setter(into))]
    #[serde(with = "with::int64::option")]
    #[serde(default)]
    pub expiration_time: ::std::option::Option<i64>,
    /// Time-based partitioning specification for this table. Only one of timePartitioning and
    /// rangePartitioning should be specified.
    #[builder(setter(into))]
    #[serde(default)]
    pub time_partitioning: TimePartitioning,
    /// [Output-only] The number of rows of data in this table, excluding any data in the streaming
    /// buffer.
    #[builder(setter(into))]
    #[serde(with = "with::uint64::option")]
    #[serde(default)]
    pub num_rows: ::std::option::Option<u64>,
    /// [Optional] Describes the data format, location, and other properties of a table stored
    /// outside of BigQuery. By defining these properties, the data source can then be queried
    /// as if it were a standard BigQuery table.
    #[builder(setter(into))]
    pub external_data_configuration: ExternalDataConfiguration,
    /// Custom encryption configuration (e.g., Cloud KMS keys).
    #[builder(setter(into))]
    #[serde(default)]
    pub encryption_configuration: EncryptionConfiguration,
    /// [Output-only] The time when this table was last modified, in milliseconds since the epoch.
    #[builder(setter(into))]
    #[serde(with = "with::uint64::option")]
    #[serde(default)]
    pub last_modified_time: ::std::option::Option<u64>,
    /// [Output-only] The time when this table was created, in milliseconds since the epoch.
    #[builder(setter(into))]
    #[serde(with = "with::int64::option")]
    #[serde(default)]
    pub creation_time: ::std::option::Option<i64>,
    /// [Output-only] Number of physical bytes used by time travel storage (deleted or changed
    /// data). This data is not kept in real time, and might be delayed by a few seconds to a
    /// few minutes.
    #[builder(setter(into))]
    #[serde(with = "with::int64::option")]
    #[serde(default)]
    pub num_time_travel_physical_bytes: ::std::option::Option<i64>,
    /// [Optional] A descriptive name for this table.
    #[builder(setter(into))]
    #[serde(default)]
    pub friendly_name: ::std::option::Option<::std::string::String>,
    /// [Output-only, Beta] Present iff this table represents a ML model. Describes the training
    /// information for the model, and it is required to run 'PREDICT' queries.
    #[builder(setter(into))]
    #[serde(default)]
    pub model: ::std::option::Option<ModelDefinition>,
    /// [Output-only] The number of bytes in the table that are considered "long-term storage".
    #[builder(setter(into))]
    #[serde(with = "with::int64::option")]
    #[serde(default)]
    pub num_long_term_bytes: ::std::option::Option<i64>,
    /// [Output-only] A URL that can be used to access this resource again.
    #[builder(setter(into))]
    #[serde(default)]
    pub self_link: ::std::option::Option<::std::string::String>,
    /// [Optional] Max staleness of data that could be returned when table or materialized view is
    /// queried (formatted as Google SQL Interval type).
    #[builder(setter(into))]
    #[serde(default)]
    pub max_staleness: ::std::option::Option<::bytes::Bytes>,
    /// [Output-only] Contains information regarding this table's streaming buffer, if one is
    /// present. This field will be absent if the table is not being streamed to or if there is
    /// no data in the streaming buffer.
    #[builder(setter(into))]
    #[serde(default)]
    pub streaming_buffer: ::std::option::Option<Streamingbuffer>,
    /// [Output-only] Number of logical bytes that are more than 90 days old.
    #[builder(setter(into))]
    #[serde(with = "with::int64::option")]
    #[serde(default)]
    pub num_long_term_logical_bytes: ::std::option::Option<i64>,
    /// [Output-only] Number of logical bytes that are less than 90 days old.
    #[builder(setter(into))]
    #[serde(with = "with::int64::option")]
    #[serde(default)]
    pub num_active_logical_bytes: ::std::option::Option<i64>,
    /// [Optional] Describes the schema of this table.
    #[builder(setter(into))]
    #[serde(default)]
    pub schema: ::std::option::Option<TableSchema>,
    /// [Required] Reference describing the ID of this table.
    #[builder(setter(into))]
    #[serde(default)]
    pub table_reference: TableReference,
}

#[derive(
    ::serde::Deserialize,
    ::serde::Serialize,
    ::typed_builder::TypedBuilder,
    Debug,
    PartialEq,
    Clone,
    Default,
)]
#[serde(rename_all = "camelCase")]
pub struct ErrorProto {
    /// A short error code that summarizes the error.
    #[builder(setter(into))]
    #[serde(default)]
    pub reason: ::std::string::String,
    /// A human-readable description of the error.
    #[builder(setter(into))]
    #[serde(default)]
    pub message: ::std::string::String,
    /// Debugging information. This property is internal to Google and should not be used.
    #[builder(setter(into))]
    #[serde(default)]
    pub debug_info: ::std::string::String,
    /// Specifies where the error occurred, if present.
    #[builder(setter(into))]
    #[serde(default)]
    pub location: ::std::string::String,
}

/// Evaluation metrics for binary classification/classifier models.
#[derive(
    ::serde::Deserialize,
    ::serde::Serialize,
    ::typed_builder::TypedBuilder,
    Debug,
    PartialEq,
    Clone,
    Default,
)]
#[serde(rename_all = "camelCase")]
pub struct BinaryClassificationMetrics {
    /// Binary confusion matrix at multiple thresholds.
    #[builder(setter(into))]
    #[serde(default)]
    pub binary_confusion_matrix_list: ::std::vec::Vec<BinaryConfusionMatrix>,
    /// Label representing the positive class.
    #[builder(setter(into))]
    #[serde(default)]
    pub positive_label: ::std::string::String,
    /// Aggregate classification metrics.
    #[builder(setter(into))]
    #[serde(default)]
    pub aggregate_classification_metrics: AggregateClassificationMetrics,
    /// Label representing the negative class.
    #[builder(setter(into))]
    #[serde(default)]
    pub negative_label: ::std::string::String,
}

#[derive(
    ::serde::Deserialize,
    ::serde::Serialize,
    ::typed_builder::TypedBuilder,
    Debug,
    PartialEq,
    Clone,
    Default,
)]
#[serde(rename_all = "camelCase")]
pub struct JobStatus {
    /// [Output-only] Final error result of the job. If present, indicates that the job has
    /// completed and was unsuccessful.
    #[builder(setter(into))]
    #[serde(default)]
    pub error_result: ErrorProto,
    /// [Output-only] Running state of the job.
    #[builder(setter(into))]
    #[serde(default)]
    pub state: ::std::option::Option<::std::string::String>,
    /// [Output-only] The first errors encountered during the running of the job. The final message
    /// includes the number of errors that caused the process to stop. Errors here do not
    /// necessarily mean that the job has completed or was unsuccessful.
    #[builder(setter(into))]
    #[serde(default)]
    pub errors: ::std::vec::Vec<ErrorProto>,
}

#[derive(
    ::serde::Deserialize,
    ::serde::Serialize,
    ::typed_builder::TypedBuilder,
    Debug,
    PartialEq,
    Clone,
    Default,
)]
#[serde(rename_all = "camelCase")]
pub struct StandardSqlStructType {
    #[builder(setter(into))]
    #[serde(default)]
    pub fields: ::std::vec::Vec<StandardSqlField>,
}

#[derive(
    ::serde::Deserialize,
    ::serde::Serialize,
    ::typed_builder::TypedBuilder,
    Debug,
    PartialEq,
    Clone,
    Default,
)]
#[serde(rename_all = "camelCase")]
pub struct TransactionInfo {
    /// [Output-only] // [Alpha] Id of the transaction.
    #[builder(setter(into))]
    #[serde(default)]
    pub transaction_id: ::std::option::Option<::std::string::String>,
}

/// Representative value of a categorical feature.
#[derive(
    ::serde::Deserialize,
    ::serde::Serialize,
    ::typed_builder::TypedBuilder,
    Debug,
    PartialEq,
    Clone,
    Default,
)]
#[serde(rename_all = "camelCase")]
pub struct CategoricalValue {
    /// Counts of all categories for the categorical feature. If there are more than ten
    /// categories, we return top ten (by count) and return one more CategoryCount with
    /// category "_OTHER_" and count as aggregate counts of remaining categories.
    #[builder(setter(into))]
    #[serde(default)]
    pub category_counts: ::std::vec::Vec<CategoryCount>,
}

/// Configuration properties as a set of key/value pairs, which will be passed on to the Spark
/// application. For more information, see [Apache
/// Spark](https://spark.apache.org/docs/latest/index.html).
pub type Properties = ::std::collections::HashMap<::std::string::String, ::std::string::String>;

/// Options for a user-defined Spark routine.
#[derive(
    ::serde::Deserialize,
    ::serde::Serialize,
    ::typed_builder::TypedBuilder,
    Debug,
    PartialEq,
    Clone,
    Default,
)]
#[serde(rename_all = "camelCase")]
pub struct SparkOptions {
    /// JARs to include on the driver and executor CLASSPATH. For more information about Apache
    /// Spark, see [Apache Spark](https://spark.apache.org/docs/latest/index.html).
    #[builder(setter(into))]
    #[serde(default)]
    pub jar_uris: ::std::vec::Vec<::std::string::String>,
    /// Files to be placed in the working directory of each executor. For more information about
    /// Apache Spark, see [Apache Spark](https://spark.apache.org/docs/latest/index.html).
    #[builder(setter(into))]
    #[serde(default)]
    pub file_uris: ::std::vec::Vec<::std::string::String>,
    /// Runtime version. If not specified, the default runtime version is used.
    #[builder(setter(into))]
    #[serde(default)]
    pub runtime_version: ::std::string::String,
    /// Configuration properties as a set of key/value pairs, which will be passed on to the Spark
    /// application. For more information, see [Apache
    /// Spark](https://spark.apache.org/docs/latest/index.html).
    #[builder(setter(into))]
    #[serde(default)]
    pub properties: Properties,
    /// Fully qualified name of the user-provided Spark connection object. Format:
    /// ```"projects/{project_id}/locations/{location_id}/connections/{connection_id}"```
    #[builder(setter(into))]
    #[serde(default)]
    pub connection: ::std::string::String,
    /// Custom container image for the runtime environment.
    #[builder(setter(into))]
    #[serde(default)]
    pub container_image: ::std::string::String,
    /// Python files to be placed on the PYTHONPATH for PySpark application. Supported file types:
    /// `.py`, `.egg`, and `.zip`. For more information about Apache Spark, see [Apache
    /// Spark](https://spark.apache.org/docs/latest/index.html).
    #[builder(setter(into))]
    #[serde(default)]
    pub py_file_uris: ::std::vec::Vec<::std::string::String>,
    /// Archive files to be extracted into the working directory of each executor. For more
    /// information about Apache Spark, see [Apache Spark](https://spark.apache.org/docs/latest/index.html).
    #[builder(setter(into))]
    #[serde(default)]
    pub archive_uris: ::std::vec::Vec<::std::string::String>,
    /// The main file URI of the Spark application. Exactly one of the definition_body field and
    /// the main_file_uri field must be set.
    #[builder(setter(into))]
    #[serde(default)]
    pub main_file_uri: ::std::string::String,
}

/// [Optional] If querying an external data source outside of BigQuery, describes the data format,
/// location and other properties of the data source. By defining these properties, the data source
/// can then be queried as if it were a standard BigQuery table.
pub type TableDefinitions =
    ::std::collections::HashMap<::std::string::String, ExternalDataConfiguration>;

#[derive(
    ::serde::Deserialize,
    ::serde::Serialize,
    ::typed_builder::TypedBuilder,
    Debug,
    PartialEq,
    Clone,
    Default,
)]
#[serde(rename_all = "camelCase")]
pub struct JobConfigurationQuery {
    /// Describes user-defined function resources used in the query.
    #[builder(setter(into))]
    #[serde(default)]
    pub user_defined_function_resources: ::std::vec::Vec<UserDefinedFunctionResource>,
    /// Query parameters for standard SQL queries.
    #[builder(setter(into))]
    #[serde(default)]
    pub query_parameters: ::std::vec::Vec<QueryParameter>,
    /// Specifies whether to use BigQuery's legacy SQL dialect for this query. The default value is
    /// true. If set to false, the query will use BigQuery's standard SQL:
    /// https://cloud.google.com/bigquery/sql-reference/ When useLegacySql is set to false, the value of
    /// flattenResults is ignored; query will be run as if flattenResults is false.
    #[builder(setter(into))]
    #[serde(default)]
    pub use_legacy_sql: bool,
    /// [Optional] Whether to look for the result in the query cache. The query cache is a
    /// best-effort cache that will be flushed whenever tables in the query are modified.
    /// Moreover, the query cache is only available when a query does not have a destination
    /// table specified. The default value is true.
    #[builder(setter(into))]
    #[serde(default)]
    pub use_query_cache: ::std::option::Option<bool>,
    /// [Optional] Specifies whether the job is allowed to create new tables. The following values
    /// are supported: CREATE_IF_NEEDED: If the table does not exist, BigQuery creates the
    /// table. CREATE_NEVER: The table must already exist. If it does not, a 'notFound' error
    /// is returned in the job result. The default value is CREATE_IF_NEEDED. Creation,
    /// truncation and append actions occur as one atomic update upon job completion.
    #[builder(setter(into))]
    #[serde(default)]
    pub create_disposition: ::std::option::Option<::std::string::String>,
    /// Standard SQL only. Set to POSITIONAL to use positional (?) query parameters or to NAMED to
    /// use named (@myparam) query parameters in this query.
    #[builder(setter(into))]
    #[serde(default)]
    pub parameter_mode: ::std::string::String,
    /// If true, creates a new session, where session id will be a server generated random id. If
    /// false, runs query with an existing session_id passed in ConnectionProperty, otherwise
    /// runs query in non-session mode.
    #[builder(setter(into))]
    #[serde(default)]
    pub create_session: bool,
    /// [Optional] If true and query uses legacy SQL dialect, allows the query to produce
    /// arbitrarily large result tables at a slight cost in performance. Requires
    /// destinationTable to be set. For standard SQL queries, this flag is ignored and large
    /// results are always allowed. However, you must still set destinationTable when result
    /// size exceeds the allowed maximum response size.
    #[builder(setter(into))]
    #[serde(default)]
    pub allow_large_results: ::std::option::Option<bool>,
    /// [Optional] Specifies the default dataset to use for unqualified table names in the query.
    /// Note that this does not alter behavior of unqualified dataset names.
    #[builder(setter(into))]
    #[serde(default)]
    pub default_dataset: ::std::option::Option<DatasetReference>,
    /// [Deprecated] This property is deprecated.
    #[builder(setter(into))]
    #[serde(default)]
    pub preserve_nulls: bool,
    /// [Optional] Describes the table where the query results should be stored. If not present, a
    /// new table will be created to store the results. This property must be set for large
    /// results that exceed the maximum response size.
    #[builder(setter(into))]
    #[serde(default)]
    pub destination_table: ::std::option::Option<TableReference>,
    /// Connection properties.
    #[builder(setter(into))]
    #[serde(default)]
    pub connection_properties: ::std::vec::Vec<ConnectionProperty>,
    /// Time-based partitioning specification for the destination table. Only one of
    /// timePartitioning and rangePartitioning should be specified.
    #[builder(setter(into))]
    #[serde(default)]
    pub time_partitioning: TimePartitioning,
    /// [Beta] Clustering specification for the destination table. Must be specified with
    /// time-based partitioning, data in the table will be first partitioned and subsequently
    /// clustered.
    #[builder(setter(into))]
    #[serde(default)]
    pub clustering: Clustering,
    /// [Optional] Limits the bytes billed for this job. Queries that will have bytes billed beyond
    /// this limit will fail (without incurring a charge). If unspecified, this will be set to
    /// your project default.
    #[builder(setter(into))]
    #[serde(with = "with::int64::option")]
    #[serde(default)]
    pub maximum_bytes_billed: ::std::option::Option<i64>,
    /// [Optional] If querying an external data source outside of BigQuery, describes the data
    /// format, location and other properties of the data source. By defining these properties,
    /// the data source can then be queried as if it were a standard BigQuery table.
    #[builder(setter(into))]
    #[serde(default)]
    pub table_definitions: TableDefinitions,
    /// Allows the schema of the destination table to be updated as a side effect of the query job.
    /// Schema update options are supported in two cases: when writeDisposition is WRITE_APPEND;
    /// when writeDisposition is WRITE_TRUNCATE and the destination table is a partition of a
    /// table, specified by partition decorators. For normal tables, WRITE_TRUNCATE will always
    /// overwrite the schema. One or more of the following values are specified:
    /// ALLOW_FIELD_ADDITION: allow adding a nullable field to the schema.
    /// ALLOW_FIELD_RELAXATION: allow relaxing a required field in the original schema to
    /// nullable.
    #[builder(setter(into))]
    #[serde(default)]
    pub schema_update_options: ::std::vec::Vec<::std::string::String>,
    /// [Optional] Specifies the action that occurs if the destination table already exists. The
    /// following values are supported: WRITE_TRUNCATE: If the table already exists, BigQuery
    /// overwrites the table data and uses the schema from the query result. WRITE_APPEND: If
    /// the table already exists, BigQuery appends the data to the table. WRITE_EMPTY: If the
    /// table already exists and contains data, a 'duplicate' error is returned in the job
    /// result. The default value is WRITE_EMPTY. Each action is atomic and only occurs if
    /// BigQuery is able to complete the job successfully. Creation, truncation and append
    /// actions occur as one atomic update upon job completion.
    #[builder(setter(into))]
    #[serde(default)]
    pub write_disposition: ::std::option::Option<::std::string::String>,
    /// [Optional] Specifies a priority for the query. Possible values include INTERACTIVE and
    /// BATCH. The default value is INTERACTIVE.
    #[builder(setter(into))]
    #[serde(default)]
    pub priority: ::std::option::Option<::std::string::String>,
    /// Custom encryption configuration (e.g., Cloud KMS keys).
    #[builder(setter(into))]
    #[serde(default)]
    pub destination_encryption_configuration: EncryptionConfiguration,
    /// [TrustedTester] Range partitioning specification for this table. Only one of
    /// timePartitioning and rangePartitioning should be specified.
    #[builder(setter(into))]
    #[serde(default)]
    pub range_partitioning: RangePartitioning,
    /// [Optional] Limits the billing tier for this job. Queries that have resource usage beyond
    /// this tier will fail (without incurring a charge). If unspecified, this will be set to
    /// your project default.
    #[builder(setter(into))]
    #[serde(default)]
    pub maximum_billing_tier: ::std::option::Option<i64>,
    /// [Optional] If true and query uses legacy SQL dialect, flattens all nested and repeated
    /// fields in the query results. allowLargeResults must be true if this is set to false.
    /// For standard SQL queries, this flag is ignored and results are never flattened.
    #[builder(setter(into))]
    #[serde(default)]
    pub flatten_results: ::std::option::Option<bool>,
    /// [Required] SQL query text to execute. The useLegacySql field can be used to indicate
    /// whether the query uses legacy SQL or standard SQL.
    #[builder(setter(into))]
    #[serde(default)]
    pub query: ::std::string::String,
}

#[derive(
    ::serde::Deserialize, ::serde::Serialize, ::typed_builder::TypedBuilder, Debug, PartialEq, Clone,
)]
#[serde(rename_all = "camelCase")]
pub struct DestinationTableProperties {
    /// [Internal] This field is for Google internal use only.
    #[builder(setter(into))]
    pub expiration_time: ::timestamp::Timestamp,
    /// [Optional] The description for the destination table. This will only be used if the
    /// destination table is newly created. If the table already exists and a value different
    /// than the current description is provided, the job will fail.
    #[builder(setter(into))]
    #[serde(default)]
    pub description: ::std::option::Option<::std::string::String>,
    /// [Optional] The friendly name for the destination table. This will only be used if the
    /// destination table is newly created. If the table already exists and a value different than
    /// the current friendly name is provided, the job will fail.
    #[builder(setter(into))]
    #[serde(default)]
    pub friendly_name: ::std::option::Option<::std::string::String>,
    /// [Optional] The labels associated with this table. You can use these to organize and group
    /// your tables. This will only be used if the destination table is newly created. If the
    /// table already exists and labels are different than the current labels are provided, the
    /// job will fail.
    #[builder(setter(into))]
    #[serde(default)]
    pub labels: Labels,
}

#[derive(
    ::serde::Deserialize,
    ::serde::Serialize,
    ::typed_builder::TypedBuilder,
    Debug,
    PartialEq,
    Clone,
    Default,
)]
#[serde(rename_all = "camelCase")]
pub struct JobStatistics4 {
    /// [Output-only] Number of files per destination URI or URI pattern specified in the extract
    /// configuration. These values will be in the same order as the URIs specified in the
    /// 'destinationUris' field.
    #[builder(setter(into))]
    #[serde(default)]
    pub destination_uri_file_counts: ::std::vec::Vec<i64>,
    /// [Output-only] Number of user bytes extracted into the result. This is the byte count as
    /// computed by BigQuery for billing purposes.
    #[builder(setter(into))]
    #[serde(with = "with::int64::option")]
    #[serde(default)]
    pub input_bytes: ::std::option::Option<i64>,
}

#[derive(
    ::serde::Deserialize,
    ::serde::Serialize,
    ::typed_builder::TypedBuilder,
    Debug,
    PartialEq,
    Clone,
    Default,
)]
#[serde(rename_all = "camelCase")]
pub struct ExplainQueryStage {
    /// Relative amount of time the slowest shard spent on writing output.
    #[builder(setter(into))]
    #[serde(default)]
    pub write_ratio_max: f64,
    /// Milliseconds the average shard spent on writing output.
    #[builder(setter(into))]
    #[serde(with = "with::int64")]
    #[serde(default)]
    pub write_ms_avg: i64,
    /// Stage end time represented as milliseconds since epoch.
    #[builder(setter(into))]
    #[serde(with = "with::int64")]
    #[serde(default)]
    pub end_ms: i64,
    /// Milliseconds the average shard spent waiting to be scheduled.
    #[builder(setter(into))]
    #[serde(with = "with::int64")]
    #[serde(default)]
    pub wait_ms_avg: i64,
    /// Unique ID for stage within plan.
    #[builder(setter(into))]
    #[serde(with = "with::int64")]
    #[serde(default)]
    pub id: i64,
    /// List of operations within the stage in dependency order (approximately chronological).
    #[builder(setter(into))]
    #[serde(default)]
    pub steps: ::std::vec::Vec<ExplainQueryStep>,
    /// IDs for stages that are inputs to this stage.
    #[builder(setter(into))]
    #[serde(default)]
    pub input_stages: ::std::vec::Vec<i64>,
    /// Number of records written by the stage.
    #[builder(setter(into))]
    #[serde(with = "with::int64")]
    #[serde(default)]
    pub records_written: i64,
    /// Milliseconds the slowest shard spent reading input.
    #[builder(setter(into))]
    #[serde(with = "with::int64")]
    #[serde(default)]
    pub read_ms_max: i64,
    /// Relative amount of time the slowest shard spent reading input.
    #[builder(setter(into))]
    #[serde(default)]
    pub read_ratio_max: f64,
    /// Milliseconds the average shard spent reading input.
    #[builder(setter(into))]
    #[serde(with = "with::int64")]
    #[serde(default)]
    pub read_ms_avg: i64,
    /// Number of parallel input segments to be processed.
    #[builder(setter(into))]
    #[serde(with = "with::int64")]
    #[serde(default)]
    pub parallel_inputs: i64,
    /// Relative amount of time the slowest shard spent on CPU-bound tasks.
    #[builder(setter(into))]
    #[serde(default)]
    pub compute_ratio_max: f64,
    /// Relative amount of time the average shard spent on CPU-bound tasks.
    #[builder(setter(into))]
    #[serde(default)]
    pub compute_ratio_avg: f64,
    /// Relative amount of time the slowest shard spent waiting to be scheduled.
    #[builder(setter(into))]
    #[serde(default)]
    pub wait_ratio_max: f64,
    /// Total number of bytes written to shuffle and spilled to disk.
    #[builder(setter(into))]
    #[serde(with = "with::int64")]
    #[serde(default)]
    pub shuffle_output_bytes_spilled: i64,
    /// Number of parallel input segments completed.
    #[builder(setter(into))]
    #[serde(with = "with::int64")]
    #[serde(default)]
    pub completed_parallel_inputs: i64,
    /// Milliseconds the average shard spent on CPU-bound tasks.
    #[builder(setter(into))]
    #[serde(with = "with::int64")]
    #[serde(default)]
    pub compute_ms_avg: i64,
    /// Total number of bytes written to shuffle.
    #[builder(setter(into))]
    #[serde(with = "with::int64")]
    #[serde(default)]
    pub shuffle_output_bytes: i64,
    /// Milliseconds the slowest shard spent on writing output.
    #[builder(setter(into))]
    #[serde(with = "with::int64")]
    #[serde(default)]
    pub write_ms_max: i64,
    /// Human-readable name for stage.
    #[builder(setter(into))]
    #[serde(default)]
    pub name: ::std::string::String,
    /// Stage start time represented as milliseconds since epoch.
    #[builder(setter(into))]
    #[serde(with = "with::int64")]
    #[serde(default)]
    pub start_ms: i64,
    /// Relative amount of time the average shard spent reading input.
    #[builder(setter(into))]
    #[serde(default)]
    pub read_ratio_avg: f64,
    /// Milliseconds the slowest shard spent on CPU-bound tasks.
    #[builder(setter(into))]
    #[serde(with = "with::int64")]
    #[serde(default)]
    pub compute_ms_max: i64,
    /// Number of records read into the stage.
    #[builder(setter(into))]
    #[serde(with = "with::int64")]
    #[serde(default)]
    pub records_read: i64,
    /// Current status for the stage.
    #[builder(setter(into))]
    #[serde(default)]
    pub status: ::std::string::String,
    /// Slot-milliseconds used by the stage.
    #[builder(setter(into))]
    #[serde(with = "with::int64")]
    #[serde(default)]
    pub slot_ms: i64,
    /// Milliseconds the slowest shard spent waiting to be scheduled.
    #[builder(setter(into))]
    #[serde(with = "with::int64")]
    #[serde(default)]
    pub wait_ms_max: i64,
    /// Relative amount of time the average shard spent waiting to be scheduled.
    #[builder(setter(into))]
    #[serde(default)]
    pub wait_ratio_avg: f64,
    /// Relative amount of time the average shard spent on writing output.
    #[builder(setter(into))]
    #[serde(default)]
    pub write_ratio_avg: f64,
}

#[derive(
    ::serde::Deserialize,
    ::serde::Serialize,
    ::typed_builder::TypedBuilder,
    Debug,
    PartialEq,
    Clone,
    Default,
)]
#[serde(rename_all = "camelCase")]
pub struct MaterializedViewDefinition {
    /// [Optional] [TrustedTester] The maximum frequency at which this materialized view will be
    /// refreshed. The default value is "1800000" (30 minutes).
    #[builder(setter(into))]
    #[serde(with = "with::int64::option")]
    #[serde(default)]
    pub refresh_interval_ms: ::std::option::Option<i64>,
    /// [Required] A query whose result is persisted.
    #[builder(setter(into))]
    #[serde(default)]
    pub query: ::std::string::String,
    /// [Optional] Max staleness of data that could be returned when materizlized view is queried
    /// (formatted as Google SQL Interval type).
    #[builder(setter(into))]
    #[serde(default)]
    pub max_staleness: ::std::option::Option<::bytes::Bytes>,
    /// [Optional] [TrustedTester] Enable automatic refresh of the materialized view when the base
    /// table is updated. The default value is "true".
    #[builder(setter(into))]
    #[serde(default)]
    pub enable_refresh: ::std::option::Option<bool>,
    /// [Output-only] [TrustedTester] The time when this materialized view was last modified, in
    /// milliseconds since the epoch.
    #[builder(setter(into))]
    #[serde(with = "with::int64::option")]
    #[serde(default)]
    pub last_refresh_time: ::std::option::Option<i64>,
}

#[derive(
    ::serde::Deserialize,
    ::serde::Serialize,
    ::typed_builder::TypedBuilder,
    Debug,
    PartialEq,
    Clone,
    Default,
)]
#[serde(rename_all = "camelCase")]
pub struct ParquetOptions {
    /// [Optional] Indicates whether to infer Parquet ENUM logical type as STRING instead of BYTES
    /// by default.
    #[builder(setter(into))]
    #[serde(default)]
    pub enum_as_string: ::std::option::Option<bool>,
    /// [Optional] Indicates whether to use schema inference specifically for Parquet LIST logical
    /// type.
    #[builder(setter(into))]
    #[serde(default)]
    pub enable_list_inference: ::std::option::Option<bool>,
}

#[derive(
    ::serde::Deserialize, ::serde::Serialize, ::typed_builder::TypedBuilder, Debug, PartialEq, Clone,
)]
#[serde(rename_all = "camelCase")]
pub struct CloneDefinition {
    /// [Required] Reference describing the ID of the table that was cloned.
    #[builder(setter(into))]
    #[serde(default)]
    pub base_table_reference: TableReference,
    /// [Required] The time at which the base table was cloned. This value is reported in the JSON
    /// response using RFC3339 format.
    #[builder(setter(into))]
    pub clone_time: ::timestamp::Timestamp,
}

/// A field or a column.
#[derive(
    ::serde::Deserialize,
    ::serde::Serialize,
    ::typed_builder::TypedBuilder,
    Debug,
    PartialEq,
    Clone,
    Default,
)]
#[serde(rename_all = "camelCase")]
pub struct StandardSqlField {
    /// Optional. The type of this parameter. Absent if not explicitly specified (e.g., CREATE
    /// FUNCTION statement can omit the return type; in this case the output parameter does not
    /// have this "type" field).
    #[builder(setter(into))]
    #[serde(rename = "type")]
    #[serde(default)]
    pub ty: ::std::option::Option<StandardSqlDataType>,
    /// Optional. The name of this field. Can be absent for struct fields.
    #[builder(setter(into))]
    #[serde(default)]
    pub name: ::std::option::Option<::std::string::String>,
}

#[derive(
    ::serde::Deserialize,
    ::serde::Serialize,
    ::typed_builder::TypedBuilder,
    Debug,
    PartialEq,
    Clone,
    Default,
)]
#[serde(rename_all = "camelCase")]
pub struct BigtableColumnFamily {
    /// [Optional] The type to convert the value in cells of this column family. The values are
    /// expected to be encoded using HBase Bytes.toBytes function when using the BINARY
    /// encoding value. Following BigQuery types are allowed (case-sensitive) - BYTES STRING
    /// INTEGER FLOAT BOOLEAN Default type is BYTES. This can be overridden for a specific
    /// column by listing that column in 'columns' and specifying a type for it.
    #[builder(setter(into))]
    #[serde(rename = "type")]
    #[serde(default)]
    pub ty: ::std::option::Option<::std::string::String>,
    /// [Optional] Lists of columns that should be exposed as individual fields as opposed to a
    /// list of (column name, value) pairs. All columns whose qualifier matches a qualifier in
    /// this list can be accessed as .. Other columns can be accessed as a list through .Column
    /// field.
    #[builder(setter(into))]
    #[serde(default)]
    pub columns: ::std::vec::Vec<BigtableColumn>,
    /// Identifier of the column family.
    #[builder(setter(into))]
    #[serde(default)]
    pub family_id: ::std::string::String,
    /// [Optional] The encoding of the values when the type is not STRING. Acceptable encoding
    /// values are: TEXT - indicates values are alphanumeric text strings. BINARY - indicates
    /// values are encoded using HBase Bytes.toBytes family of functions. This can be
    /// overridden for a specific column by listing that column in 'columns' and specifying an
    /// encoding for it.
    #[builder(setter(into))]
    #[serde(default)]
    pub encoding: ::std::option::Option<::std::string::String>,
    /// [Optional] If this is set only the latest version of value are exposed for all columns in
    /// this column family. This can be overridden for a specific column by listing that column
    /// in 'columns' and specifying a different setting for that column.
    #[builder(setter(into))]
    #[serde(default)]
    pub only_read_latest: ::std::option::Option<bool>,
}

/// Explanation for a single feature.
#[derive(
    ::serde::Deserialize,
    ::serde::Serialize,
    ::typed_builder::TypedBuilder,
    Debug,
    PartialEq,
    Clone,
    Default,
)]
#[serde(rename_all = "camelCase")]
pub struct Explanation {
    /// The full feature name. For non-numerical features, will be formatted like `.`. Overall size
    /// of feature name will always be truncated to first 120 characters.
    #[builder(setter(into))]
    #[serde(default)]
    pub feature_name: ::std::string::String,
    /// Attribution of feature.
    #[builder(setter(into))]
    #[serde(default)]
    pub attribution: f64,
}

/// Principal component infos, used only for eigen decomposition based models, e.g., PCA. Ordered by
/// explained_variance in the descending order.
#[derive(
    ::serde::Deserialize,
    ::serde::Serialize,
    ::typed_builder::TypedBuilder,
    Debug,
    PartialEq,
    Clone,
    Default,
)]
#[serde(rename_all = "camelCase")]
pub struct PrincipalComponentInfo {
    /// Explained_variance over the total explained variance.
    #[builder(setter(into))]
    #[serde(default)]
    pub explained_variance_ratio: f64,
    /// The explained_variance is pre-ordered in the descending order to compute the cumulative
    /// explained variance ratio.
    #[builder(setter(into))]
    #[serde(default)]
    pub cumulative_explained_variance_ratio: f64,
    /// Explained variance by this principal component, which is simply the eigenvalue.
    #[builder(setter(into))]
    #[serde(default)]
    pub explained_variance: f64,
    /// Id of the principal component.
    #[builder(setter(into))]
    #[serde(with = "with::int64")]
    #[serde(default)]
    pub principal_component_id: i64,
}

#[derive(
    ::serde::Deserialize,
    ::serde::Serialize,
    ::typed_builder::TypedBuilder,
    Debug,
    PartialEq,
    Clone,
    Default,
)]
#[serde(rename_all = "camelCase")]
pub struct ScriptStatistics {
    /// [Output-only] Whether this child job was a statement or expression.
    #[builder(setter(into))]
    #[serde(default)]
    pub evaluation_kind: ::std::option::Option<::std::string::String>,
    /// Stack trace showing the line/column/procedure name of each frame on the stack at the point
    /// where the current evaluation happened. The leaf frame is first, the primary script is
    /// last. Never empty.
    #[builder(setter(into))]
    #[serde(default)]
    pub stack_frames: ::std::vec::Vec<ScriptStackFrame>,
}

#[derive(
    ::serde::Deserialize,
    ::serde::Serialize,
    ::typed_builder::TypedBuilder,
    Debug,
    PartialEq,
    Clone,
    Default,
)]
#[serde(rename_all = "camelCase")]
pub struct GoogleSheetsOptions {
    /// [Optional] The number of rows at the top of a sheet that BigQuery will skip when reading
    /// the data. The default value is 0. This property is useful if you have header rows that
    /// should be skipped. When autodetect is on, behavior is the following: * skipLeadingRows
    /// unspecified - Autodetect tries to detect headers in the first row. If they are not
    /// detected, the row is read as data. Otherwise data is read starting from the second row.
    /// * skipLeadingRows is 0 - Instructs autodetect that there are no headers and data should
    /// be read starting from the first row. * skipLeadingRows = N > 0 - Autodetect skips N-1
    /// rows and tries to detect headers in row N. If headers are not detected, row N is just
    /// skipped. Otherwise row N is used to extract column names for the detected schema.
    #[builder(setter(into))]
    #[serde(with = "with::int64::option")]
    #[serde(default)]
    pub skip_leading_rows: ::std::option::Option<i64>,
    /// [Optional] Range of a sheet to query from. Only used when non-empty. Typical format:
    /// sheet_name!top_left_cell_id:bottom_right_cell_id For example: sheet1!A1:B20
    #[builder(setter(into))]
    #[serde(default)]
    pub range: ::std::option::Option<::std::string::String>,
}

#[derive(
    ::serde::Deserialize,
    ::serde::Serialize,
    ::typed_builder::TypedBuilder,
    Debug,
    PartialEq,
    Clone,
    Default,
)]
#[serde(rename_all = "camelCase")]
pub struct QueryParameter {
    /// [Required] The type of this parameter.
    #[builder(setter(into))]
    #[serde(default)]
    pub parameter_type: QueryParameterType,
    /// [Optional] If unset, this is a positional parameter. Otherwise, should be unique within a
    /// query.
    #[builder(setter(into))]
    #[serde(default)]
    pub name: ::std::option::Option<::std::string::String>,
    /// [Required] The value of this parameter.
    #[builder(setter(into))]
    #[serde(default)]
    pub parameter_value: QueryParameterValue,
}

#[derive(
    ::serde::Deserialize,
    ::serde::Serialize,
    ::typed_builder::TypedBuilder,
    Debug,
    PartialEq,
    Clone,
    Default,
)]
#[serde(rename_all = "camelCase")]
pub struct JobStatistics3 {
    /// [Output-only] Number of rows imported in a load job. Note that while an import job is in
    /// the running state, this value may change.
    #[builder(setter(into))]
    #[serde(with = "with::int64::option")]
    #[serde(default)]
    pub output_rows: ::std::option::Option<i64>,
    /// [Output-only] Number of bytes of source data in a load job.
    #[builder(setter(into))]
    #[serde(with = "with::int64::option")]
    #[serde(default)]
    pub input_file_bytes: ::std::option::Option<i64>,
    /// [Output-only] Size of the loaded data in bytes. Note that while a load job is in the
    /// running state, this value may change.
    #[builder(setter(into))]
    #[serde(with = "with::int64::option")]
    #[serde(default)]
    pub output_bytes: ::std::option::Option<i64>,
    /// [Output-only] The number of bad records encountered. Note that if the job has failed
    /// because of more bad records encountered than the maximum allowed in the load job
    /// configuration, then this number can be less than the total number of bad records
    /// present in the input data.
    #[builder(setter(into))]
    #[serde(with = "with::int64::option")]
    #[serde(default)]
    pub bad_records: ::std::option::Option<i64>,
    /// [Output-only] Number of source files in a load job.
    #[builder(setter(into))]
    #[serde(with = "with::int64::option")]
    #[serde(default)]
    pub input_files: ::std::option::Option<i64>,
}

/// [Optional] The struct field values, in order of the struct type's declaration.
pub type StructValues = ::std::collections::HashMap<::std::string::String, QueryParameterValue>;

#[derive(
    ::serde::Deserialize,
    ::serde::Serialize,
    ::typed_builder::TypedBuilder,
    Debug,
    PartialEq,
    Clone,
    Default,
)]
#[serde(rename_all = "camelCase")]
pub struct QueryParameterValue {
    /// [Optional] The array values, if this is an array type.
    #[builder(setter(into))]
    #[serde(default)]
    pub array_values: ::std::vec::Vec<QueryParameterValue>,
    /// [Optional] The struct field values, in order of the struct type's declaration.
    #[builder(setter(into))]
    #[serde(default)]
    pub struct_values: StructValues,
    /// [Optional] The value of this value, if a simple scalar type.
    #[builder(setter(into))]
    #[serde(default)]
    pub value: ::std::option::Option<::std::string::String>,
}

#[derive(
    ::serde::Deserialize,
    ::serde::Serialize,
    ::typed_builder::TypedBuilder,
    Debug,
    PartialEq,
    Clone,
    Default,
)]
#[serde(rename_all = "camelCase")]
pub struct ExplainQueryStep {
    /// Human-readable stage descriptions.
    #[builder(setter(into))]
    #[serde(default)]
    pub substeps: ::std::vec::Vec<::std::string::String>,
    /// Machine-readable operation type.
    #[builder(setter(into))]
    #[serde(default)]
    pub kind: ::std::string::String,
}

/// Model evaluation metrics for ARIMA forecasting models.
#[derive(
    ::serde::Deserialize,
    ::serde::Serialize,
    ::typed_builder::TypedBuilder,
    Debug,
    PartialEq,
    Clone,
    Default,
)]
#[serde(rename_all = "camelCase")]
pub struct ArimaForecastingMetrics {
    /// Whether Arima model fitted with drift or not. It is always false when d is not 1.
    #[builder(setter(into))]
    #[serde(default)]
    pub has_drift: ::std::vec::Vec<bool>,
    /// Id to differentiate different time series for the large-scale case.
    #[builder(setter(into))]
    #[serde(default)]
    pub time_series_id: ::std::vec::Vec<::std::string::String>,
    /// Non-seasonal order.
    #[builder(setter(into))]
    #[serde(default)]
    pub non_seasonal_order: ::std::vec::Vec<ArimaOrder>,
    /// Repeated as there can be many metric sets (one for each model) in auto-arima and the
    /// large-scale case.
    #[builder(setter(into))]
    #[serde(default)]
    pub arima_single_model_forecasting_metrics: ::std::vec::Vec<ArimaSingleModelForecastingMetrics>,
    /// Arima model fitting metrics.
    #[builder(setter(into))]
    #[serde(default)]
    pub arima_fitting_metrics: ::std::vec::Vec<ArimaFittingMetrics>,
    /// Seasonal periods. Repeated because multiple periods are supported for one time series.
    #[builder(setter(into))]
    #[serde(default)]
    pub seasonal_periods: ::std::vec::Vec<SeasonalPeriods>,
}

/// Evaluation metrics used by weighted-ALS models specified by feedback_type=implicit.
#[derive(
    ::serde::Deserialize,
    ::serde::Serialize,
    ::typed_builder::TypedBuilder,
    Debug,
    PartialEq,
    Clone,
    Default,
)]
#[serde(rename_all = "camelCase")]
pub struct RankingMetrics {
    /// Calculates a precision per user for all the items by ranking them and then averages all the
    /// precisions across all the users.
    #[builder(setter(into))]
    #[serde(default)]
    pub mean_average_precision: f64,
    /// Determines the goodness of a ranking by computing the percentile rank from the predicted
    /// confidence and dividing it by the original rank.
    #[builder(setter(into))]
    #[serde(default)]
    pub average_rank: f64,
    /// Similar to the mean squared error computed in regression and explicit recommendation models
    /// except instead of computing the rating directly, the output from evaluate is computed
    /// against a preference which is 1 or 0 depending on if the rating exists or not.
    #[builder(setter(into))]
    #[serde(default)]
    pub mean_squared_error: f64,
    /// A metric to determine the goodness of a ranking calculated from the predicted confidence by
    /// comparing it to an ideal rank measured by the original ratings.
    #[builder(setter(into))]
    #[serde(default)]
    pub normalized_discounted_cumulative_gain: f64,
}

#[derive(
    ::serde::Deserialize,
    ::serde::Serialize,
    ::typed_builder::TypedBuilder,
    Debug,
    PartialEq,
    Clone,
    Default,
)]
#[serde(rename_all = "camelCase")]
pub struct QueryResponse {
    /// An object with as many results as can be contained within the maximum permitted reply size.
    /// To get any additional rows, you can call GetQueryResults and specify the jobReference
    /// returned above.
    #[builder(setter(into))]
    #[serde(default)]
    pub rows: ::std::vec::Vec<TableRow>,
    /// The total number of rows in the complete query result set, which can be more than the
    /// number of rows in this single page of results.
    #[builder(setter(into))]
    #[serde(with = "with::uint64")]
    #[serde(default)]
    pub total_rows: u64,
    /// [Output-only] [Preview] Information of the session if this job is part of one.
    #[builder(setter(into))]
    #[serde(default)]
    pub session_info: ::std::option::Option<SessionInfo>,
    /// Reference to the Job that was created to run the query. This field will be present even if
    /// the original request timed out, in which case GetQueryResults can be used to read the
    /// results once the query has completed. Since this API only returns the first page of
    /// results, subsequent pages can be fetched via the same mechanism (GetQueryResults).
    #[builder(setter(into))]
    #[serde(default)]
    pub job_reference: JobReference,
    /// The schema of the results. Present only when the query completes successfully.
    #[builder(setter(into))]
    #[serde(default)]
    pub schema: TableSchema,
    /// Whether the query has completed or not. If rows or totalRows are present, this will always
    /// be true. If this is false, totalRows will not be available.
    #[builder(setter(into))]
    #[serde(default)]
    pub job_complete: bool,
    /// [Output-only] The number of rows affected by a DML statement. Present only for DML
    /// statements INSERT, UPDATE or DELETE.
    #[builder(setter(into))]
    #[serde(with = "with::int64::option")]
    #[serde(default)]
    pub num_dml_affected_rows: ::std::option::Option<i64>,
    /// A token used for paging results.
    #[builder(setter(into))]
    #[serde(default)]
    pub page_token: ::std::string::String,
    /// [Output-only] The first errors or warnings encountered during the running of the job. The
    /// final message includes the number of errors that caused the process to stop. Errors
    /// here do not necessarily mean that the job has completed or was unsuccessful.
    #[builder(setter(into))]
    #[serde(default)]
    pub errors: ::std::vec::Vec<ErrorProto>,
    /// The resource type.
    #[builder(setter(into))]
    #[serde(default)]
    pub kind: ::std::string::String,
    /// [Output-only] Detailed statistics for DML statements Present only for DML statements
    /// INSERT, UPDATE, DELETE or TRUNCATE.
    #[builder(setter(into))]
    #[serde(default)]
    pub dml_stats: ::std::option::Option<DmlStatistics>,
    /// Whether the query result was fetched from the query cache.
    #[builder(setter(into))]
    #[serde(default)]
    pub cache_hit: bool,
    /// The total number of bytes processed for this query. If this query was a dry run, this is
    /// the number of bytes that would be processed if the query were run.
    #[builder(setter(into))]
    #[serde(with = "with::int64")]
    #[serde(default)]
    pub total_bytes_processed: i64,
}

#[derive(
    ::serde::Deserialize, ::serde::Serialize, ::typed_builder::TypedBuilder, Debug, PartialEq, Clone,
)]
#[serde(rename_all = "camelCase")]
pub struct Job {
    /// [Output-only] A hash of this resource.
    #[builder(setter(into))]
    #[serde(default)]
    pub etag: ::std::option::Option<::std::string::String>,
    /// [Output-only] Information about the job, including starting time and ending time of the
    /// job.
    #[builder(setter(into))]
    #[serde(default)]
    pub statistics: ::std::option::Option<JobStatistics>,
    /// [Output-only] Opaque ID field of the job
    #[builder(setter(into))]
    #[serde(default)]
    pub id: ::std::option::Option<::std::string::String>,
    /// [Optional] Reference describing the unique-per-user name of the job.
    #[builder(setter(into))]
    #[serde(default)]
    pub job_reference: JobReference,
    /// [Output-only] The type of the resource.
    #[builder(setter(into))]
    #[serde(default)]
    pub kind: ::std::option::Option<::std::string::String>,
    /// [Output-only] Email address of the user who ran the job.
    #[builder(setter(into))]
    #[serde(default)]
    pub user_email: ::std::option::Option<::std::string::String>,
    /// [Required] Describes the job configuration.
    #[builder(setter(into))]
    pub configuration: JobConfiguration,
    /// [Output-only] The status of this job. Examine this value when polling an asynchronous job
    /// to see if the job is complete.
    #[builder(setter(into))]
    #[serde(default)]
    pub status: JobStatus,
    /// [Output-only] A URL that can be used to access this resource again.
    #[builder(setter(into))]
    #[serde(default)]
    pub self_link: ::std::option::Option<::std::string::String>,
}

/// Additional details for a view.
#[derive(
    ::serde::Deserialize,
    ::serde::Serialize,
    ::typed_builder::TypedBuilder,
    Debug,
    PartialEq,
    Clone,
    Default,
)]
#[serde(rename_all = "camelCase")]
pub struct View {
    /// True if view is defined in legacy SQL dialect, false if in standard SQL.
    #[builder(setter(into))]
    #[serde(default)]
    pub use_legacy_sql: bool,
}

#[derive(
    ::serde::Deserialize,
    ::serde::Serialize,
    ::typed_builder::TypedBuilder,
    Debug,
    PartialEq,
    Clone,
    Default,
)]
#[serde(rename_all = "camelCase")]
pub struct Tables {
    /// The range partitioning specification for this table, if configured.
    #[builder(setter(into))]
    #[serde(default)]
    pub range_partitioning: RangePartitioning,
    /// The time-based partitioning specification for this table, if configured.
    #[builder(setter(into))]
    #[serde(default)]
    pub time_partitioning: TimePartitioning,
    /// The labels associated with this table. You can use these to organize and group your tables.
    #[builder(setter(into))]
    #[serde(default)]
    pub labels: Labels,
    /// The time when this table was created, in milliseconds since the epoch.
    #[builder(setter(into))]
    #[serde(with = "with::int64")]
    #[serde(default)]
    pub creation_time: i64,
    /// A reference uniquely identifying the table.
    #[builder(setter(into))]
    #[serde(default)]
    pub table_reference: TableReference,
    /// The user-friendly name for this table.
    #[builder(setter(into))]
    #[serde(default)]
    pub friendly_name: ::std::string::String,
    /// [Optional] The time when this table expires, in milliseconds since the epoch. If not
    /// present, the table will persist indefinitely. Expired tables will be deleted and their
    /// storage reclaimed.
    #[builder(setter(into))]
    #[serde(with = "with::int64::option")]
    #[serde(default)]
    pub expiration_time: ::std::option::Option<i64>,
    /// The resource type.
    #[builder(setter(into))]
    #[serde(default)]
    pub kind: ::std::string::String,
    /// [Beta] Clustering specification for this table, if configured.
    #[builder(setter(into))]
    #[serde(default)]
    pub clustering: Clustering,
    /// Additional details for a view.
    #[builder(setter(into))]
    #[serde(default)]
    pub view: View,
    /// An opaque ID of the table
    #[builder(setter(into))]
    #[serde(default)]
    pub id: ::std::string::String,
    /// The type of table. Possible values are: TABLE, VIEW.
    #[builder(setter(into))]
    #[serde(rename = "type")]
    #[serde(default)]
    pub ty: ::std::string::String,
}

#[derive(
    ::serde::Deserialize,
    ::serde::Serialize,
    ::typed_builder::TypedBuilder,
    Debug,
    PartialEq,
    Clone,
    Default,
)]
#[serde(rename_all = "camelCase")]
pub struct TableList {
    /// A token to request the next page of results.
    #[builder(setter(into))]
    #[serde(default)]
    pub next_page_token: ::std::string::String,
    /// The type of list.
    #[builder(setter(into))]
    #[serde(default)]
    pub kind: ::std::string::String,
    /// Tables in the requested dataset.
    #[builder(setter(into))]
    #[serde(default)]
    pub tables: ::std::vec::Vec<Tables>,
    /// The total number of tables in the dataset.
    #[builder(setter(into))]
    #[serde(default)]
    pub total_items: i64,
    /// A hash of this page of results.
    #[builder(setter(into))]
    #[serde(default)]
    pub etag: ::std::string::String,
}

#[derive(
    ::serde::Deserialize, ::serde::Serialize, ::typed_builder::TypedBuilder, Debug, PartialEq, Clone,
)]
#[serde(rename_all = "camelCase")]
pub struct JobStatistics {
    /// [Output-only] Creation time of this job, in milliseconds since the epoch. This field will
    /// be present on all jobs.
    #[builder(setter(into))]
    #[serde(with = "with::int64::option")]
    #[serde(default)]
    pub creation_time: ::std::option::Option<i64>,
    /// [Output-only] Job resource usage breakdown by reservation.
    #[builder(setter(into))]
    #[serde(default)]
    pub reservation_usage: ::std::vec::Vec<ReservationUsage>,
    /// [Output-only] Statistics for a child job of a script.
    #[builder(setter(into))]
    #[serde(default)]
    pub script_statistics: ScriptStatistics,
    /// [Output-only] [Alpha] Information of the multi-statement transaction if this job is part of
    /// one.
    #[builder(setter(into))]
    #[serde(default)]
    pub transaction_info: TransactionInfo,
    /// [Output-only] Quotas which delayed this job's start time.
    #[builder(setter(into))]
    #[serde(default)]
    pub quota_deferments: ::std::vec::Vec<::std::string::String>,
    /// [Output-only] Slot-milliseconds for the job.
    #[builder(setter(into))]
    #[serde(with = "with::int64::option")]
    #[serde(default)]
    pub total_slot_ms: ::std::option::Option<i64>,
    /// [Output-only] Statistics for a copy job.
    #[builder(setter(into))]
    #[serde(default)]
    pub copy: JobStatistics5,
    /// [Output-only] Start time of this job, in milliseconds since the epoch. This field will be
    /// present when the job transitions from the PENDING state to either RUNNING or DONE.
    #[builder(setter(into))]
    #[serde(with = "with::int64::option")]
    #[serde(default)]
    pub start_time: ::std::option::Option<i64>,
    /// [Output-only] End time of this job, in milliseconds since the epoch. This field will be
    /// present whenever a job is in the DONE state.
    #[builder(setter(into))]
    #[serde(with = "with::int64::option")]
    #[serde(default)]
    pub end_time: ::std::option::Option<i64>,
    /// [Output-only] Number of child jobs executed.
    #[builder(setter(into))]
    #[serde(with = "with::int64::option")]
    #[serde(default)]
    pub num_child_jobs: ::std::option::Option<i64>,
    /// [Output-only] Name of the primary reservation assigned to this job. Note that this could be
    /// different than reservations reported in the reservation usage field if parent reservations
    /// were used to execute this job.
    #[builder(setter(into))]
    #[serde(default)]
    pub reservation_id: ::std::option::Option<::std::string::String>,
    /// [Output-only] Statistics for a load job.
    #[builder(setter(into))]
    #[serde(default)]
    pub load: JobStatistics3,
    /// [Output-only] If this is a child job, the id of the parent.
    #[builder(setter(into))]
    #[serde(default)]
    pub parent_job_id: ::std::option::Option<::std::string::String>,
    /// [Output-only] [Deprecated] Use the bytes processed in the query statistics instead.
    #[builder(setter(into))]
    #[serde(with = "with::int64::option")]
    #[serde(default)]
    pub total_bytes_processed: ::std::option::Option<i64>,
    /// [Output-only] Statistics for data masking. Present only for query and extract jobs.
    #[builder(setter(into))]
    #[serde(default)]
    pub data_masking_statistics: ::std::option::Option<DataMaskingStatistics>,
    /// [Output-only] Statistics for an extract job.
    #[builder(setter(into))]
    #[serde(default)]
    pub extract: JobStatistics4,
    /// [Output-only] Statistics for a query job.
    #[builder(setter(into))]
    pub query: JobStatistics2,
    /// [TrustedTester] [Output-only] Job progress (0.0 -> 1.0) for LOAD and EXTRACT jobs.
    #[builder(setter(into))]
    #[serde(default)]
    pub completion_ratio: f64,
    /// [Output-only] [Preview] Information of the session if this job is part of one.
    #[builder(setter(into))]
    #[serde(default)]
    pub session_info: ::std::option::Option<SessionInfo>,
    /// [Output-only] [Preview] Statistics for row-level security. Present only for query and
    /// extract jobs.
    #[builder(setter(into))]
    #[serde(default)]
    pub row_level_security_statistics: ::std::option::Option<RowLevelSecurityStatistics>,
}

#[derive(
    ::serde::Deserialize,
    ::serde::Serialize,
    ::typed_builder::TypedBuilder,
    Debug,
    PartialEq,
    Clone,
    Default,
)]
#[serde(rename_all = "camelCase")]
pub struct SessionInfo {
    /// [Output-only] // [Preview] Id of the session.
    #[builder(setter(into))]
    #[serde(default)]
    pub session_id: ::std::option::Option<::std::string::String>,
}

#[derive(
    ::serde::Deserialize,
    ::serde::Serialize,
    ::typed_builder::TypedBuilder,
    Debug,
    PartialEq,
    Clone,
    Default,
)]
#[serde(rename_all = "camelCase")]
pub struct EncryptionConfiguration {
    /// [Optional] Describes the Cloud KMS encryption key that will be used to protect destination
    /// BigQuery table. The BigQuery Service Account associated with your project requires access
    /// to this encryption key.
    #[builder(setter(into))]
    #[serde(default)]
    pub kms_key_name: ::std::option::Option<::std::string::String>,
}

#[derive(
    ::serde::Deserialize,
    ::serde::Serialize,
    ::typed_builder::TypedBuilder,
    Debug,
    PartialEq,
    Clone,
    Default,
)]
#[serde(rename_all = "camelCase")]
pub struct TimePartitioning {
    /// [Beta] [Optional] If not set, the table is partitioned by pseudo column, referenced via
    /// either '_PARTITIONTIME' as TIMESTAMP type, or '_PARTITIONDATE' as DATE type. If field
    /// is specified, the table is instead partitioned by this field. The field must be a
    /// top-level TIMESTAMP or DATE field. Its mode must be NULLABLE or REQUIRED.
    #[builder(setter(into))]
    #[serde(default)]
    pub field: ::std::option::Option<::std::string::String>,
    /// [Optional] Number of milliseconds for which to keep the storage for partitions in the
    /// table. The storage in a partition will have an expiration time of its partition time
    /// plus this value.
    #[builder(setter(into))]
    #[serde(with = "with::int64::option")]
    #[serde(default)]
    pub expiration_ms: ::std::option::Option<i64>,
    /// [Required] The supported types are DAY, HOUR, MONTH, and YEAR, which will generate one
    /// partition per day, hour, month, and year, respectively. When the type is not specified,
    /// the default behavior is DAY.
    #[builder(setter(into))]
    #[serde(rename = "type")]
    #[serde(default)]
    pub ty: ::std::string::String,
    #[builder(setter(into))]
    #[serde(default)]
    pub require_partition_filter: bool,
}

#[derive(
    ::serde::Deserialize,
    ::serde::Serialize,
    ::typed_builder::TypedBuilder,
    Debug,
    PartialEq,
    Clone,
    Default,
)]
#[serde(rename_all = "camelCase")]
pub struct Streamingbuffer {
    /// [Output-only] A lower-bound estimate of the number of rows currently in the streaming
    /// buffer.
    #[builder(setter(into))]
    #[serde(with = "with::uint64::option")]
    #[serde(default)]
    pub estimated_rows: ::std::option::Option<u64>,
    /// [Output-only] Contains the timestamp of the oldest entry in the streaming buffer, in
    /// milliseconds since the epoch, if the streaming buffer is available.
    #[builder(setter(into))]
    #[serde(with = "with::uint64::option")]
    #[serde(default)]
    pub oldest_entry_time: ::std::option::Option<u64>,
    /// [Output-only] A lower-bound estimate of the number of bytes currently in the streaming
    /// buffer.
    #[builder(setter(into))]
    #[serde(with = "with::uint64::option")]
    #[serde(default)]
    pub estimated_bytes: ::std::option::Option<u64>,
}

#[derive(
    ::serde::Deserialize,
    ::serde::Serialize,
    ::typed_builder::TypedBuilder,
    Debug,
    PartialEq,
    Clone,
    Default,
)]
#[serde(rename_all = "camelCase")]
pub struct DmlStatistics {
    /// Number of inserted Rows. Populated by DML INSERT and MERGE statements.
    #[builder(setter(into))]
    #[serde(with = "with::int64")]
    #[serde(default)]
    pub inserted_row_count: i64,
    /// Number of updated Rows. Populated by DML UPDATE and MERGE statements.
    #[builder(setter(into))]
    #[serde(with = "with::int64")]
    #[serde(default)]
    pub updated_row_count: i64,
    /// Number of deleted Rows. populated by DML DELETE, MERGE and TRUNCATE statements.
    #[builder(setter(into))]
    #[serde(with = "with::int64")]
    #[serde(default)]
    pub deleted_row_count: i64,
}

#[derive(
    ::serde::Deserialize,
    ::serde::Serialize,
    ::typed_builder::TypedBuilder,
    Debug,
    PartialEq,
    Clone,
    Default,
)]
#[serde(rename_all = "camelCase")]
pub struct ListRoutinesResponse {
    /// A token to request the next page of results.
    #[builder(setter(into))]
    #[serde(default)]
    pub next_page_token: ::std::string::String,
    /// Routines in the requested dataset. Unless read_mask is set in the request, only the
    /// following fields are populated: etag, project_id, dataset_id, routine_id, routine_type,
    /// creation_time, last_modified_time, and language.
    #[builder(setter(into))]
    #[serde(default)]
    pub routines: ::std::vec::Vec<Routine>,
}

#[derive(
    ::serde::Deserialize,
    ::serde::Serialize,
    ::typed_builder::TypedBuilder,
    Debug,
    PartialEq,
    Clone,
    Default,
)]
#[serde(rename_all = "camelCase")]
pub struct Datasets {
    /// The labels associated with this dataset. You can use these to organize and group your
    /// datasets.
    #[builder(setter(into))]
    #[serde(default)]
    pub labels: Labels,
    /// The geographic location where the data resides.
    #[builder(setter(into))]
    #[serde(default)]
    pub location: ::std::string::String,
    /// The dataset reference. Use this property to access specific parts of the dataset's ID, such
    /// as project ID or dataset ID.
    #[builder(setter(into))]
    #[serde(default)]
    pub dataset_reference: DatasetReference,
    /// The fully-qualified, unique, opaque ID of the dataset.
    #[builder(setter(into))]
    #[serde(default)]
    pub id: ::std::string::String,
    /// A descriptive name for the dataset, if one exists.
    #[builder(setter(into))]
    #[serde(default)]
    pub friendly_name: ::std::string::String,
    /// The resource type. This property always returns the value "bigquery#dataset".
    #[builder(setter(into))]
    #[serde(default)]
    pub kind: ::std::string::String,
}

#[derive(
    ::serde::Deserialize,
    ::serde::Serialize,
    ::typed_builder::TypedBuilder,
    Debug,
    PartialEq,
    Clone,
    Default,
)]
#[serde(rename_all = "camelCase")]
pub struct DatasetList {
    /// A token that can be used to request the next results page. This property is omitted on the
    /// final results page.
    #[builder(setter(into))]
    #[serde(default)]
    pub next_page_token: ::std::string::String,
    /// An array of the dataset resources in the project. Each resource contains basic information.
    /// For full information about a particular dataset resource, use the Datasets: get method.
    /// This property is omitted when there are no datasets in the project.
    #[builder(setter(into))]
    #[serde(default)]
    pub datasets: ::std::vec::Vec<Datasets>,
    /// The list type. This property always returns the value "bigquery#datasetList".
    #[builder(setter(into))]
    #[serde(default)]
    pub kind: ::std::string::String,
    /// A hash value of the results page. You can use this property to determine if the page has
    /// changed since the last request.
    #[builder(setter(into))]
    #[serde(default)]
    pub etag: ::std::string::String,
}

#[derive(
    ::serde::Deserialize,
    ::serde::Serialize,
    ::typed_builder::TypedBuilder,
    Debug,
    PartialEq,
    Clone,
    Default,
)]
#[serde(rename_all = "camelCase")]
pub struct TableSchema {
    /// Describes the fields in a table.
    #[builder(setter(into))]
    #[serde(default)]
    pub fields: ::std::vec::Vec<TableFieldSchema>,
}

/// [Output-only, Beta] Model options used for the first training run. These options are immutable
/// for subsequent training runs. Default values are used for any options not specified in the input
/// query.
#[derive(
    ::serde::Deserialize,
    ::serde::Serialize,
    ::typed_builder::TypedBuilder,
    Debug,
    PartialEq,
    Clone,
    Default,
)]
#[serde(rename_all = "camelCase")]
pub struct ModelOptions {
    #[builder(setter(into))]
    #[serde(default)]
    pub loss_type: ::std::string::String,
    #[builder(setter(into))]
    #[serde(default)]
    pub labels: ::std::vec::Vec<::std::string::String>,
    #[builder(setter(into))]
    #[serde(default)]
    pub model_type: ::std::string::String,
}

#[derive(
    ::serde::Deserialize,
    ::serde::Serialize,
    ::typed_builder::TypedBuilder,
    Debug,
    PartialEq,
    Clone,
    Default,
)]
#[serde(rename_all = "camelCase")]
pub struct ModelDefinition {
    /// [Output-only, Beta] Model options used for the first training run. These options are
    /// immutable for subsequent training runs. Default values are used for any options not
    /// specified in the input query.
    #[builder(setter(into))]
    #[serde(default)]
    pub model_options: ModelOptions,
    /// [Output-only, Beta] Information about ml training runs, each training run comprises of
    /// multiple iterations and there may be multiple training runs for the model if warm start
    /// is used or if a user decides to continue a previously cancelled query.
    #[builder(setter(into))]
    #[serde(default)]
    pub training_runs: ::std::vec::Vec<BqmlTrainingRun>,
}

#[derive(
    ::serde::Deserialize,
    ::serde::Serialize,
    ::typed_builder::TypedBuilder,
    Debug,
    PartialEq,
    Clone,
    Default,
)]
#[serde(rename_all = "camelCase")]
pub struct JobConfigurationTableCopy {
    /// [Optional] Supported operation types in table copy job.
    #[builder(setter(into))]
    #[serde(default)]
    pub operation_type: ::std::option::Option<::std::string::String>,
    /// [Required] The destination table
    #[builder(setter(into))]
    #[serde(default)]
    pub destination_table: TableReference,
    /// [Optional] The time when the destination table expires. Expired tables will be deleted and
    /// their storage reclaimed.
    #[builder(setter(into))]
    #[serde(default)]
    pub destination_expiration_time: ::serde_json::Value,
    /// [Optional] Specifies whether the job is allowed to create new tables. The following values
    /// are supported: CREATE_IF_NEEDED: If the table does not exist, BigQuery creates the
    /// table. CREATE_NEVER: The table must already exist. If it does not, a 'notFound' error
    /// is returned in the job result. The default value is CREATE_IF_NEEDED. Creation,
    /// truncation and append actions occur as one atomic update upon job completion.
    #[builder(setter(into))]
    #[serde(default)]
    pub create_disposition: ::std::option::Option<::std::string::String>,
    /// [Pick one] Source table to copy.
    #[builder(setter(into))]
    #[serde(default)]
    pub source_table: TableReference,
    /// [Pick one] Source tables to copy.
    #[builder(setter(into))]
    #[serde(default)]
    pub source_tables: ::std::vec::Vec<TableReference>,
    /// [Optional] Specifies the action that occurs if the destination table already exists. The
    /// following values are supported: WRITE_TRUNCATE: If the table already exists, BigQuery
    /// overwrites the table data. WRITE_APPEND: If the table already exists, BigQuery appends
    /// the data to the table. WRITE_EMPTY: If the table already exists and contains data, a
    /// 'duplicate' error is returned in the job result. The default value is WRITE_EMPTY. Each
    /// action is atomic and only occurs if BigQuery is able to complete the job successfully.
    /// Creation, truncation and append actions occur as one atomic update upon job completion.
    #[builder(setter(into))]
    #[serde(default)]
    pub write_disposition: ::std::option::Option<::std::string::String>,
    /// Custom encryption configuration (e.g., Cloud KMS keys).
    #[builder(setter(into))]
    #[serde(default)]
    pub destination_encryption_configuration: EncryptionConfiguration,
}

#[derive(
    ::serde::Deserialize, ::serde::Serialize, ::typed_builder::TypedBuilder, Debug, PartialEq, Clone,
)]
#[serde(rename_all = "camelCase")]
pub struct JobConfiguration {
    /// [Optional] If set, don't actually run this job. A valid query will return a mostly empty
    /// response with some processing statistics, while an invalid query will return the same error
    /// it would if it wasn't a dry run. Behavior of non-query jobs is undefined.
    #[builder(setter(into))]
    #[serde(default)]
    pub dry_run: ::std::option::Option<bool>,
    /// [Output-only] The type of the job. Can be QUERY, LOAD, EXTRACT, COPY or UNKNOWN.
    #[builder(setter(into))]
    #[serde(default)]
    pub job_type: ::std::option::Option<::std::string::String>,
    /// [Pick one] Copies a table.
    #[builder(setter(into))]
    pub copy: JobConfigurationTableCopy,
    /// [Optional] Job timeout in milliseconds. If this time limit is exceeded, BigQuery may
    /// attempt to terminate the job.
    #[builder(setter(into))]
    #[serde(with = "with::int64::option")]
    #[serde(default)]
    pub job_timeout_ms: ::std::option::Option<i64>,
    /// [Pick one] Configures a load job.
    #[builder(setter(into))]
    pub load: JobConfigurationLoad,
    /// [Pick one] Configures a query job.
    #[builder(setter(into))]
    pub query: JobConfigurationQuery,
    /// [Pick one] Configures an extract job.
    #[builder(setter(into))]
    pub extract: JobConfigurationExtract,
    /// The labels associated with this job. You can use these to organize and group your jobs.
    /// Label keys and values can be no longer than 63 characters, can only contain lowercase
    /// letters, numeric characters, underscores and dashes. International characters are
    /// allowed. Label values are optional. Label keys must start with a letter and each label
    /// in the list must have a different key.
    #[builder(setter(into))]
    #[serde(default)]
    pub labels: Labels,
}

/// Discrete candidates of a double hyperparameter.
#[derive(
    ::serde::Deserialize,
    ::serde::Serialize,
    ::typed_builder::TypedBuilder,
    Debug,
    PartialEq,
    Clone,
    Default,
)]
#[serde(rename_all = "camelCase")]
pub struct DoubleCandidates {
    /// Candidates for the double parameter in increasing order.
    #[builder(setter(into))]
    #[serde(default)]
    pub candidates: ::std::vec::Vec<f64>,
}

/// Optional. The determinism level of the JavaScript UDF, if defined.
#[derive(::serde::Deserialize, ::serde::Serialize, Debug, PartialEq, Clone)]
pub enum DeterminismLevel {
    /// The UDF is deterministic, meaning that 2 function calls with the same inputs always produce
    /// the same result, even across 2 query runs.
    #[serde(rename = "DETERMINISTIC")]
    Deterministic,
    /// The UDF is not deterministic.
    #[serde(rename = "NOT_DETERMINISTIC")]
    NotDeterministic,
}

/// Required. The type of routine.
#[derive(::serde::Deserialize, ::serde::Serialize, Debug, PartialEq, Clone)]
pub enum RoutineType {
    /// Non-builtin permanent scalar function.
    #[serde(rename = "SCALAR_FUNCTION")]
    ScalarFunction,
    /// Stored procedure.
    #[serde(rename = "PROCEDURE")]
    Procedure,
    /// Non-builtin permanent TVF.
    #[serde(rename = "TABLE_VALUED_FUNCTION")]
    TableValuedFunction,
}

/// Optional. Defaults to "SQL".
#[derive(::serde::Deserialize, ::serde::Serialize, Debug, PartialEq, Clone)]
pub enum Language {
    /// SQL language.
    #[serde(rename = "SQL")]
    Sql,
    /// JavaScript language.
    #[serde(rename = "JAVASCRIPT")]
    Javascript,
    /// Python language.
    #[serde(rename = "PYTHON")]
    Python,
}

/// A user-defined function or a stored procedure.
#[derive(
    ::serde::Deserialize, ::serde::Serialize, ::typed_builder::TypedBuilder, Debug, PartialEq, Clone,
)]
#[serde(rename_all = "camelCase")]
pub struct Routine {
    /// Optional. Remote function specific options.
    #[builder(setter(into))]
    #[serde(default)]
    pub remote_function_options: RemoteFunctionOptions,
    /// Optional. If language = "JAVASCRIPT", this field stores the path of the imported JAVASCRIPT
    /// libraries.
    #[builder(setter(into))]
    #[serde(default)]
    pub imported_libraries: ::std::vec::Vec<::std::string::String>,
    /// Output only. The time when this routine was created, in milliseconds since the epoch.
    #[builder(setter(into))]
    #[serde(with = "with::int64")]
    #[serde(default)]
    pub creation_time: i64,
    /// Optional.
    #[builder(setter(into))]
    #[serde(default)]
    pub arguments: ::std::vec::Vec<Argument>,
    /// Required. The body of the routine. For functions, this is the expression in the AS clause.
    /// If language=SQL, it is the substring inside (but excluding) the parentheses. For
    /// example, for the function created with the following statement: `CREATE FUNCTION
    /// JoinLines(x string, y string) as (concat(x, "\n", y))` The definition_body is
    /// `concat(x, "\n", y)` (\n is not replaced with linebreak). If language=JAVASCRIPT, it is
    /// the evaluated string in the AS clause. For example, for the function created with the
    /// following statement: `CREATE FUNCTION f() RETURNS STRING LANGUAGE js AS 'return
    /// "\n";\n'` The definition_body is `return "\n";\n` Note that both \n are replaced with
    /// linebreaks.
    #[builder(setter(into))]
    #[serde(default)]
    pub definition_body: ::std::string::String,
    /// Optional. The determinism level of the JavaScript UDF, if defined.
    #[builder(setter(into))]
    pub determinism_level: DeterminismLevel,
    /// Optional. Spark specific options.
    #[builder(setter(into))]
    #[serde(default)]
    pub spark_options: SparkOptions,
    /// Required. The type of routine.
    #[builder(setter(into))]
    pub routine_type: RoutineType,
    /// Optional if language = "SQL"; required otherwise. Cannot be set if routine_type =
    /// "TABLE_VALUED_FUNCTION". If absent, the return type is inferred from definition_body at
    /// query time in each query that references this routine. If present, then the evaluated
    /// result will be cast to the specified returned type at query time. For example, for the
    /// functions created with the following statements: * `CREATE FUNCTION Add(x FLOAT64, y
    /// FLOAT64) RETURNS FLOAT64 AS (x + y);` * `CREATE FUNCTION Increment(x FLOAT64) AS
    /// (Add(x, 1));` * `CREATE FUNCTION Decrement(x FLOAT64) RETURNS FLOAT64 AS (Add(x, -1));`
    /// The return_type is `{type_kind: "FLOAT64"}` for `Add` and `Decrement`, and is absent
    /// for `Increment` (inferred as FLOAT64 at query time). Suppose the function `Add` is
    /// replaced by `CREATE OR REPLACE FUNCTION Add(x INT64, y INT64) AS (x + y);`
    /// Then the inferred return type of `Increment` is automatically changed to INT64 at query
    /// time, while the return type of `Decrement` remains FLOAT64.
    #[builder(setter(into))]
    #[serde(default)]
    pub return_type: ::std::option::Option<StandardSqlDataType>,
    /// Required. Reference describing the ID of this routine.
    #[builder(setter(into))]
    #[serde(default)]
    pub routine_reference: RoutineReference,
    /// Optional. Defaults to "SQL".
    #[builder(setter(into))]
    pub language: Language,
    /// Output only. The time when this routine was last modified, in milliseconds since the epoch.
    #[builder(setter(into))]
    #[serde(with = "with::int64")]
    #[serde(default)]
    pub last_modified_time: i64,
    /// Optional. The description of the routine, if defined.
    #[builder(setter(into))]
    #[serde(default)]
    pub description: ::std::option::Option<::std::string::String>,
    /// Optional. Can be set for procedures only. If true (default), the definition body will be
    /// validated in the creation and the updates of the procedure. For procedures with an argument
    /// of ANY TYPE, the definition body validtion is not supported at creation/update time,
    /// and thus this field must be set to false explicitly.
    #[builder(setter(into))]
    #[serde(default)]
    pub strict_mode: ::std::option::Option<bool>,
    /// Optional. Can be set only if routine_type = "TABLE_VALUED_FUNCTION". If absent, the return
    /// table type is inferred from definition_body at query time in each query that references
    /// this routine. If present, then the columns in the evaluated table result will be cast
    /// to match the column types specificed in return table type, at query time.
    #[builder(setter(into))]
    #[serde(default)]
    pub return_table_type: StandardSqlTableType,
    /// Output only. A hash of this resource.
    #[builder(setter(into))]
    #[serde(default)]
    pub etag: ::std::string::String,
}

#[derive(
    ::serde::Deserialize,
    ::serde::Serialize,
    ::typed_builder::TypedBuilder,
    Debug,
    PartialEq,
    Clone,
    Default,
)]
#[serde(rename_all = "camelCase")]
pub struct Rows {
    /// [Optional] A unique ID for each row. BigQuery uses this property to detect duplicate
    /// insertion requests on a best-effort basis.
    #[builder(setter(into))]
    #[serde(default)]
    pub insert_id: ::std::option::Option<::std::string::String>,
    /// [Required] A JSON object that contains a row of data. The object's properties and values
    /// must match the destination table's schema.
    #[builder(setter(into))]
    #[serde(default)]
    pub json: JsonObject,
}

#[derive(
    ::serde::Deserialize,
    ::serde::Serialize,
    ::typed_builder::TypedBuilder,
    Debug,
    PartialEq,
    Clone,
    Default,
)]
#[serde(rename_all = "camelCase")]
pub struct TableDataInsertAllRequest {
    /// [Optional] Accept rows that contain values that do not match the schema. The unknown values
    /// are ignored. Default is false, which treats unknown values as errors.
    #[builder(setter(into))]
    #[serde(default)]
    pub ignore_unknown_values: ::std::option::Option<bool>,
    /// If specified, treats the destination table as a base template, and inserts the rows into an
    /// instance table named "{destination}{templateSuffix}". BigQuery will manage creation of the
    /// instance table, using the schema of the base template table. See
    /// https://cloud.google.com/bigquery/streaming-data-into-bigquery#template-tables for
    /// considerations when working with templates tables.
    #[builder(setter(into))]
    #[serde(default)]
    pub template_suffix: ::std::string::String,
    /// [Optional] Insert all valid rows of a request, even if invalid rows exist. The default
    /// value is false, which causes the entire request to fail if any invalid rows exist.
    #[builder(setter(into))]
    #[serde(default)]
    pub skip_invalid_rows: ::std::option::Option<bool>,
    /// The rows to insert.
    #[builder(setter(into))]
    #[serde(default)]
    pub rows: ::std::vec::Vec<Rows>,
    /// The resource type of the response.
    #[builder(setter(into))]
    #[serde(default)]
    pub kind: ::std::string::String,
}

/// Response message for the ListRowAccessPolicies method.
#[derive(
    ::serde::Deserialize,
    ::serde::Serialize,
    ::typed_builder::TypedBuilder,
    Debug,
    PartialEq,
    Clone,
    Default,
)]
#[serde(rename_all = "camelCase")]
pub struct ListRowAccessPoliciesResponse {
    /// A token to request the next page of results.
    #[builder(setter(into))]
    #[serde(default)]
    pub next_page_token: ::std::string::String,
    /// Row access policies on the requested table.
    #[builder(setter(into))]
    #[serde(default)]
    pub row_access_policies: ::std::vec::Vec<RowAccessPolicy>,
}

/// A single entry in the confusion matrix.
#[derive(
    ::serde::Deserialize,
    ::serde::Serialize,
    ::typed_builder::TypedBuilder,
    Debug,
    PartialEq,
    Clone,
    Default,
)]
#[serde(rename_all = "camelCase")]
pub struct Entry {
    /// The predicted label. For confidence_threshold > 0, we will also add an entry indicating the
    /// number of items under the confidence threshold.
    #[builder(setter(into))]
    #[serde(default)]
    pub predicted_label: ::std::string::String,
    /// Number of items being predicted as this label.
    #[builder(setter(into))]
    #[serde(with = "with::int64")]
    #[serde(default)]
    pub item_count: i64,
}

/// (Auto-)arima fitting result. Wrap everything in ArimaResult for easier refactoring if we want to
/// use model-specific iteration results.
#[derive(
    ::serde::Deserialize,
    ::serde::Serialize,
    ::typed_builder::TypedBuilder,
    Debug,
    PartialEq,
    Clone,
    Default,
)]
#[serde(rename_all = "camelCase")]
pub struct ArimaResult {
    /// This message is repeated because there are multiple arima models fitted in auto-arima. For
    /// non-auto-arima model, its size is one.
    #[builder(setter(into))]
    #[serde(default)]
    pub arima_model_info: ::std::vec::Vec<ArimaModelInfo>,
    /// Seasonal periods. Repeated because multiple periods are supported for one time series.
    #[builder(setter(into))]
    #[serde(default)]
    pub seasonal_periods: ::std::vec::Vec<SeasonalPeriods>,
}

/// Required. The top level type of this field. Can be any standard SQL data type (e.g., "INT64",
/// "DATE", "ARRAY").
#[derive(::serde::Deserialize, ::serde::Serialize, Debug, PartialEq, Clone)]
pub enum TypeKind {
    /// Encoded as a string in decimal format.
    #[serde(rename = "INT64")]
    Int64,
    /// Encoded as a boolean "false" or "true".
    #[serde(rename = "BOOL")]
    Bool,
    /// Encoded as a number, or string "NaN", "Infinity" or "-Infinity".
    #[serde(rename = "FLOAT64")]
    Float64,
    /// Encoded as a string value.
    #[serde(rename = "STRING")]
    String,
    /// Encoded as a base64 string per RFC 4648, section 4.
    #[serde(rename = "BYTES")]
    Bytes,
    /// Encoded as an RFC 3339 timestamp with mandatory "Z" time zone string:
    /// 1985-04-12T23:20:50.52Z
    #[serde(rename = "TIMESTAMP")]
    Timestamp,
    /// Encoded as RFC 3339 full-date format string: 1985-04-12
    #[serde(rename = "DATE")]
    Date,
    /// Encoded as RFC 3339 partial-time format string: 23:20:50.52
    #[serde(rename = "TIME")]
    Time,
    /// Encoded as RFC 3339 full-date "T" partial-time: 1985-04-12T23:20:50.52
    #[serde(rename = "DATETIME")]
    Datetime,
    /// Encoded as fully qualified 3 part: 0-5 15 2:30:45.6
    #[serde(rename = "INTERVAL")]
    Interval,
    /// Encoded as WKT
    #[serde(rename = "GEOGRAPHY")]
    Geography,
    /// Encoded as a decimal string.
    #[serde(rename = "NUMERIC")]
    Numeric,
    /// Encoded as a decimal string.
    #[serde(rename = "BIGNUMERIC")]
    Bignumeric,
    /// Encoded as a string.
    #[serde(rename = "JSON")]
    Json,
    /// Encoded as a list with types matching Type.array_type.
    #[serde(rename = "ARRAY")]
    Array,
    /// Encoded as a list with fields of type Type.struct_type[i]. List is used because a JSON
    /// object cannot have duplicate field names.
    #[serde(rename = "STRUCT")]
    Struct,
}

/// The data type of a variable such as a function argument. Examples include: * INT64:
/// `{"typeKind": "INT64"}` * ARRAY: { "typeKind": "ARRAY", "arrayElementType": {"typeKind":
/// "STRING"} } * STRUCT>: { "typeKind": "STRUCT", "structType": { "fields": [ { "name": "x",
/// "type": {"typeKind": "STRING"} }, { "name": "y", "type": { "typeKind": "ARRAY",
/// "arrayElementType": {"typeKind": "DATE"} } } ] } }
#[derive(
    ::serde::Deserialize, ::serde::Serialize, ::typed_builder::TypedBuilder, Debug, PartialEq, Clone,
)]
#[serde(rename_all = "camelCase")]
pub struct StandardSqlDataType {
    /// The fields of this struct, in order, if type_kind = "STRUCT".
    #[builder(setter(into))]
    #[serde(default)]
    pub struct_type: StandardSqlStructType,
    /// Required. The top level type of this field. Can be any standard SQL data type (e.g.,
    /// "INT64", "DATE", "ARRAY").
    #[builder(setter(into))]
    pub type_kind: TypeKind,
    /// The type of the array's elements, if type_kind = "ARRAY".
    #[builder(setter(into))]
    pub array_element_type: ::std::boxed::Box<StandardSqlDataType>,
}

#[derive(
    ::serde::Deserialize,
    ::serde::Serialize,
    ::typed_builder::TypedBuilder,
    Debug,
    PartialEq,
    Clone,
    Default,
)]
#[serde(rename_all = "camelCase")]
pub struct TableDataList {
    /// The resource type of the response.
    #[builder(setter(into))]
    #[serde(default)]
    pub kind: ::std::string::String,
    /// The total number of rows in the complete table.
    #[builder(setter(into))]
    #[serde(with = "with::int64")]
    #[serde(default)]
    pub total_rows: i64,
    /// A token used for paging results. Providing this token instead of the startIndex parameter
    /// can help you retrieve stable results when an underlying table is changing.
    #[builder(setter(into))]
    #[serde(default)]
    pub page_token: ::std::string::String,
    /// A hash of this page of results.
    #[builder(setter(into))]
    #[serde(default)]
    pub etag: ::std::string::String,
    /// Rows of results.
    #[builder(setter(into))]
    #[serde(default)]
    pub rows: ::std::vec::Vec<TableRow>,
}

/// Aggregate metrics for classification/classifier models. For multi-class models, the metrics are
/// either macro-averaged or micro-averaged. When macro-averaged, the metrics are calculated for
/// each label and then an unweighted average is taken of those values. When micro-averaged, the
/// metric is calculated globally by counting the total number of correctly predicted rows.
#[derive(
    ::serde::Deserialize,
    ::serde::Serialize,
    ::typed_builder::TypedBuilder,
    Debug,
    PartialEq,
    Clone,
    Default,
)]
#[serde(rename_all = "camelCase")]
pub struct AggregateClassificationMetrics {
    /// Threshold at which the metrics are computed. For binary classification models this is the
    /// positive class threshold. For multi-class classfication models this is the confidence
    /// threshold.
    #[builder(setter(into))]
    #[serde(default)]
    pub threshold: f64,
    /// Precision is the fraction of actual positive predictions that had positive actual labels.
    /// For multiclass this is a macro-averaged metric treating each class as a binary
    /// classifier.
    #[builder(setter(into))]
    #[serde(default)]
    pub precision: f64,
    /// Logarithmic Loss. For multiclass this is a macro-averaged metric.
    #[builder(setter(into))]
    #[serde(default)]
    pub log_loss: f64,
    /// Accuracy is the fraction of predictions given the correct label. For multiclass this is a
    /// micro-averaged metric.
    #[builder(setter(into))]
    #[serde(default)]
    pub accuracy: f64,
    /// The F1 score is an average of recall and precision. For multiclass this is a macro-averaged
    /// metric.
    #[builder(setter(into))]
    #[serde(default)]
    pub f_1_score: f64,
    /// Area Under a ROC Curve. For multiclass this is a macro-averaged metric.
    #[builder(setter(into))]
    #[serde(default)]
    pub roc_auc: f64,
    /// Recall is the fraction of actual positive labels that were given a positive prediction. For
    /// multiclass this is a macro-averaged metric.
    #[builder(setter(into))]
    #[serde(default)]
    pub recall: f64,
}

/// Model evaluation metrics for a single ARIMA forecasting model.
#[derive(
    ::serde::Deserialize,
    ::serde::Serialize,
    ::typed_builder::TypedBuilder,
    Debug,
    PartialEq,
    Clone,
    Default,
)]
#[serde(rename_all = "camelCase")]
pub struct ArimaSingleModelForecastingMetrics {
    /// Non-seasonal order.
    #[builder(setter(into))]
    #[serde(default)]
    pub non_seasonal_order: ArimaOrder,
    /// If true, holiday_effect is a part of time series decomposition result.
    #[builder(setter(into))]
    #[serde(default)]
    pub has_holiday_effect: bool,
    /// Seasonal periods. Repeated because multiple periods are supported for one time series.
    #[builder(setter(into))]
    #[serde(default)]
    pub seasonal_periods: ::std::vec::Vec<SeasonalPeriods>,
    /// If true, spikes_and_dips is a part of time series decomposition result.
    #[builder(setter(into))]
    #[serde(default)]
    pub has_spikes_and_dips: bool,
    /// The tuple of time_series_ids identifying this time series. It will be one of the unique
    /// tuples of values present in the time_series_id_columns specified during ARIMA model
    /// training. Only present when time_series_id_columns training option was used and the
    /// order of values here are same as the order of time_series_id_columns.
    #[builder(setter(into))]
    #[serde(default)]
    pub time_series_ids: ::std::vec::Vec<::std::string::String>,
    /// The time_series_id value for this time series. It will be one of the unique values from the
    /// time_series_id_column specified during ARIMA model training. Only present when
    /// time_series_id_column training option was used.
    #[builder(setter(into))]
    #[serde(default)]
    pub time_series_id: ::std::string::String,
    /// Is arima model fitted with drift or not. It is always false when d is not 1.
    #[builder(setter(into))]
    #[serde(default)]
    pub has_drift: bool,
    /// Arima fitting metrics.
    #[builder(setter(into))]
    #[serde(default)]
    pub arima_fitting_metrics: ArimaFittingMetrics,
    /// If true, step_changes is a part of time series decomposition result.
    #[builder(setter(into))]
    #[serde(default)]
    pub has_step_changes: bool,
}

#[derive(
    ::serde::Deserialize,
    ::serde::Serialize,
    ::typed_builder::TypedBuilder,
    Debug,
    PartialEq,
    Clone,
    Default,
)]
#[serde(rename_all = "camelCase")]
pub struct BqmlIterationResult {
    /// [Output-only, Beta] Eval loss computed on the eval data at the end of the iteration. The
    /// eval loss is used for early stopping to avoid overfitting. No eval loss if
    /// eval_split_method option is specified as no_split or auto_split with input data size
    /// less than 500 rows.
    #[builder(setter(into))]
    #[serde(default)]
    pub eval_loss: ::std::option::Option<f64>,
    /// [Output-only, Beta] Training loss computed on the training data at the end of the
    /// iteration. The training loss function is defined by model type.
    #[builder(setter(into))]
    #[serde(default)]
    pub training_loss: ::std::option::Option<f64>,
    /// [Output-only, Beta] Time taken to run the training iteration in milliseconds.
    #[builder(setter(into))]
    #[serde(with = "with::int64::option")]
    #[serde(default)]
    pub duration_ms: ::std::option::Option<i64>,
    /// [Output-only, Beta] Learning rate used for this iteration, it varies for different training
    /// iterations if learn_rate_strategy option is not constant.
    #[builder(setter(into))]
    #[serde(default)]
    pub learn_rate: ::std::option::Option<f64>,
    /// [Output-only, Beta] Index of the ML training iteration, starting from zero for each
    /// training run.
    #[builder(setter(into))]
    #[serde(default)]
    pub index: ::std::option::Option<i64>,
}

/// This is used for defining User Defined Function (UDF) resources only when using legacy SQL.
/// Users of Standard SQL should leverage either DDL (e.g. CREATE [TEMPORARY] FUNCTION ... ) or the
/// Routines API to define UDF resources. For additional information on migrating, see:
/// https://cloud.google.com/bigquery/docs/reference/standard-sql/migrating-from-legacy-sql#differences_in_user-defined_javascript_functions
#[derive(
    ::serde::Deserialize,
    ::serde::Serialize,
    ::typed_builder::TypedBuilder,
    Debug,
    PartialEq,
    Clone,
    Default,
)]
#[serde(rename_all = "camelCase")]
pub struct UserDefinedFunctionResource {
    /// [Pick one] An inline resource that contains code for a user-defined function (UDF).
    /// Providing a inline code resource is equivalent to providing a URI for a file containing
    /// the same code.
    #[builder(setter(into))]
    #[serde(default)]
    pub inline_code: ::std::string::String,
    /// [Pick one] A code resource to load from a Google Cloud Storage URI (gs://bucket/path).
    #[builder(setter(into))]
    #[serde(default)]
    pub resource_uri: ::std::string::String,
}

/// Confusion matrix for binary classification models.
#[derive(
    ::serde::Deserialize,
    ::serde::Serialize,
    ::typed_builder::TypedBuilder,
    Debug,
    PartialEq,
    Clone,
    Default,
)]
#[serde(rename_all = "camelCase")]
pub struct BinaryConfusionMatrix {
    /// Number of true samples predicted as false.
    #[builder(setter(into))]
    #[serde(with = "with::int64")]
    #[serde(default)]
    pub true_negatives: i64,
    /// The fraction of predictions given the correct label.
    #[builder(setter(into))]
    #[serde(default)]
    pub accuracy: f64,
    /// The fraction of actual positive predictions that had positive actual labels.
    #[builder(setter(into))]
    #[serde(default)]
    pub precision: f64,
    /// Number of false samples predicted as false.
    #[builder(setter(into))]
    #[serde(with = "with::int64")]
    #[serde(default)]
    pub false_negatives: i64,
    /// The equally weighted average of recall and precision.
    #[builder(setter(into))]
    #[serde(default)]
    pub f_1_score: f64,
    /// Number of false samples predicted as true.
    #[builder(setter(into))]
    #[serde(with = "with::int64")]
    #[serde(default)]
    pub false_positives: i64,
    /// Threshold value used when computing each of the following metric.
    #[builder(setter(into))]
    #[serde(default)]
    pub positive_class_threshold: f64,
    /// Number of true samples predicted as true.
    #[builder(setter(into))]
    #[serde(with = "with::int64")]
    #[serde(default)]
    pub true_positives: i64,
    /// The fraction of actual positive labels that were given a positive prediction.
    #[builder(setter(into))]
    #[serde(default)]
    pub recall: f64,
}

#[derive(
    ::serde::Deserialize, ::serde::Serialize, ::typed_builder::TypedBuilder, Debug, PartialEq, Clone,
)]
#[serde(rename_all = "camelCase")]
pub struct JobConfigurationLoad {
    /// If true, creates a new session, where session id will be a server generated random id. If
    /// false, runs query with an existing session_id passed in ConnectionProperty, otherwise
    /// runs the load job in non-session mode.
    #[builder(setter(into))]
    #[serde(default)]
    pub create_session: bool,
    /// [Optional] Accept rows that are missing trailing optional columns. The missing values are
    /// treated as nulls. If false, records with missing trailing columns are treated as bad
    /// records, and if there are too many bad records, an invalid error is returned in the job
    /// result. The default value is false. Only applicable to CSV, ignored for other formats.
    #[builder(setter(into))]
    #[serde(default)]
    pub allow_jagged_rows: ::std::option::Option<bool>,
    /// [Optional] Indicates if BigQuery should allow extra values that are not represented in the
    /// table schema. If true, the extra values are ignored. If false, records with extra
    /// columns are treated as bad records, and if there are too many bad records, an invalid
    /// error is returned in the job result. The default value is false. The sourceFormat
    /// property determines what BigQuery treats as an extra value: CSV: Trailing columns JSON:
    /// Named values that don't match any column names
    #[builder(setter(into))]
    #[serde(default)]
    pub ignore_unknown_values: ::std::option::Option<bool>,
    /// [TrustedTester] Range partitioning specification for this table. Only one of
    /// timePartitioning and rangePartitioning should be specified.
    #[builder(setter(into))]
    #[serde(default)]
    pub range_partitioning: RangePartitioning,
    /// Custom encryption configuration (e.g., Cloud KMS keys).
    #[builder(setter(into))]
    #[serde(default)]
    pub destination_encryption_configuration: EncryptionConfiguration,
    /// [Optional] Options to configure parquet support.
    #[builder(setter(into))]
    #[serde(default)]
    pub parquet_options: ParquetOptions,
    /// [Optional] The value that is used to quote data sections in a CSV file. BigQuery converts
    /// the string to ISO-8859-1 encoding, and then uses the first byte of the encoded string
    /// to split the data in its raw, binary state. The default value is a double-quote ('"').
    /// If your data does not contain quoted sections, set the property value to an empty
    /// string. If your data contains quoted newline characters, you must also set the
    /// allowQuotedNewlines property to true.
    #[builder(setter(into))]
    #[serde(default)]
    pub quote: ::std::option::Option<::std::string::String>,
    /// [Deprecated] The format of the schemaInline property.
    #[builder(setter(into))]
    #[serde(default)]
    pub schema_inline_format: ::std::string::String,
    /// Connection properties.
    #[builder(setter(into))]
    #[serde(default)]
    pub connection_properties: ::std::vec::Vec<ConnectionProperty>,
    /// [Optional] The maximum number of bad records that BigQuery can ignore when running the job.
    /// If the number of bad records exceeds this value, an invalid error is returned in the
    /// job result. This is only valid for CSV and JSON. The default value is 0, which requires
    /// that all records are valid.
    #[builder(setter(into))]
    #[serde(default)]
    pub max_bad_records: ::std::option::Option<i64>,
    /// Indicates if BigQuery should allow quoted data sections that contain newline characters in
    /// a CSV file. The default value is false.
    #[builder(setter(into))]
    #[serde(default)]
    pub allow_quoted_newlines: bool,
    /// [Required] The destination table to load the data into.
    #[builder(setter(into))]
    #[serde(default)]
    pub destination_table: TableReference,
    /// [Optional] If sourceFormat is set to newline-delimited JSON, indicates whether it should be
    /// processed as a JSON variant such as GeoJSON. For a sourceFormat other than JSON, omit this
    /// field. If the sourceFormat is newline-delimited JSON: - for newline-delimited GeoJSON: set
    /// to GEOJSON.
    #[builder(setter(into))]
    #[serde(default)]
    pub json_extension: ::std::option::Option<::std::string::String>,
    /// [Optional] Specifies a string that represents a null value in a CSV file. For example, if
    /// you specify "\N", BigQuery interprets "\N" as a null value when loading a CSV file. The
    /// default value is the empty string. If you set this property to a custom value, BigQuery
    /// throws an error if an empty string is present for all data types except for STRING and
    /// BYTE. For STRING and BYTE columns, BigQuery interprets the empty string as an empty
    /// value.
    #[builder(setter(into))]
    #[serde(default)]
    pub null_marker: ::std::option::Option<::std::string::String>,
    /// [Optional] Indicates if we should automatically infer the options and schema for CSV and
    /// JSON sources.
    #[builder(setter(into))]
    #[serde(default)]
    pub autodetect: ::std::option::Option<bool>,
    /// [Optional] Defines the list of possible SQL data types to which the source decimal values
    /// are converted. This list and the precision and the scale parameters of the decimal
    /// field determine the target type. In the order of NUMERIC, BIGNUMERIC, and STRING, a
    /// type is picked if it is in the specified list and if it supports the precision and the
    /// scale. STRING supports all precision and scale values. If none of the listed types
    /// supports the precision and the scale, the type supporting the widest range in the
    /// specified list is picked, and if a value exceeds the supported range when reading the
    /// data, an error will be thrown. Example: Suppose the value of this field is ["NUMERIC",
    /// "BIGNUMERIC"]. If (precision,scale) is: (38,9) -> NUMERIC; (39,9) -> BIGNUMERIC
    /// (NUMERIC cannot hold 30 integer digits); (38,10) -> BIGNUMERIC (NUMERIC cannot hold
    /// 10 fractional digits); (76,38) -> BIGNUMERIC; (77,38) -> BIGNUMERIC (error if value exeeds
    /// supported range). This field cannot contain duplicate types. The order of the types in this
    /// field is ignored. For example, ["BIGNUMERIC", "NUMERIC"] is the same as ["NUMERIC",
    /// "BIGNUMERIC"] and NUMERIC always takes precedence over BIGNUMERIC. Defaults to ["NUMERIC",
    /// "STRING"] for ORC and ["NUMERIC"] for the other file formats.
    #[builder(setter(into))]
    #[serde(default)]
    pub decimal_target_types: ::std::vec::Vec<::std::string::String>,
    /// [Optional] Specifies the action that occurs if the destination table already exists. The
    /// following values are supported: WRITE_TRUNCATE: If the table already exists, BigQuery
    /// overwrites the table data. WRITE_APPEND: If the table already exists, BigQuery appends
    /// the data to the table. WRITE_EMPTY: If the table already exists and contains data, a
    /// 'duplicate' error is returned in the job result. The default value is WRITE_APPEND.
    /// Each action is atomic and only occurs if BigQuery is able to complete the job
    /// successfully. Creation, truncation and append actions occur as one atomic update upon
    /// job completion.
    #[builder(setter(into))]
    #[serde(default)]
    pub write_disposition: ::std::option::Option<::std::string::String>,
    /// [Optional] If sourceFormat is set to "AVRO", indicates whether to interpret logical types
    /// as the corresponding BigQuery data type (for example, TIMESTAMP), instead of using the
    /// raw type (for example, INTEGER).
    #[builder(setter(into))]
    #[serde(default)]
    pub use_avro_logical_types: ::std::option::Option<bool>,
    /// [Required] The fully-qualified URIs that point to your data in Google Cloud. For Google
    /// Cloud Storage URIs: Each URI can contain one '*' wildcard character and it must come
    /// after the 'bucket' name. Size limits related to load jobs apply to external data
    /// sources. For Google Cloud Bigtable URIs: Exactly one URI can be specified and it has be
    /// a fully specified and valid HTTPS URL for a Google Cloud Bigtable table. For Google
    /// Cloud Datastore backups: Exactly one URI can be specified. Also, the '*' wildcard
    /// character is not allowed.
    #[builder(setter(into))]
    #[serde(default)]
    pub source_uris: ::std::vec::Vec<::std::string::String>,
    /// [Deprecated] The inline schema. For CSV schemas, specify as "Field1:Type1[,Field2:Type2]*".
    /// For example, "foo:STRING, bar:INTEGER, baz:FLOAT".
    #[builder(setter(into))]
    #[serde(default)]
    pub schema_inline: ::std::string::String,
    /// [Optional] The format of the data files. For CSV files, specify "CSV". For datastore
    /// backups, specify "DATASTORE_BACKUP". For newline-delimited JSON, specify
    /// "NEWLINE_DELIMITED_JSON". For Avro, specify "AVRO". For parquet, specify "PARQUET". For
    /// orc, specify "ORC". The default value is CSV.
    #[builder(setter(into))]
    #[serde(default)]
    pub source_format: ::std::option::Option<::std::string::String>,
    /// Allows the schema of the destination table to be updated as a side effect of the load job
    /// if a schema is autodetected or supplied in the job configuration. Schema update options
    /// are supported in two cases: when writeDisposition is WRITE_APPEND; when
    /// writeDisposition is WRITE_TRUNCATE and the destination table is a partition of a table,
    /// specified by partition decorators. For normal tables, WRITE_TRUNCATE will always
    /// overwrite the schema. One or more of the following values are specified:
    /// ALLOW_FIELD_ADDITION: allow adding a nullable field to the schema.
    /// ALLOW_FIELD_RELAXATION: allow relaxing a required field in the original schema to nullable.
    #[builder(setter(into))]
    #[serde(default)]
    pub schema_update_options: ::std::vec::Vec<::std::string::String>,
    /// [Optional] Preserves the embedded ASCII control characters (the first 32 characters in the
    /// ASCII-table, from '\x00' to '\x1F') when loading from CSV. Only applicable to CSV, ignored
    /// for other formats.
    #[builder(setter(into))]
    #[serde(default)]
    pub preserve_ascii_control_characters: ::std::option::Option<bool>,
    /// [Optional] The character encoding of the data. The supported values are UTF-8 or
    /// ISO-8859-1. The default value is UTF-8. BigQuery decodes the data after the raw, binary
    /// data has been split using the values of the quote and fieldDelimiter properties.
    #[builder(setter(into))]
    #[serde(default)]
    pub encoding: ::std::option::Option<::std::string::String>,
    /// [Beta] [Optional] Properties with which to create the destination table if it is new.
    #[builder(setter(into))]
    pub destination_table_properties: DestinationTableProperties,
    /// [Optional] Specifies whether the job is allowed to create new tables. The following values
    /// are supported: CREATE_IF_NEEDED: If the table does not exist, BigQuery creates the
    /// table. CREATE_NEVER: The table must already exist. If it does not, a 'notFound' error
    /// is returned in the job result. The default value is CREATE_IF_NEEDED. Creation,
    /// truncation and append actions occur as one atomic update upon job completion.
    #[builder(setter(into))]
    #[serde(default)]
    pub create_disposition: ::std::option::Option<::std::string::String>,
    /// [Optional] The separator for fields in a CSV file. The separator can be any ISO-8859-1
    /// single-byte character. To use a character in the range 128-255, you must encode the
    /// character as UTF8. BigQuery converts the string to ISO-8859-1 encoding, and then uses
    /// the first byte of the encoded string to split the data in its raw, binary state.
    /// BigQuery also supports the escape sequence "\t" to specify a tab separator. The default
    /// value is a comma (',').
    #[builder(setter(into))]
    #[serde(default)]
    pub field_delimiter: ::std::option::Option<::std::string::String>,
    /// [Optional] The schema for the destination table. The schema can be omitted if the
    /// destination table already exists, or if you're loading data from Google Cloud
    /// Datastore.
    #[builder(setter(into))]
    #[serde(default)]
    pub schema: TableSchema,
    /// Time-based partitioning specification for the destination table. Only one of
    /// timePartitioning and rangePartitioning should be specified.
    #[builder(setter(into))]
    #[serde(default)]
    pub time_partitioning: TimePartitioning,
    /// [Beta] Clustering specification for the destination table. Must be specified with
    /// time-based partitioning, data in the table will be first partitioned and subsequently
    /// clustered.
    #[builder(setter(into))]
    #[serde(default)]
    pub clustering: Clustering,
    /// [Optional] The number of rows at the top of a CSV file that BigQuery will skip when loading
    /// the data. The default value is 0. This property is useful if you have header rows in
    /// the file that should be skipped.
    #[builder(setter(into))]
    #[serde(default)]
    pub skip_leading_rows: ::std::option::Option<i64>,
    /// User provided referencing file with the expected reader schema, Available for the format:
    /// AVRO, PARQUET, ORC.
    #[builder(setter(into))]
    #[serde(default)]
    pub reference_file_schema_uri: ::std::string::String,
    /// [Optional] Options to configure hive partitioning support.
    #[builder(setter(into))]
    #[serde(default)]
    pub hive_partitioning_options: HivePartitioningOptions,
    /// If sourceFormat is set to "DATASTORE_BACKUP", indicates which entity properties to load
    /// into BigQuery from a Cloud Datastore backup. Property names are case sensitive and must
    /// be top-level properties. If no properties are specified, BigQuery loads all properties.
    /// If any named property isn't found in the Cloud Datastore backup, an invalid error is
    /// returned in the job result.
    #[builder(setter(into))]
    #[serde(default)]
    pub projection_fields: ::std::vec::Vec<::std::string::String>,
}

#[derive(
    ::serde::Deserialize,
    ::serde::Serialize,
    ::typed_builder::TypedBuilder,
    Debug,
    PartialEq,
    Clone,
    Default,
)]
#[serde(rename_all = "camelCase")]
pub struct Clustering {
    /// [Repeated] One or more fields on which data should be clustered. Only top-level,
    /// non-repeated, simple-type fields are supported. When you cluster a table using multiple
    /// columns, the order of columns you specify is important. The order of the specified
    /// columns determines the sort order of the data.
    #[builder(setter(into))]
    #[serde(default)]
    pub fields: ::std::vec::Vec<::std::string::String>,
}

/// Request message for `TestIamPermissions` method.
#[derive(
    ::serde::Deserialize,
    ::serde::Serialize,
    ::typed_builder::TypedBuilder,
    Debug,
    PartialEq,
    Clone,
    Default,
)]
#[serde(rename_all = "camelCase")]
pub struct TestIamPermissionsRequest {
    /// The set of permissions to check for the `resource`. Permissions with wildcards (such as `*`
    /// or `storage.*`) are not allowed. For more information see [IAM
    /// Overview](https://cloud.google.com/iam/docs/overview#permissions).
    #[builder(setter(into))]
    #[serde(default)]
    pub permissions: ::std::vec::Vec<::std::string::String>,
}

#[derive(
    ::serde::Deserialize,
    ::serde::Serialize,
    ::typed_builder::TypedBuilder,
    Debug,
    PartialEq,
    Clone,
    Default,
)]
#[serde(rename_all = "camelCase")]
pub struct IterationResult {
    /// Learn rate used for this iteration.
    #[builder(setter(into))]
    #[serde(default)]
    pub learn_rate: f64,
    /// Loss computed on the eval data at the end of iteration.
    #[builder(setter(into))]
    #[serde(default)]
    pub eval_loss: f64,
    /// Time taken to run the iteration in milliseconds.
    #[builder(setter(into))]
    #[serde(with = "with::int64")]
    #[serde(default)]
    pub duration_ms: i64,
    /// Index of the iteration, 0 based.
    #[builder(setter(into))]
    #[serde(default)]
    pub index: i64,
    /// Loss computed on the training data at the end of iteration.
    #[builder(setter(into))]
    #[serde(default)]
    pub training_loss: f64,
}

#[derive(
    ::serde::Deserialize,
    ::serde::Serialize,
    ::typed_builder::TypedBuilder,
    Debug,
    PartialEq,
    Clone,
    Default,
)]
#[serde(rename_all = "camelCase")]
pub struct JobConfigurationExtract {
    /// A reference to the table being exported.
    #[builder(setter(into))]
    #[serde(default)]
    pub source_table: TableReference,
    /// [Optional] Delimiter to use between fields in the exported data. Default is ','. Not
    /// applicable when extracting models.
    #[builder(setter(into))]
    #[serde(default)]
    pub field_delimiter: ::std::option::Option<::std::string::String>,
    /// A reference to the model being exported.
    #[builder(setter(into))]
    #[serde(default)]
    pub source_model: ModelReference,
    /// [Optional] If destinationFormat is set to "AVRO", this flag indicates whether to enable
    /// extracting applicable column types (such as TIMESTAMP) to their corresponding AVRO logical
    /// types (timestamp-micros), instead of only using their raw types (avro-long). Not
    /// applicable when extracting models.
    #[builder(setter(into))]
    #[serde(default)]
    pub use_avro_logical_types: ::std::option::Option<bool>,
    /// [Pick one] A list of fully-qualified Google Cloud Storage URIs where the extracted table
    /// should be written.
    #[builder(setter(into))]
    #[serde(default)]
    pub destination_uris: ::std::vec::Vec<::std::string::String>,
    /// [Optional] Whether to print out a header row in the results. Default is true. Not
    /// applicable when extracting models.
    #[builder(setter(into))]
    #[serde(default)]
    pub print_header: ::std::option::Option<bool>,
    /// [Optional] The exported file format. Possible values include CSV, NEWLINE_DELIMITED_JSON,
    /// PARQUET or AVRO for tables and ML_TF_SAVED_MODEL or ML_XGBOOST_BOOSTER for models. The
    /// default value for tables is CSV. Tables with nested or repeated fields cannot be
    /// exported as CSV. The default value for models is ML_TF_SAVED_MODEL.
    #[builder(setter(into))]
    #[serde(default)]
    pub destination_format: ::std::option::Option<::std::string::String>,
    /// [Optional] The compression type to use for exported files. Possible values include GZIP,
    /// DEFLATE, SNAPPY, and NONE. The default value is NONE. DEFLATE and SNAPPY are only supported
    /// for Avro. Not applicable when extracting models.
    #[builder(setter(into))]
    #[serde(default)]
    pub compression: ::std::option::Option<::std::string::String>,
    /// [Pick one] DEPRECATED: Use destinationUris instead, passing only one URI as necessary. The
    /// fully-qualified Google Cloud Storage URI where the extracted table should be written.
    #[builder(setter(into))]
    #[serde(default)]
    pub destination_uri: ::std::string::String,
}

#[derive(
    ::serde::Deserialize,
    ::serde::Serialize,
    ::typed_builder::TypedBuilder,
    Debug,
    PartialEq,
    Clone,
    Default,
)]
#[serde(rename_all = "camelCase")]
pub struct BiEngineStatistics {
    /// In case of DISABLED or PARTIAL bi_engine_mode, these contain the explanatory reasons as to
    /// why BI Engine could not accelerate. In case the full query was accelerated, this field
    /// is not populated.
    #[builder(setter(into))]
    #[serde(default)]
    pub bi_engine_reasons: ::std::vec::Vec<BiEngineReason>,
    /// [Output-only] Specifies which mode of BI Engine acceleration was performed (if any).
    #[builder(setter(into))]
    #[serde(default)]
    pub bi_engine_mode: ::std::option::Option<::std::string::String>,
}

#[derive(
    ::serde::Deserialize,
    ::serde::Serialize,
    ::typed_builder::TypedBuilder,
    Debug,
    PartialEq,
    Clone,
    Default,
)]
#[serde(rename_all = "camelCase")]
pub struct RoutineReference {
    /// [Required] The ID of the project containing this routine.
    #[builder(setter(into))]
    #[serde(default)]
    pub project_id: ::std::string::String,
    /// [Required] The ID of the routine. The ID must contain only letters (a-z, A-Z), numbers
    /// (0-9), or underscores (_). The maximum length is 256 characters.
    #[builder(setter(into))]
    #[serde(default)]
    pub routine_id: ::std::string::String,
    /// [Required] The ID of the dataset containing this routine.
    #[builder(setter(into))]
    #[serde(default)]
    pub dataset_id: ::std::string::String,
}

/// Request message for `GetIamPolicy` method.
#[derive(
    ::serde::Deserialize,
    ::serde::Serialize,
    ::typed_builder::TypedBuilder,
    Debug,
    PartialEq,
    Clone,
    Default,
)]
#[serde(rename_all = "camelCase")]
pub struct GetIamPolicyRequest {
    /// OPTIONAL: A `GetPolicyOptions` object for specifying options to `GetIamPolicy`.
    #[builder(setter(into))]
    #[serde(default)]
    pub options: GetPolicyOptions,
}

/// Specifies the audit configuration for a service. The configuration determines which permission
/// types are logged, and what identities, if any, are exempted from logging. An AuditConfig must
/// have one or more AuditLogConfigs. If there are AuditConfigs for both `allServices` and a
/// specific service, the union of the two AuditConfigs is used for that service: the log_types
/// specified in each AuditConfig are enabled, and the exempted_members in each AuditLogConfig are
/// exempted. Example Policy with multiple AuditConfigs: { "audit_configs": [ { "service":
/// "allServices", "audit_log_configs": [ { "log_type": "DATA_READ", "exempted_members": [
/// "user:jose@example.com" ] }, { "log_type": "DATA_WRITE" }, { "log_type": "ADMIN_READ" } ] }, {
/// "service": "sampleservice.googleapis.com", "audit_log_configs": [ { "log_type": "DATA_READ" }, {
/// "log_type": "DATA_WRITE", "exempted_members": [ "user:aliya@example.com" ] } ] } ] } For
/// sampleservice, this policy enables DATA_READ, DATA_WRITE and ADMIN_READ logging. It also exempts
/// `jose@example.com` from DATA_READ logging, and `aliya@example.com` from DATA_WRITE logging.
#[derive(
    ::serde::Deserialize,
    ::serde::Serialize,
    ::typed_builder::TypedBuilder,
    Debug,
    PartialEq,
    Clone,
    Default,
)]
#[serde(rename_all = "camelCase")]
pub struct AuditConfig {
    /// The configuration for logging of each type of permission.
    #[builder(setter(into))]
    #[serde(default)]
    pub audit_log_configs: ::std::vec::Vec<AuditLogConfig>,
    /// Specifies a service that will be enabled for audit logging. For example,
    /// `storage.googleapis.com`, `cloudsql.googleapis.com`. `allServices` is a special value that
    /// covers all services.
    #[builder(setter(into))]
    #[serde(default)]
    pub service: ::std::string::String,
}

/// An array of int.
#[derive(
    ::serde::Deserialize,
    ::serde::Serialize,
    ::typed_builder::TypedBuilder,
    Debug,
    PartialEq,
    Clone,
    Default,
)]
#[serde(rename_all = "camelCase")]
pub struct IntArray {
    /// Elements in the int array.
    #[builder(setter(into))]
    #[serde(default)]
    pub elements: ::std::vec::Vec<i64>,
}

#[derive(
    ::serde::Deserialize,
    ::serde::Serialize,
    ::typed_builder::TypedBuilder,
    Debug,
    PartialEq,
    Clone,
    Default,
)]
#[serde(rename_all = "camelCase")]
pub struct SearchStatistics {
    /// Specifies index usage mode for the query.
    #[builder(setter(into))]
    #[serde(default)]
    pub index_usage_mode: ::std::string::String,
    /// When index_usage_mode is UNUSED or PARTIALLY_USED, this field explains why index was not
    /// used in all or part of the search query. If index_usage_mode is FULLLY_USED, this field
    /// is not populated.
    #[builder(setter(into))]
    #[serde(default)]
    pub index_unused_reason: ::std::vec::Vec<IndexUnusedReason>,
}

#[derive(
    ::serde::Deserialize,
    ::serde::Serialize,
    ::typed_builder::TypedBuilder,
    Debug,
    PartialEq,
    Clone,
    Default,
)]
#[serde(rename_all = "camelCase")]
pub struct RowLevelSecurityStatistics {
    /// [Output-only] [Preview] Whether any accessed data was protected by row access policies.
    #[builder(setter(into))]
    #[serde(default)]
    pub row_level_security_applied: ::std::option::Option<bool>,
}

#[derive(
    ::serde::Deserialize,
    ::serde::Serialize,
    ::typed_builder::TypedBuilder,
    Debug,
    PartialEq,
    Clone,
    Default,
)]
#[serde(rename_all = "camelCase")]
pub struct ViewDefinition {
    /// Specifies whether to use BigQuery's legacy SQL for this view. The default value is true. If
    /// set to false, the view will use BigQuery's standard SQL:
    /// https://cloud.google.com/bigquery/sql-reference/ Queries and views that reference this view must
    /// use the same flag value.
    #[builder(setter(into))]
    #[serde(default)]
    pub use_legacy_sql: bool,
    /// [Required] A query that BigQuery executes when the view is referenced.
    #[builder(setter(into))]
    #[serde(default)]
    pub query: ::std::string::String,
    /// True if the column names are explicitly specified. For example by using the 'CREATE VIEW
    /// v(c1, c2) AS ...' syntax. Can only be set using BigQuery's standard SQL:
    /// https://cloud.google.com/bigquery/sql-reference/
    #[builder(setter(into))]
    #[serde(default)]
    pub use_explicit_column_names: bool,
    /// Describes user-defined function resources used in the query.
    #[builder(setter(into))]
    #[serde(default)]
    pub user_defined_function_resources: ::std::vec::Vec<UserDefinedFunctionResource>,
}

/// Output only. Type of the model resource.
#[derive(::serde::Deserialize, ::serde::Serialize, Debug, PartialEq, Clone)]
pub enum ModelType {
    /// Linear regression model.
    #[serde(rename = "LINEAR_REGRESSION")]
    LinearRegression,
    /// Logistic regression based classification model.
    #[serde(rename = "LOGISTIC_REGRESSION")]
    LogisticRegression,
    /// K-means clustering model.
    #[serde(rename = "KMEANS")]
    Kmeans,
    /// Matrix factorization model.
    #[serde(rename = "MATRIX_FACTORIZATION")]
    MatrixFactorization,
    /// DNN classifier model.
    #[serde(rename = "DNN_CLASSIFIER")]
    DnnClassifier,
    /// An imported TensorFlow model.
    #[serde(rename = "TENSORFLOW")]
    Tensorflow,
    /// DNN regressor model.
    #[serde(rename = "DNN_REGRESSOR")]
    DnnRegressor,
    /// Boosted tree regressor model.
    #[serde(rename = "BOOSTED_TREE_REGRESSOR")]
    BoostedTreeRegressor,
    /// Boosted tree classifier model.
    #[serde(rename = "BOOSTED_TREE_CLASSIFIER")]
    BoostedTreeClassifier,
    /// ARIMA model.
    #[serde(rename = "ARIMA")]
    Arima,
    /// AutoML Tables regression model.
    #[serde(rename = "AUTOML_REGRESSOR")]
    AutomlRegressor,
    /// AutoML Tables classification model.
    #[serde(rename = "AUTOML_CLASSIFIER")]
    AutomlClassifier,
    /// Prinpical Component Analysis model.
    #[serde(rename = "PCA")]
    Pca,
    /// Wide-and-deep classifier model.
    #[serde(rename = "DNN_LINEAR_COMBINED_CLASSIFIER")]
    DnnLinearCombinedClassifier,
    /// Wide-and-deep regressor model.
    #[serde(rename = "DNN_LINEAR_COMBINED_REGRESSOR")]
    DnnLinearCombinedRegressor,
    /// Autoencoder model.
    #[serde(rename = "AUTOENCODER")]
    Autoencoder,
    /// New name for the ARIMA model.
    #[serde(rename = "ARIMA_PLUS")]
    ArimaPlus,
}

#[derive(
    ::serde::Deserialize, ::serde::Serialize, ::typed_builder::TypedBuilder, Debug, PartialEq, Clone,
)]
#[serde(rename_all = "camelCase")]
pub struct Model {
    /// Output only. A hash of this resource.
    #[builder(setter(into))]
    #[serde(default)]
    pub etag: ::std::string::String,
    /// Output only. For single-objective [hyperparameter
    /// tuning](/bigquery-ml/docs/reference/standard-sql/bigqueryml-syntax-hp-tuning-overview)
    /// models, it only contains the best trial. For multi-objective [hyperparameter
    /// tuning](/bigquery-ml/docs/reference/standard-sql/bigqueryml-syntax-hp-tuning-overview)
    /// models, it contains all Pareto optimal trials sorted by trial_id.
    #[builder(setter(into))]
    #[serde(default)]
    pub optimal_trial_ids: ::std::vec::Vec<i64>,
    /// Output only. Input feature columns that were used to train this model.
    #[builder(setter(into))]
    #[serde(default)]
    pub feature_columns: ::std::vec::Vec<StandardSqlField>,
    /// Optional. A descriptive name for this model.
    #[builder(setter(into))]
    #[serde(default)]
    pub friendly_name: ::std::option::Option<::std::string::String>,
    /// Output only. The time when this model was created, in millisecs since the epoch.
    #[builder(setter(into))]
    #[serde(with = "with::int64")]
    #[serde(default)]
    pub creation_time: i64,
    /// The best trial_id across all training runs.
    #[builder(setter(into))]
    #[serde(with = "with::int64")]
    #[serde(default)]
    pub best_trial_id: i64,
    /// Output only. The geographic location where the model resides. This value is inherited from
    /// the dataset.
    #[builder(setter(into))]
    #[serde(default)]
    pub location: ::std::string::String,
    /// Optional. The time when this model expires, in milliseconds since the epoch. If not
    /// present, the model will persist indefinitely. Expired models will be deleted and their
    /// storage reclaimed. The defaultTableExpirationMs property of the encapsulating dataset
    /// can be used to set a default expirationTime on newly created models.
    #[builder(setter(into))]
    #[serde(with = "with::int64::option")]
    #[serde(default)]
    pub expiration_time: ::std::option::Option<i64>,
    /// Output only. The time when this model was last modified, in millisecs since the epoch.
    #[builder(setter(into))]
    #[serde(with = "with::int64")]
    #[serde(default)]
    pub last_modified_time: i64,
    /// Custom encryption configuration (e.g., Cloud KMS keys). This shows the encryption
    /// configuration of the model data while stored in BigQuery storage. This field can be
    /// used with PatchModel to update encryption key for an already encrypted model.
    #[builder(setter(into))]
    #[serde(default)]
    pub encryption_configuration: EncryptionConfiguration,
    /// Output only. Label columns that were used to train this model. The output of the model will
    /// have a "predicted_" prefix to these columns.
    #[builder(setter(into))]
    #[serde(default)]
    pub label_columns: ::std::vec::Vec<StandardSqlField>,
    /// Optional. A user-friendly description of this model.
    #[builder(setter(into))]
    #[serde(default)]
    pub description: ::std::option::Option<::std::string::String>,
    /// Required. Unique identifier for this model.
    #[builder(setter(into))]
    #[serde(default)]
    pub model_reference: ModelReference,
    /// Output only. Trials of a [hyperparameter
    /// tuning](/bigquery-ml/docs/reference/standard-sql/bigqueryml-syntax-hp-tuning-overview)
    /// model sorted by trial_id.
    #[builder(setter(into))]
    #[serde(default)]
    pub hparam_trials: ::std::vec::Vec<HparamTuningTrial>,
    /// The labels associated with this model. You can use these to organize and group your models.
    /// Label keys and values can be no longer than 63 characters, can only contain lowercase
    /// letters, numeric characters, underscores and dashes. International characters are
    /// allowed. Label values are optional. Label keys must start with a letter and each label
    /// in the list must have a different key.
    #[builder(setter(into))]
    #[serde(default)]
    pub labels: Labels,
    /// Output only. The default trial_id to use in TVFs when the trial_id is not passed in. For
    /// single-objective [hyperparameter
    /// tuning](/bigquery-ml/docs/reference/standard-sql/bigqueryml-syntax-hp-tuning-overview)
    /// models, this is the best trial ID. For multi-objective [hyperparameter
    /// tuning](/bigquery-ml/docs/reference/standard-sql/bigqueryml-syntax-hp-tuning-overview)
    /// models, this is the smallest trial ID among all Pareto optimal trials.
    #[builder(setter(into))]
    #[serde(with = "with::int64")]
    #[serde(default)]
    pub default_trial_id: i64,
    /// Output only. Type of the model resource.
    #[builder(setter(into))]
    pub model_type: ModelType,
    /// Information for all training runs in increasing order of start_time.
    #[builder(setter(into))]
    #[serde(default)]
    pub training_runs: ::std::vec::Vec<TrainingRun>,
    /// Output only. All hyperparameter search spaces in this model.
    #[builder(setter(into))]
    pub hparam_search_spaces: HparamSearchSpaces,
}

#[derive(
    ::serde::Deserialize,
    ::serde::Serialize,
    ::typed_builder::TypedBuilder,
    Debug,
    PartialEq,
    Clone,
    Default,
)]
#[serde(rename_all = "camelCase")]
pub struct ListModelsResponse {
    /// Models in the requested dataset. Only the following fields are populated: model_reference,
    /// model_type, creation_time, last_modified_time and labels.
    #[builder(setter(into))]
    #[serde(default)]
    pub models: ::std::vec::Vec<Model>,
    /// A token to request the next page of results.
    #[builder(setter(into))]
    #[serde(default)]
    pub next_page_token: ::std::string::String,
}

#[derive(
    ::serde::Deserialize, ::serde::Serialize, ::typed_builder::TypedBuilder, Debug, PartialEq, Clone,
)]
#[serde(rename_all = "camelCase")]
pub struct Jobs {
    /// Unique opaque ID of the job.
    #[builder(setter(into))]
    #[serde(default)]
    pub id: ::std::string::String,
    /// Running state of the job. When the state is DONE, errorResult can be checked to determine
    /// whether the job succeeded or failed.
    #[builder(setter(into))]
    #[serde(default)]
    pub state: ::std::string::String,
    /// [Full-projection-only] Email address of the user who ran the job.
    #[builder(setter(into))]
    #[serde(default)]
    pub user_email: ::std::string::String,
    /// [Output-only] Information about the job, including starting time and ending time of the
    /// job.
    #[builder(setter(into))]
    pub statistics: JobStatistics,
    /// The resource type.
    #[builder(setter(into))]
    #[serde(default)]
    pub kind: ::std::string::String,
    /// A result object that will be present only if the job has failed.
    #[builder(setter(into))]
    #[serde(default)]
    pub error_result: ErrorProto,
    /// Job reference uniquely identifying the job.
    #[builder(setter(into))]
    #[serde(default)]
    pub job_reference: JobReference,
    /// [Full-projection-only] Specifies the job configuration.
    #[builder(setter(into))]
    pub configuration: JobConfiguration,
    /// [Full-projection-only] Describes the state of the job.
    #[builder(setter(into))]
    #[serde(default)]
    pub status: JobStatus,
}

#[derive(
    ::serde::Deserialize,
    ::serde::Serialize,
    ::typed_builder::TypedBuilder,
    Debug,
    PartialEq,
    Clone,
    Default,
)]
#[serde(rename_all = "camelCase")]
pub struct JobList {
    /// A token to request the next page of results.
    #[builder(setter(into))]
    #[serde(default)]
    pub next_page_token: ::std::string::String,
    /// List of jobs that were requested.
    #[builder(setter(into))]
    #[serde(default)]
    pub jobs: ::std::vec::Vec<Jobs>,
    /// The resource type of the response.
    #[builder(setter(into))]
    #[serde(default)]
    pub kind: ::std::string::String,
    /// A hash of this page of results.
    #[builder(setter(into))]
    #[serde(default)]
    pub etag: ::std::string::String,
}

#[derive(
    ::serde::Deserialize,
    ::serde::Serialize,
    ::typed_builder::TypedBuilder,
    Debug,
    PartialEq,
    Clone,
    Default,
)]
#[serde(rename_all = "camelCase")]
pub struct DataMaskingStatistics {
    /// [Output-only] [Preview] Whether any accessed data was protected by data masking. The actual
    /// evaluation is done by accessStats.masked_field_count > 0. Since this is only used for the
    /// discovery_doc generation purpose, as long as the type (boolean) matches, client library can
    /// leverage this. The actual evaluation of the variable is done else-where.
    #[builder(setter(into))]
    #[serde(default)]
    pub data_masking_applied: ::std::option::Option<bool>,
}

/// Arima order, can be used for both non-seasonal and seasonal parts.
#[derive(
    ::serde::Deserialize,
    ::serde::Serialize,
    ::typed_builder::TypedBuilder,
    Debug,
    PartialEq,
    Clone,
    Default,
)]
#[serde(rename_all = "camelCase")]
pub struct ArimaOrder {
    /// Order of the differencing part.
    #[builder(setter(into))]
    #[serde(with = "with::int64")]
    #[serde(default)]
    pub d: i64,
    /// Order of the autoregressive part.
    #[builder(setter(into))]
    #[serde(with = "with::int64")]
    #[serde(default)]
    pub p: i64,
    /// Order of the moving-average part.
    #[builder(setter(into))]
    #[serde(with = "with::int64")]
    #[serde(default)]
    pub q: i64,
}

#[derive(
    ::serde::Deserialize,
    ::serde::Serialize,
    ::typed_builder::TypedBuilder,
    Debug,
    PartialEq,
    Clone,
    Default,
)]
#[serde(rename_all = "camelCase")]
pub struct TableRow {
    /// Represents a single row in the result set, consisting of one or more fields.
    #[builder(setter(into))]
    #[serde(default)]
    pub f: ::std::vec::Vec<TableCell>,
}

/// Range of an int hyperparameter.
#[derive(
    ::serde::Deserialize,
    ::serde::Serialize,
    ::typed_builder::TypedBuilder,
    Debug,
    PartialEq,
    Clone,
    Default,
)]
#[serde(rename_all = "camelCase")]
pub struct IntRange {
    /// Min value of the int parameter.
    #[builder(setter(into))]
    #[serde(with = "with::int64")]
    #[serde(default)]
    pub min: i64,
    /// Max value of the int parameter.
    #[builder(setter(into))]
    #[serde(with = "with::int64")]
    #[serde(default)]
    pub max: i64,
}

#[derive(
    ::serde::Deserialize, ::serde::Serialize, ::typed_builder::TypedBuilder, Debug, PartialEq, Clone,
)]
#[serde(rename_all = "camelCase")]
pub struct JobCancelResponse {
    /// The resource type of the response.
    #[builder(setter(into))]
    #[serde(default)]
    pub kind: ::std::string::String,
    /// The final state of the job.
    #[builder(setter(into))]
    pub job: Job,
}

/// Search space for an int hyperparameter.
#[derive(
    ::serde::Deserialize,
    ::serde::Serialize,
    ::typed_builder::TypedBuilder,
    Debug,
    PartialEq,
    Clone,
    Default,
)]
#[serde(rename_all = "camelCase")]
pub struct IntHparamSearchSpace {
    /// Candidates of the int hyperparameter.
    #[builder(setter(into))]
    #[serde(default)]
    pub candidates: IntCandidates,
    /// Range of the int hyperparameter.
    #[builder(setter(into))]
    #[serde(default)]
    pub range: IntRange,
}

#[derive(
    ::serde::Deserialize,
    ::serde::Serialize,
    ::typed_builder::TypedBuilder,
    Debug,
    PartialEq,
    Clone,
    Default,
)]
#[serde(rename_all = "camelCase")]
pub struct SparkLoggingInfo {
    /// [Output-only] Resource type used for logging
    #[builder(setter(into))]
    #[serde(default)]
    pub resource_type: ::std::option::Option<::std::string::String>,
    /// [Output-only] Project ID used for logging
    #[builder(setter(into))]
    #[serde(default)]
    pub project_id: ::std::option::Option<::std::string::String>,
}

/// Represents the count of a single category within the cluster.
#[derive(
    ::serde::Deserialize,
    ::serde::Serialize,
    ::typed_builder::TypedBuilder,
    Debug,
    PartialEq,
    Clone,
    Default,
)]
#[serde(rename_all = "camelCase")]
pub struct CategoryCount {
    /// The name of category.
    #[builder(setter(into))]
    #[serde(default)]
    pub category: ::std::string::String,
    /// The count of training samples matching the category within the cluster.
    #[builder(setter(into))]
    #[serde(with = "with::int64")]
    #[serde(default)]
    pub count: i64,
}

#[derive(
    ::serde::Deserialize,
    ::serde::Serialize,
    ::typed_builder::TypedBuilder,
    Debug,
    PartialEq,
    Clone,
    Default,
)]
#[serde(rename_all = "camelCase")]
pub struct TableReference {
    /// [Required] The ID of the project containing this table.
    #[builder(setter(into))]
    #[serde(default)]
    pub project_id: ::std::string::String,
    /// [Required] The ID of the table. The ID must contain only letters (a-z, A-Z), numbers (0-9),
    /// or underscores (_). The maximum length is 1,024 characters.
    #[builder(setter(into))]
    #[serde(default)]
    pub table_id: ::std::string::String,
    /// [Required] The ID of the dataset containing this table.
    #[builder(setter(into))]
    #[serde(default)]
    pub dataset_id: ::std::string::String,
}

#[derive(
    ::serde::Deserialize,
    ::serde::Serialize,
    ::typed_builder::TypedBuilder,
    Debug,
    PartialEq,
    Clone,
    Default,
)]
#[serde(rename_all = "camelCase")]
pub struct IndexUnusedReason {
    /// [Output-only] Specifies the high-level reason for the scenario when no search index was
    /// used.
    #[builder(setter(into))]
    #[serde(default)]
    pub code: ::std::option::Option<::std::string::String>,
    /// [Output-only] Free form human-readable reason for the scenario when no search index was
    /// used.
    #[builder(setter(into))]
    #[serde(default)]
    pub message: ::std::option::Option<::std::string::String>,
    /// [Output-only] Specifies the name of the unused search index, if available.
    #[builder(setter(into))]
    #[serde(default)]
    pub index_name: ::std::option::Option<::std::string::String>,
    /// [Output-only] Specifies the base table involved in the reason that no search index was
    /// used.
    #[builder(setter(into))]
    #[serde(default)]
    pub base_table: TableReference,
}

#[derive(
    ::serde::Deserialize,
    ::serde::Serialize,
    ::typed_builder::TypedBuilder,
    Debug,
    PartialEq,
    Clone,
    Default,
)]
#[serde(rename_all = "camelCase")]
pub struct MlStatistics {
    /// Maximum number of iterations specified as max_iterations in the 'CREATE MODEL' query. The
    /// actual number of iterations may be less than this number due to early stop.
    #[builder(setter(into))]
    #[serde(with = "with::int64")]
    #[serde(default)]
    pub max_iterations: i64,
    /// Results for all completed iterations.
    #[builder(setter(into))]
    #[serde(default)]
    pub iteration_results: ::std::vec::Vec<IterationResult>,
}

#[derive(
    ::serde::Deserialize,
    ::serde::Serialize,
    ::typed_builder::TypedBuilder,
    Debug,
    PartialEq,
    Clone,
    Default,
)]
#[serde(rename_all = "camelCase")]
pub struct ScriptStackFrame {
    /// [Output-only] One-based start line.
    #[builder(setter(into))]
    #[serde(default)]
    pub start_line: ::std::option::Option<i64>,
    /// [Output-only] One-based end line.
    #[builder(setter(into))]
    #[serde(default)]
    pub end_line: ::std::option::Option<i64>,
    /// [Output-only] Text of the current statement/expression.
    #[builder(setter(into))]
    #[serde(default)]
    pub text: ::std::option::Option<::std::string::String>,
    /// [Output-only] One-based end column.
    #[builder(setter(into))]
    #[serde(default)]
    pub end_column: ::std::option::Option<i64>,
    /// [Output-only] One-based start column.
    #[builder(setter(into))]
    #[serde(default)]
    pub start_column: ::std::option::Option<i64>,
    /// [Output-only] Name of the active procedure, empty if in a top-level script.
    #[builder(setter(into))]
    #[serde(default)]
    pub procedure_id: ::std::option::Option<::std::string::String>,
}

/// Global explanations containing the top most important features after training.
#[derive(
    ::serde::Deserialize,
    ::serde::Serialize,
    ::typed_builder::TypedBuilder,
    Debug,
    PartialEq,
    Clone,
    Default,
)]
#[serde(rename_all = "camelCase")]
pub struct GlobalExplanation {
    /// A list of the top global explanations. Sorted by absolute value of attribution in
    /// descending order.
    #[builder(setter(into))]
    #[serde(default)]
    pub explanations: ::std::vec::Vec<Explanation>,
    /// Class label for this set of global explanations. Will be empty/null for binary logistic and
    /// linear regression models. Sorted alphabetically in descending order.
    #[builder(setter(into))]
    #[serde(default)]
    pub class_label: ::std::string::String,
}

#[derive(
    ::serde::Deserialize,
    ::serde::Serialize,
    ::typed_builder::TypedBuilder,
    Debug,
    PartialEq,
    Clone,
    Default,
)]
#[serde(rename_all = "camelCase")]
pub struct RowAccessPolicyReference {
    /// [Required] The ID of the table containing this row access policy.
    #[builder(setter(into))]
    #[serde(default)]
    pub table_id: ::std::string::String,
    /// [Required] The ID of the dataset containing this row access policy.
    #[builder(setter(into))]
    #[serde(default)]
    pub dataset_id: ::std::string::String,
    /// [Required] The ID of the project containing this row access policy.
    #[builder(setter(into))]
    #[serde(default)]
    pub project_id: ::std::string::String,
    /// [Required] The ID of the row access policy. The ID must contain only letters (a-z, A-Z),
    /// numbers (0-9), or underscores (_). The maximum length is 256 characters.
    #[builder(setter(into))]
    #[serde(default)]
    pub policy_id: ::std::string::String,
}

#[derive(
    ::serde::Deserialize,
    ::serde::Serialize,
    ::typed_builder::TypedBuilder,
    Debug,
    PartialEq,
    Clone,
    Default,
)]
#[serde(rename_all = "camelCase")]
pub struct DatasetReference {
    /// [Optional] The ID of the project containing this dataset.
    #[builder(setter(into))]
    #[serde(default)]
    pub project_id: ::std::option::Option<::std::string::String>,
    /// [Required] A unique ID for this dataset, without the project name. The ID must contain only
    /// letters (a-z, A-Z), numbers (0-9), or underscores (_). The maximum length is 1,024
    /// characters.
    #[builder(setter(into))]
    #[serde(default)]
    pub dataset_id: ::std::string::String,
}

pub mod with {
    use ::serde::de::{self, Expected, Unexpected};
    use ::std::fmt;

    macro_rules! int_try_from_fns {
        (
            $($fn_name:ident($arg_ty:ty)),*
            $(,)?
        ) => {
            $(
                fn $fn_name<E>(self, value: $arg_ty) -> Result<Self::Value, E>
                where
                    E: serde::de::Error,
                {
                    match TryFrom::try_from(value) {
                        Ok(converted) => Ok(converted),
                        Err(err) => Err(serde::de::Error::invalid_value(super::UnexpectedHelper::from(value).0, &super::ExpectingWrapper(&err))),
                    }
                }
            )*
        };
    }

    macro_rules! int_to_float_cast_fns {
        (
            $($fn_name:ident($arg_ty:ty)),*
            $(,)?
        ) => {
            $(
                fn $fn_name<E>(self, value: $arg_ty) -> Result<Self::Value, E>
                where
                    E: serde::de::Error,
                {
                    Ok(value as f64)
                }
            )*
        };
    }

    struct ExpectingWrapper<'a, T>(&'a T);

    impl<T: fmt::Display> Expected for ExpectingWrapper<'_, T> {
        fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            fmt::Display::fmt(&self.0, formatter)
        }
    }

    // helper type to provide From impls for the types it can contain
    struct UnexpectedHelper<'a>(Unexpected<'a>);

    macro_rules! impl_unexpected_helper_from {
        ($($from_ty:ty: $arg:ident => $blk:expr),* $(,)?) => {
            $(
                impl<'a> From<$from_ty> for UnexpectedHelper<'a> {
                    fn from($arg: $from_ty) -> Self {
                        Self($blk)
                    }
                }
            )*
        };
    }

    impl_unexpected_helper_from! {
        i8: int => Unexpected::Signed(int as i64),
        i16: int => Unexpected::Signed(int as i64),
        i32: int => Unexpected::Signed(int as i64),
        i64: int => Unexpected::Signed(int as i64),
        i128: int => Unexpected::Signed(int as i64),
        u8: uint => Unexpected::Unsigned(uint as u64),
        u16: uint => Unexpected::Unsigned(uint as u64),
        u32: uint => Unexpected::Unsigned(uint as u64),
        u64: uint => Unexpected::Unsigned(uint as u64),
        u128: uint => Unexpected::Unsigned(uint as u64),
        f32: double => Unexpected::Float(double as f64),
        f64: double => Unexpected::Float(double as f64),
    }

    struct OptionalVisitor<V>(V);

    impl<'de, V> de::Visitor<'de> for OptionalVisitor<V>
    where
        V: de::Visitor<'de>,
    {
        type Value = Option<V::Value>;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            self.0.expecting(formatter)?;
            formatter.write_str(" (optional)")
        }

        fn visit_none<E>(self) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            Ok(None)
        }

        fn visit_unit<E>(self) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            Ok(None)
        }

        fn visit_some<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
        where
            D: serde::Deserializer<'de>,
        {
            deserializer.deserialize_any(self.0).map(Some)
        }

        fn visit_newtype_struct<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
        where
            D: serde::Deserializer<'de>,
        {
            deserializer.deserialize_any(self.0).map(Some)
        }
    }

    macro_rules! float_noop {
        ($num_ty:ty; $visitor:expr, $value:expr) => {{ Ok($value as $num_ty) }};
    }

    macro_rules! float_to_int {
        ($num_ty:ty; $visitor:expr, $value:expr) => {{
            const MIN: f64 = <$num_ty>::MIN as f64;
            const MAX: f64 = <$num_ty>::MAX as f64;

            let rounded = $value.round();
            if (MIN..=MAX).contains(&rounded) {
                Ok(rounded as $num_ty)
            } else {
                Err(de::Error::invalid_value(
                    super::UnexpectedHelper::from($value).0,
                    &$visitor,
                ))
            }
        }};
    }

    macro_rules! impl_mod {
        ($($num_ty:ty => { $mod_name:ident, $signed_unsigned_str:literal, $int_macro:ident, $float_macro:ident }),* $(,)?) => {
            $(
                pub mod $mod_name {
                    use serde::{de, Deserializer, Serializer};
                    use std::fmt;

                    #[allow(dead_code)]
                    pub fn serialize<S>(int: &$num_ty, serializer: S) -> Result<S::Ok, S::Error>
                    where
                        S: Serializer,
                    {
                        serializer.collect_str(int)
                    }

                    #[allow(dead_code)]
                    pub fn deserialize<'de, D>(deserializer: D) -> Result<$num_ty, D::Error>
                    where
                        D: Deserializer<'de>,
                    {
                        deserializer.deserialize_str(Visitor)
                    }

                    struct Visitor;

                    impl<'de> de::Visitor<'de> for Visitor {
                        type Value = $num_ty;

                        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                            formatter.write_str(
                                concat!("a ", $signed_unsigned_str, ", either as a number or string")
                            )
                        }

                        fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
                        where
                            E: de::Error,
                        {
                            match v.trim().parse::<$num_ty>() {
                                Ok(v) => Ok(v),
                                Err(_) => Err(de::Error::invalid_value(de::Unexpected::Str(v), &self)),
                            }
                        }

                        fn visit_f64<E>(self, v: f64) -> Result<Self::Value, E>
                        where
                            E: de::Error
                        {
                            $float_macro!($num_ty; self, v)
                        }

                        $int_macro! {
                            visit_i8(i8),
                            visit_i16(i16),
                            visit_i32(i32),
                            visit_i64(i64),
                            visit_i128(i128),
                            visit_u8(u8),
                            visit_u16(u16),
                            visit_u32(u32),
                            visit_u64(u64),
                            visit_u128(u128),
                        }
                    }

                    pub mod option {
                        use super::super::OptionalVisitor;
                        use super::*;

                        #[allow(dead_code)]
                        pub fn serialize<S>(int: &Option<$num_ty>, serializer: S) -> Result<S::Ok, S::Error>
                        where
                            S: Serializer,
                        {
                            match int {
                                Some(int) => serializer.collect_str(int),
                                None => serializer.serialize_none(),
                            }
                        }

                        #[allow(dead_code)]
                        pub fn deserialize<'de, D>(deserializer: D) -> Result<Option<$num_ty>, D::Error>
                        where
                            D: Deserializer<'de>,
                        {
                            deserializer.deserialize_option(OptionalVisitor(Visitor))
                        }
                    }
                }

            )*
        };
    }

    impl_mod! {
        f64 => { double, "double", int_to_float_cast_fns, float_noop },
        i64 => { int64, "signed integer (up to 64 bit)", int_try_from_fns, float_to_int },
        u64 => { uint64, "unsigned integer (up to 64 bit)", int_try_from_fns, float_to_int },
        i32 => { int32, "signed integer", int_try_from_fns, float_to_int },
        u32 => { uint32, "unsigned integer", int_try_from_fns, float_to_int },
    }
}
