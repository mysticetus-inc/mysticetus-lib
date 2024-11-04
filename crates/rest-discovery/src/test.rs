// @generated
//! A data platform for customers to create, manage, share and query data.

#[doc=" The Base URL for this service."]
pub const BASE_URL: &str = "https://bigquery.googleapis.com/bigquery/v2/";

/// Aggregate metrics for classification/classifier models. For multi-class models, the metrics are
/// either macro-averaged or micro-averaged. When macro-averaged, the metrics are calculated for
/// each label and then an unweighted average is taken of those values. When micro-averaged, the
/// metric is calculated globally by counting the total number of correctly predicted rows.
#[derive(::serde::Deserialize, ::serde::Serialize, ::typed_builder::TypedBuilder, Debug, PartialEq, Clone)]
#[derive(Default)]
#[serde(rename_all = "camelCase")]
pub struct AggregateClassificationMetrics {
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
    /// Logarithmic Loss. For multiclass this is a macro-averaged metric.
    #[builder(setter(into))]
    #[serde(default)]
    pub log_loss: f64,
    /// Precision is the fraction of actual positive predictions that had positive actual labels. For
    /// multiclass this is a macro-averaged metric treating each class as a binary classifier.
    #[builder(setter(into))]
    #[serde(default)]
    pub precision: f64,
    /// Recall is the fraction of actual positive labels that were given a positive prediction. For
    /// multiclass this is a macro-averaged metric.
    #[builder(setter(into))]
    #[serde(default)]
    pub recall: f64,
    /// Area Under a ROC Curve. For multiclass this is a macro-averaged metric.
    #[builder(setter(into))]
    #[serde(default)]
    pub roc_auc: f64,
    /// Threshold at which the metrics are computed. For binary classification models this is the
    /// positive class threshold. For multi-class classfication models this is the confidence threshold.
    #[builder(setter(into))]
    #[serde(default)]
    pub threshold: f64,
}

/// Represents privacy policy associated with "aggregation threshold" method.
#[derive(::serde::Deserialize, ::serde::Serialize, ::typed_builder::TypedBuilder, Debug, PartialEq, Clone)]
#[derive(Default)]
#[serde(rename_all = "camelCase")]
pub struct AggregationThresholdPolicy {
    /// Optional. The privacy unit column(s) associated with this policy. For now, only one column per
    /// data source object (table, view) is allowed as a privacy unit column. Representing as a repeated
    /// field in metadata for extensibility to multiple columns in future. Duplicates and Repeated
    /// struct fields are not allowed. For nested fields, use dot notation ("outer.inner")
    #[builder(setter(into))]
    #[serde(default)]
    pub privacy_unit_columns: ::std::vec::Vec<::std::string::String>,
    /// Optional. The threshold for the "aggregation threshold" policy.
    #[builder(setter(into))]
    #[serde(with = "with::int64::option")]
    #[serde(default)]
    pub threshold: ::std::option::Option<i64>,
}

/// Optional. Defaults to FIXED_TYPE.
#[derive(::serde::Deserialize, ::serde::Serialize, Debug, PartialEq, Clone)]
pub enum ArgumentKind {
        /// The argument is a variable with fully specified type, which can be a struct or an array, but
        /// not a table.
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
#[derive(::serde::Deserialize, ::serde::Serialize, ::typed_builder::TypedBuilder, Debug, PartialEq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Argument {
    /// Optional. Defaults to FIXED_TYPE.
    #[builder(setter(into))]
    pub argument_kind: ArgumentKind,
    /// Required unless argument_kind = ANY_TYPE.
    #[builder(setter(into))]
    pub data_type: StandardSqlDataType,
    /// Optional. Whether the argument is an aggregate function parameter. Must be Unset for routine
    /// types other than AGGREGATE_FUNCTION. For AGGREGATE_FUNCTION, if set to false, it is equivalent
    /// to adding "NOT AGGREGATE" clause in DDL; Otherwise, it is equivalent to omitting "NOT AGGREGATE"
    /// clause in DDL.
    #[builder(setter(into))]
    #[serde(default)]
    pub is_aggregate: ::std::option::Option<bool>,
    /// Optional. Specifies whether the argument is input or output. Can be set for procedures only.
    #[builder(setter(into))]
    pub mode: Mode,
    /// Optional. The name of this argument. Can be absent for function return argument.
    #[builder(setter(into))]
    #[serde(default)]
    pub name: ::std::option::Option<::std::string::String>,
}

/// Arima coefficients.
#[derive(::serde::Deserialize, ::serde::Serialize, ::typed_builder::TypedBuilder, Debug, PartialEq, Clone)]
#[derive(Default)]
#[serde(rename_all = "camelCase")]
pub struct ArimaCoefficients {
    /// Auto-regressive coefficients, an array of double.
    #[builder(setter(into))]
    #[serde(default)]
    pub auto_regressive_coefficients: ::std::vec::Vec<f64>,
    /// Intercept coefficient, just a double not an array.
    #[builder(setter(into))]
    #[serde(default)]
    pub intercept_coefficient: f64,
    /// Moving-average coefficients, an array of double.
    #[builder(setter(into))]
    #[serde(default)]
    pub moving_average_coefficients: ::std::vec::Vec<f64>,
}

/// ARIMA model fitting metrics.
#[derive(::serde::Deserialize, ::serde::Serialize, ::typed_builder::TypedBuilder, Debug, PartialEq, Clone)]
#[derive(Default)]
#[serde(rename_all = "camelCase")]
pub struct ArimaFittingMetrics {
    /// AIC.
    #[builder(setter(into))]
    #[serde(default)]
    pub aic: f64,
    /// Log-likelihood.
    #[builder(setter(into))]
    #[serde(default)]
    pub log_likelihood: f64,
    /// Variance.
    #[builder(setter(into))]
    #[serde(default)]
    pub variance: f64,
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

/// Model evaluation metrics for ARIMA forecasting models.
#[derive(::serde::Deserialize, ::serde::Serialize, ::typed_builder::TypedBuilder, Debug, PartialEq, Clone)]
#[derive(Default)]
#[serde(rename_all = "camelCase")]
pub struct ArimaForecastingMetrics {
    /// Arima model fitting metrics.
    #[builder(setter(into))]
    #[serde(default)]
    pub arima_fitting_metrics: ::std::vec::Vec<ArimaFittingMetrics>,
    /// Repeated as there can be many metric sets (one for each model) in auto-arima and the large-scale
    /// case.
    #[builder(setter(into))]
    #[serde(default)]
    pub arima_single_model_forecasting_metrics: ::std::vec::Vec<ArimaSingleModelForecastingMetrics>,
    /// Whether Arima model fitted with drift or not. It is always false when d is not 1.
    #[builder(setter(into))]
    #[serde(default)]
    pub has_drift: ::std::vec::Vec<bool>,
    /// Non-seasonal order.
    #[builder(setter(into))]
    #[serde(default)]
    pub non_seasonal_order: ::std::vec::Vec<ArimaOrder>,
    /// Seasonal periods. Repeated because multiple periods are supported for one time series.
    #[builder(setter(into))]
    #[serde(default)]
    pub seasonal_periods: ::std::vec::Vec<SeasonalPeriods>,
    /// Id to differentiate different time series for the large-scale case.
    #[builder(setter(into))]
    #[serde(default)]
    pub time_series_id: ::std::vec::Vec<::std::string::String>,
}

/// Arima model information.
#[derive(::serde::Deserialize, ::serde::Serialize, ::typed_builder::TypedBuilder, Debug, PartialEq, Clone)]
#[derive(Default)]
#[serde(rename_all = "camelCase")]
pub struct ArimaModelInfo {
    /// Arima coefficients.
    #[builder(setter(into))]
    #[serde(default)]
    pub arima_coefficients: ArimaCoefficients,
    /// Arima fitting metrics.
    #[builder(setter(into))]
    #[serde(default)]
    pub arima_fitting_metrics: ArimaFittingMetrics,
    /// Whether Arima model fitted with drift or not. It is always false when d is not 1.
    #[builder(setter(into))]
    #[serde(default)]
    pub has_drift: bool,
    /// If true, holiday_effect is a part of time series decomposition result.
    #[builder(setter(into))]
    #[serde(default)]
    pub has_holiday_effect: bool,
    /// If true, spikes_and_dips is a part of time series decomposition result.
    #[builder(setter(into))]
    #[serde(default)]
    pub has_spikes_and_dips: bool,
    /// If true, step_changes is a part of time series decomposition result.
    #[builder(setter(into))]
    #[serde(default)]
    pub has_step_changes: bool,
    /// Non-seasonal order.
    #[builder(setter(into))]
    #[serde(default)]
    pub non_seasonal_order: ArimaOrder,
    /// Seasonal periods. Repeated because multiple periods are supported for one time series.
    #[builder(setter(into))]
    #[serde(default)]
    pub seasonal_periods: ::std::vec::Vec<SeasonalPeriods>,
    /// The time_series_id value for this time series. It will be one of the unique values from the
    /// time_series_id_column specified during ARIMA model training. Only present when
    /// time_series_id_column training option was used.
    #[builder(setter(into))]
    #[serde(default)]
    pub time_series_id: ::std::string::String,
    /// The tuple of time_series_ids identifying this time series. It will be one of the unique tuples
    /// of values present in the time_series_id_columns specified during ARIMA model training. Only
    /// present when time_series_id_columns training option was used and the order of values here are
    /// same as the order of time_series_id_columns.
    #[builder(setter(into))]
    #[serde(default)]
    pub time_series_ids: ::std::vec::Vec<::std::string::String>,
}

/// Arima order, can be used for both non-seasonal and seasonal parts.
#[derive(::serde::Deserialize, ::serde::Serialize, ::typed_builder::TypedBuilder, Debug, PartialEq, Clone)]
#[derive(Default)]
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

/// (Auto-)arima fitting result. Wrap everything in ArimaResult for easier refactoring if we want to
/// use model-specific iteration results.
#[derive(::serde::Deserialize, ::serde::Serialize, ::typed_builder::TypedBuilder, Debug, PartialEq, Clone)]
#[derive(Default)]
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

/// Model evaluation metrics for a single ARIMA forecasting model.
#[derive(::serde::Deserialize, ::serde::Serialize, ::typed_builder::TypedBuilder, Debug, PartialEq, Clone)]
#[derive(Default)]
#[serde(rename_all = "camelCase")]
pub struct ArimaSingleModelForecastingMetrics {
    /// Arima fitting metrics.
    #[builder(setter(into))]
    #[serde(default)]
    pub arima_fitting_metrics: ArimaFittingMetrics,
    /// Is arima model fitted with drift or not. It is always false when d is not 1.
    #[builder(setter(into))]
    #[serde(default)]
    pub has_drift: bool,
    /// If true, holiday_effect is a part of time series decomposition result.
    #[builder(setter(into))]
    #[serde(default)]
    pub has_holiday_effect: bool,
    /// If true, spikes_and_dips is a part of time series decomposition result.
    #[builder(setter(into))]
    #[serde(default)]
    pub has_spikes_and_dips: bool,
    /// If true, step_changes is a part of time series decomposition result.
    #[builder(setter(into))]
    #[serde(default)]
    pub has_step_changes: bool,
    /// Non-seasonal order.
    #[builder(setter(into))]
    #[serde(default)]
    pub non_seasonal_order: ArimaOrder,
    /// Seasonal periods. Repeated because multiple periods are supported for one time series.
    #[builder(setter(into))]
    #[serde(default)]
    pub seasonal_periods: ::std::vec::Vec<SeasonalPeriods>,
    /// The time_series_id value for this time series. It will be one of the unique values from the
    /// time_series_id_column specified during ARIMA model training. Only present when
    /// time_series_id_column training option was used.
    #[builder(setter(into))]
    #[serde(default)]
    pub time_series_id: ::std::string::String,
    /// The tuple of time_series_ids identifying this time series. It will be one of the unique tuples
    /// of values present in the time_series_id_columns specified during ARIMA model training. Only
    /// present when time_series_id_columns training option was used and the order of values here are
    /// same as the order of time_series_id_columns.
    #[builder(setter(into))]
    #[serde(default)]
    pub time_series_ids: ::std::vec::Vec<::std::string::String>,
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
#[derive(::serde::Deserialize, ::serde::Serialize, ::typed_builder::TypedBuilder, Debug, PartialEq, Clone)]
#[derive(Default)]
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
#[derive(::serde::Deserialize, ::serde::Serialize, ::typed_builder::TypedBuilder, Debug, PartialEq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AuditLogConfig {
    /// Specifies the identities that do not cause logging for this type of permission. Follows the same
    /// format of Binding.members.
    #[builder(setter(into))]
    #[serde(default)]
    pub exempted_members: ::std::vec::Vec<::std::string::String>,
    /// The log type that this config enables.
    #[builder(setter(into))]
    pub log_type: LogType,
}

/// Options for external data sources.
#[derive(::serde::Deserialize, ::serde::Serialize, ::typed_builder::TypedBuilder, Debug, PartialEq, Clone)]
#[derive(Default)]
#[serde(rename_all = "camelCase")]
pub struct AvroOptions {
    /// Optional. If sourceFormat is set to "AVRO", indicates whether to interpret logical types as the
    /// corresponding BigQuery data type (for example, TIMESTAMP), instead of using the raw type (for
    /// example, INTEGER).
    #[builder(setter(into))]
    #[serde(default)]
    pub use_avro_logical_types: ::std::option::Option<bool>,
}

/// Output only. Specifies the high level reason why a Job was created.
#[derive(::serde::Deserialize, ::serde::Serialize, Debug, PartialEq, Clone)]
pub enum Code {
        /// Job creation was requested.
    #[serde(rename = "REQUESTED")]
    Requested,
        /// The query request ran beyond a system defined timeout specified by the [timeoutMs field in
        /// the
        /// QueryRequest](https://cloud.google.com/bigquery/docs/reference/rest/v2/jobs/query#queryrequest).
        /// As a result it was considered a long running operation for which a job was created.
    #[serde(rename = "LONG_RUNNING")]
    LongRunning,
        /// The results from the query cannot fit in the response.
    #[serde(rename = "LARGE_RESULTS")]
    LargeResults,
        /// BigQuery has determined that the query needs to be executed as a Job.
    #[serde(rename = "OTHER")]
    Other,
}

/// Reason why BI Engine didn't accelerate the query (or sub-query).
#[derive(::serde::Deserialize, ::serde::Serialize, ::typed_builder::TypedBuilder, Debug, PartialEq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct BiEngineReason {
    /// Output only. High-level BI Engine reason for partial or disabled acceleration
    #[builder(setter(into))]
    pub code: Code,
    /// Output only. Free form human-readable reason for partial or disabled acceleration.
    #[builder(setter(into))]
    #[serde(default)]
    pub message: ::std::string::String,
}

/// Output only. Specifies which mode of BI Engine acceleration was performed (if any).
#[derive(::serde::Deserialize, ::serde::Serialize, Debug, PartialEq, Clone)]
pub enum AccelerationMode {
        /// BI Engine acceleration was attempted but disabled. bi_engine_reasons specifies a more
        /// detailed reason.
    #[serde(rename = "BI_ENGINE_DISABLED")]
    BiEngineDisabled,
        /// Some inputs were accelerated using BI Engine. See bi_engine_reasons for why parts of the
        /// query were not accelerated.
    #[serde(rename = "PARTIAL_INPUT")]
    PartialInput,
        /// All of the query inputs were accelerated using BI Engine.
    #[serde(rename = "FULL_INPUT")]
    FullInput,
        /// All of the query was accelerated using BI Engine.
    #[serde(rename = "FULL_QUERY")]
    FullQuery,
}

/// Output only. Specifies which mode of BI Engine acceleration was performed (if any).
#[derive(::serde::Deserialize, ::serde::Serialize, Debug, PartialEq, Clone)]
pub enum BiEngineMode {
        /// BI Engine disabled the acceleration. bi_engine_reasons specifies a more detailed reason.
    #[serde(rename = "DISABLED")]
    Disabled,
        /// Part of the query was accelerated using BI Engine. See bi_engine_reasons for why parts of
        /// the query were not accelerated.
    #[serde(rename = "PARTIAL")]
    Partial,
        /// All of the query was accelerated using BI Engine.
    #[serde(rename = "FULL")]
    Full,
}

/// Statistics for a BI Engine specific query. Populated as part of JobStatistics2
#[derive(::serde::Deserialize, ::serde::Serialize, ::typed_builder::TypedBuilder, Debug, PartialEq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct BiEngineStatistics {
    /// Output only. Specifies which mode of BI Engine acceleration was performed (if any).
    #[builder(setter(into))]
    pub acceleration_mode: AccelerationMode,
    /// Output only. Specifies which mode of BI Engine acceleration was performed (if any).
    #[builder(setter(into))]
    pub bi_engine_mode: BiEngineMode,
    /// In case of DISABLED or PARTIAL bi_engine_mode, these contain the explanatory reasons as to why
    /// BI Engine could not accelerate. In case the full query was accelerated, this field is not
    /// populated.
    #[builder(setter(into))]
    #[serde(default)]
    pub bi_engine_reasons: ::std::vec::Vec<BiEngineReason>,
}

/// Required. The file format the table data is stored in.
#[derive(::serde::Deserialize, ::serde::Serialize, Debug, PartialEq, Clone)]
pub enum FileFormat {
        /// Apache Parquet format.
    #[serde(rename = "PARQUET")]
    Parquet,
}

/// Required. The table format the metadata only snapshots are stored in.
#[derive(::serde::Deserialize, ::serde::Serialize, Debug, PartialEq, Clone)]
pub enum TableFormat {
        /// Apache Iceberg format.
    #[serde(rename = "ICEBERG")]
    Iceberg,
}

/// Configuration for BigLake managed tables.
#[derive(::serde::Deserialize, ::serde::Serialize, ::typed_builder::TypedBuilder, Debug, PartialEq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct BigLakeConfiguration {
    /// Required. The connection specifying the credentials to be used to read and write to external
    /// storage, such as Cloud Storage. The connection_id can have the form
    /// "<project\_id>.<location\_id>.<connection\_id>" or
    /// "projects/<project\_id>/locations/<location\_id>/connections/<connection\_id>".
    #[builder(setter(into))]
    #[serde(default)]
    pub connection_id: ::std::string::String,
    /// Required. The file format the table data is stored in.
    #[builder(setter(into))]
    pub file_format: FileFormat,
    /// Required. The fully qualified location prefix of the external folder where table data is stored.
    /// The '*' wildcard character is not allowed. The URI should be in the format
    /// "gs://bucket/path_to_table/"
    #[builder(setter(into))]
    #[serde(default)]
    pub storage_uri: ::std::string::String,
    /// Required. The table format the metadata only snapshots are stored in.
    #[builder(setter(into))]
    pub table_format: TableFormat,
}

#[derive(::serde::Deserialize, ::serde::Serialize, ::typed_builder::TypedBuilder, Debug, PartialEq, Clone)]
#[derive(Default)]
#[serde(rename_all = "camelCase")]
pub struct BigQueryModelTraining {
    /// Deprecated.
    #[builder(setter(into))]
    #[serde(default)]
    pub current_iteration: i64,
    /// Deprecated.
    #[builder(setter(into))]
    #[serde(with = "with::int64")]
    #[serde(default)]
    pub expected_total_iterations: i64,
}

/// Information related to a Bigtable column.
#[derive(::serde::Deserialize, ::serde::Serialize, ::typed_builder::TypedBuilder, Debug, PartialEq, Clone)]
#[derive(Default)]
#[serde(rename_all = "camelCase")]
pub struct BigtableColumn {
    /// Optional. The encoding of the values when the type is not STRING. Acceptable encoding values
    /// are: TEXT - indicates values are alphanumeric text strings. BINARY - indicates values are
    /// encoded using HBase Bytes.toBytes family of functions. 'encoding' can also be set at the column
    /// family level. However, the setting at this level takes precedence if 'encoding' is set at both
    /// levels.
    #[builder(setter(into))]
    #[serde(default)]
    pub encoding: ::std::option::Option<::std::string::String>,
    /// Optional. If the qualifier is not a valid BigQuery field identifier i.e. does not match a-zA-Z*,
    /// a valid identifier must be provided as the column field name and is used as field name in
    /// queries.
    #[builder(setter(into))]
    #[serde(default)]
    pub field_name: ::std::option::Option<::std::string::String>,
    /// Optional. If this is set, only the latest version of value in this column are exposed.
    /// 'onlyReadLatest' can also be set at the column family level. However, the setting at this level
    /// takes precedence if 'onlyReadLatest' is set at both levels.
    #[builder(setter(into))]
    #[serde(default)]
    pub only_read_latest: ::std::option::Option<bool>,
    /// [Required] Qualifier of the column. Columns in the parent column family that has this exact
    /// qualifier are exposed as . field. If the qualifier is valid UTF-8 string, it can be specified in
    /// the qualifier_string field. Otherwise, a base-64 encoded value must be set to qualifier_encoded.
    /// The column field name is the same as the column qualifier. However, if the qualifier is not a
    /// valid BigQuery field identifier i.e. does not match a-zA-Z*, a valid identifier must be provided
    /// as field_name.
    #[builder(setter(into))]
    #[serde(default)]
    pub qualifier_encoded: ::std::vec::Vec<u8>,
    /// Qualifier string.
    #[builder(setter(into))]
    #[serde(default)]
    pub qualifier_string: ::std::string::String,
    /// Optional. The type to convert the value in cells of this column. The values are expected to be
    /// encoded using HBase Bytes.toBytes function when using the BINARY encoding value. Following
    /// BigQuery types are allowed (case-sensitive): * BYTES * STRING * INTEGER * FLOAT * BOOLEAN * JSON
    /// Default type is BYTES. 'type' can also be set at the column family level. However, the setting
    /// at this level takes precedence if 'type' is set at both levels.
    #[builder(setter(into))]
    #[serde(rename = "type")]
    #[serde(default)]
    pub ty: ::std::option::Option<::std::string::String>,
}

/// Information related to a Bigtable column family.
#[derive(::serde::Deserialize, ::serde::Serialize, ::typed_builder::TypedBuilder, Debug, PartialEq, Clone)]
#[derive(Default)]
#[serde(rename_all = "camelCase")]
pub struct BigtableColumnFamily {
    /// Optional. Lists of columns that should be exposed as individual fields as opposed to a list of
    /// (column name, value) pairs. All columns whose qualifier matches a qualifier in this list can be
    /// accessed as .. Other columns can be accessed as a list through .Column field.
    #[builder(setter(into))]
    #[serde(default)]
    pub columns: ::std::vec::Vec<BigtableColumn>,
    /// Optional. The encoding of the values when the type is not STRING. Acceptable encoding values
    /// are: TEXT - indicates values are alphanumeric text strings. BINARY - indicates values are
    /// encoded using HBase Bytes.toBytes family of functions. This can be overridden for a specific
    /// column by listing that column in 'columns' and specifying an encoding for it.
    #[builder(setter(into))]
    #[serde(default)]
    pub encoding: ::std::option::Option<::std::string::String>,
    /// Identifier of the column family.
    #[builder(setter(into))]
    #[serde(default)]
    pub family_id: ::std::string::String,
    /// Optional. If this is set only the latest version of value are exposed for all columns in this
    /// column family. This can be overridden for a specific column by listing that column in 'columns'
    /// and specifying a different setting for that column.
    #[builder(setter(into))]
    #[serde(default)]
    pub only_read_latest: ::std::option::Option<bool>,
    /// Optional. The type to convert the value in cells of this column family. The values are expected
    /// to be encoded using HBase Bytes.toBytes function when using the BINARY encoding value. Following
    /// BigQuery types are allowed (case-sensitive): * BYTES * STRING * INTEGER * FLOAT * BOOLEAN * JSON
    /// Default type is BYTES. This can be overridden for a specific column by listing that column in
    /// 'columns' and specifying a type for it.
    #[builder(setter(into))]
    #[serde(rename = "type")]
    #[serde(default)]
    pub ty: ::std::option::Option<::std::string::String>,
}

/// Options specific to Google Cloud Bigtable data sources.
#[derive(::serde::Deserialize, ::serde::Serialize, ::typed_builder::TypedBuilder, Debug, PartialEq, Clone)]
#[derive(Default)]
#[serde(rename_all = "camelCase")]
pub struct BigtableOptions {
    /// Optional. List of column families to expose in the table schema along with their types. This
    /// list restricts the column families that can be referenced in queries and specifies their value
    /// types. You can use this list to do type conversions - see the 'type' field for more details. If
    /// you leave this list empty, all column families are present in the table schema and their values
    /// are read as BYTES. During a query only the column families referenced in that query are read
    /// from Bigtable.
    #[builder(setter(into))]
    #[serde(default)]
    pub column_families: ::std::vec::Vec<BigtableColumnFamily>,
    /// Optional. If field is true, then the column families that are not specified in columnFamilies
    /// list are not exposed in the table schema. Otherwise, they are read with BYTES type values. The
    /// default value is false.
    #[builder(setter(into))]
    #[serde(default)]
    pub ignore_unspecified_column_families: ::std::option::Option<bool>,
    /// Optional. If field is true, then each column family will be read as a single JSON column.
    /// Otherwise they are read as a repeated cell structure containing timestamp/value tuples. The
    /// default value is false.
    #[builder(setter(into))]
    #[serde(default)]
    pub output_column_families_as_json: ::std::option::Option<bool>,
    /// Optional. If field is true, then the rowkey column families will be read and converted to
    /// string. Otherwise they are read with BYTES type values and users need to manually cast them with
    /// CAST if necessary. The default value is false.
    #[builder(setter(into))]
    #[serde(default)]
    pub read_rowkey_as_string: ::std::option::Option<bool>,
}

/// Evaluation metrics for binary classification/classifier models.
#[derive(::serde::Deserialize, ::serde::Serialize, ::typed_builder::TypedBuilder, Debug, PartialEq, Clone)]
#[derive(Default)]
#[serde(rename_all = "camelCase")]
pub struct BinaryClassificationMetrics {
    /// Aggregate classification metrics.
    #[builder(setter(into))]
    #[serde(default)]
    pub aggregate_classification_metrics: AggregateClassificationMetrics,
    /// Binary confusion matrix at multiple thresholds.
    #[builder(setter(into))]
    #[serde(default)]
    pub binary_confusion_matrix_list: ::std::vec::Vec<BinaryConfusionMatrix>,
    /// Label representing the negative class.
    #[builder(setter(into))]
    #[serde(default)]
    pub negative_label: ::std::string::String,
    /// Label representing the positive class.
    #[builder(setter(into))]
    #[serde(default)]
    pub positive_label: ::std::string::String,
}

/// Confusion matrix for binary classification models.
#[derive(::serde::Deserialize, ::serde::Serialize, ::typed_builder::TypedBuilder, Debug, PartialEq, Clone)]
#[derive(Default)]
#[serde(rename_all = "camelCase")]
pub struct BinaryConfusionMatrix {
    /// The fraction of predictions given the correct label.
    #[builder(setter(into))]
    #[serde(default)]
    pub accuracy: f64,
    /// The equally weighted average of recall and precision.
    #[builder(setter(into))]
    #[serde(default)]
    pub f_1_score: f64,
    /// Number of false samples predicted as false.
    #[builder(setter(into))]
    #[serde(with = "with::int64")]
    #[serde(default)]
    pub false_negatives: i64,
    /// Number of false samples predicted as true.
    #[builder(setter(into))]
    #[serde(with = "with::int64")]
    #[serde(default)]
    pub false_positives: i64,
    /// Threshold value used when computing each of the following metric.
    #[builder(setter(into))]
    #[serde(default)]
    pub positive_class_threshold: f64,
    /// The fraction of actual positive predictions that had positive actual labels.
    #[builder(setter(into))]
    #[serde(default)]
    pub precision: f64,
    /// The fraction of actual positive labels that were given a positive prediction.
    #[builder(setter(into))]
    #[serde(default)]
    pub recall: f64,
    /// Number of true samples predicted as false.
    #[builder(setter(into))]
    #[serde(with = "with::int64")]
    #[serde(default)]
    pub true_negatives: i64,
    /// Number of true samples predicted as true.
    #[builder(setter(into))]
    #[serde(with = "with::int64")]
    #[serde(default)]
    pub true_positives: i64,
}

/// Associates `members`, or principals, with a `role`.
#[derive(::serde::Deserialize, ::serde::Serialize, ::typed_builder::TypedBuilder, Debug, PartialEq, Clone)]
#[derive(Default)]
#[serde(rename_all = "camelCase")]
pub struct Binding {
    /// The condition that is associated with this binding. If the condition evaluates to `true`, then
    /// this binding applies to the current request. If the condition evaluates to `false`, then this
    /// binding does not apply to the current request. However, a different role binding might grant the
    /// same role to one or more of the principals in this binding. To learn which resources support
    /// conditions in their IAM policies, see the [IAM
    /// documentation](https://cloud.google.com/iam/help/conditions/resource-policies).
    #[builder(setter(into))]
    #[serde(default)]
    pub condition: Expr,
    /// Specifies the principals requesting access for a Google Cloud resource. `members` can have the
    /// following values: * `allUsers`: A special identifier that represents anyone who is on the
    /// internet; with or without a Google account. * `allAuthenticatedUsers`: A special identifier that
    /// represents anyone who is authenticated with a Google account or a service account. Does not
    /// include identities that come from external identity providers (IdPs) through identity
    /// federation. * `user:{emailid}`: An email address that represents a specific Google account. For
    /// example, `alice@example.com` . * `serviceAccount:{emailid}`: An email address that represents a
    /// Google service account. For example, `my-other-app@appspot.gserviceaccount.com`. *
    /// `serviceAccount:{projectid}.svc.id.goog[{namespace}/{kubernetes-sa}]`: An identifier for a
    /// [Kubernetes service
    /// account](https://cloud.google.com/kubernetes-engine/docs/how-to/kubernetes-service-accounts).
    /// For example, `my-project.svc.id.goog[my-namespace/my-kubernetes-sa]`. * `group:{emailid}`: An
    /// email address that represents a Google group. For example, `admins@example.com`. *
    /// `domain:{domain}`: The G Suite domain (primary) that represents all the users of that domain.
    /// For example, `google.com` or `example.com`. *
    /// `principal://iam.googleapis.com/locations/global/workforcePools/{pool_id}/subject/{subject_attribute_value}`:
    /// A single identity in a workforce identity pool. *
    /// `principalSet://iam.googleapis.com/locations/global/workforcePools/{pool_id}/group/{group_id}`:
    /// All workforce identities in a group. *
    /// `principalSet://iam.googleapis.com/locations/global/workforcePools/{pool_id}/attribute.{attribute_name}/{attribute_value}`:
    /// All workforce identities with a specific attribute value. *
    /// `principalSet://iam.googleapis.com/locations/global/workforcePools/{pool_id}/*`: All identities
    /// in a workforce identity pool. *
    /// `principal://iam.googleapis.com/projects/{project_number}/locations/global/workloadIdentityPools/{pool_id}/subject/{subject_attribute_value}`:
    /// A single identity in a workload identity pool. *
    /// `principalSet://iam.googleapis.com/projects/{project_number}/locations/global/workloadIdentityPools/{pool_id}/group/{group_id}`:
    /// A workload identity pool group. *
    /// `principalSet://iam.googleapis.com/projects/{project_number}/locations/global/workloadIdentityPools/{pool_id}/attribute.{attribute_name}/{attribute_value}`:
    /// All identities in a workload identity pool with a certain attribute. *
    /// `principalSet://iam.googleapis.com/projects/{project_number}/locations/global/workloadIdentityPools/{pool_id}/*`:
    /// All identities in a workload identity pool. * `deleted:user:{emailid}?uid={uniqueid}`: An email
    /// address (plus unique identifier) representing a user that has been recently deleted. For
    /// example, `alice@example.com?uid=123456789012345678901`. If the user is recovered, this value
    /// reverts to `user:{emailid}` and the recovered user retains the role in the binding. *
    /// `deleted:serviceAccount:{emailid}?uid={uniqueid}`: An email address (plus unique identifier)
    /// representing a service account that has been recently deleted. For example,
    /// `my-other-app@appspot.gserviceaccount.com?uid=123456789012345678901`. If the service account is
    /// undeleted, this value reverts to `serviceAccount:{emailid}` and the undeleted service account
    /// retains the role in the binding. * `deleted:group:{emailid}?uid={uniqueid}`: An email address
    /// (plus unique identifier) representing a Google group that has been recently deleted. For
    /// example, `admins@example.com?uid=123456789012345678901`. If the group is recovered, this value
    /// reverts to `group:{emailid}` and the recovered group retains the role in the binding. *
    /// `deleted:principal://iam.googleapis.com/locations/global/workforcePools/{pool_id}/subject/{subject_attribute_value}`:
    /// Deleted single identity in a workforce identity pool. For example,
    /// `deleted:principal://iam.googleapis.com/locations/global/workforcePools/my-pool-id/subject/my-subject-attribute-value`.
    #[builder(setter(into))]
    #[serde(default)]
    pub members: ::std::vec::Vec<::std::string::String>,
    /// Role that is assigned to the list of `members`, or principals. For example, `roles/viewer`,
    /// `roles/editor`, or `roles/owner`. For an overview of the IAM roles and permissions, see the [IAM
    /// documentation](https://cloud.google.com/iam/docs/roles-overview). For a list of the available
    /// pre-defined roles, see [here](https://cloud.google.com/iam/docs/understanding-roles).
    #[builder(setter(into))]
    #[serde(default)]
    pub role: ::std::string::String,
}

#[derive(::serde::Deserialize, ::serde::Serialize, ::typed_builder::TypedBuilder, Debug, PartialEq, Clone)]
#[derive(Default)]
#[serde(rename_all = "camelCase")]
pub struct BqmlIterationResult {
    /// Deprecated.
    #[builder(setter(into))]
    #[serde(with = "with::int64")]
    #[serde(default)]
    pub duration_ms: i64,
    /// Deprecated.
    #[builder(setter(into))]
    #[serde(default)]
    pub eval_loss: f64,
    /// Deprecated.
    #[builder(setter(into))]
    #[serde(default)]
    pub index: i64,
    /// Deprecated.
    #[builder(setter(into))]
    #[serde(default)]
    pub learn_rate: f64,
    /// Deprecated.
    #[builder(setter(into))]
    #[serde(default)]
    pub training_loss: f64,
}

/// Deprecated.
#[derive(::serde::Deserialize, ::serde::Serialize, ::typed_builder::TypedBuilder, Debug, PartialEq, Clone)]
#[derive(Default)]
#[serde(rename_all = "camelCase")]
pub struct TrainingOptions {
    #[builder(setter(into))]
    #[serde(default)]
    pub early_stop: bool,
    #[builder(setter(into))]
    #[serde(default)]
    pub l_1_reg: f64,
    #[builder(setter(into))]
    #[serde(default)]
    pub l_2_reg: f64,
    #[builder(setter(into))]
    #[serde(default)]
    pub learn_rate: f64,
    #[builder(setter(into))]
    #[serde(default)]
    pub learn_rate_strategy: ::std::string::String,
    #[builder(setter(into))]
    #[serde(default)]
    pub line_search_init_learn_rate: f64,
    #[builder(setter(into))]
    #[serde(with = "with::int64")]
    #[serde(default)]
    pub max_iteration: i64,
    #[builder(setter(into))]
    #[serde(default)]
    pub min_rel_progress: f64,
    #[builder(setter(into))]
    #[serde(default)]
    pub warm_start: bool,
}

#[derive(::serde::Deserialize, ::serde::Serialize, ::typed_builder::TypedBuilder, Debug, PartialEq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct BqmlTrainingRun {
    /// Deprecated.
    #[builder(setter(into))]
    #[serde(default)]
    pub iteration_results: ::std::vec::Vec<BqmlIterationResult>,
    /// Deprecated.
    #[builder(setter(into))]
    pub start_time: ::timestamp::Timestamp,
    /// Deprecated.
    #[builder(setter(into))]
    #[serde(default)]
    pub state: ::std::string::String,
    /// Deprecated.
    #[builder(setter(into))]
    #[serde(default)]
    pub training_options: TrainingOptions,
}

/// Representative value of a categorical feature.
#[derive(::serde::Deserialize, ::serde::Serialize, ::typed_builder::TypedBuilder, Debug, PartialEq, Clone)]
#[derive(Default)]
#[serde(rename_all = "camelCase")]
pub struct CategoricalValue {
    /// Counts of all categories for the categorical feature. If there are more than ten categories, we
    /// return top ten (by count) and return one more CategoryCount with category "_OTHER_" and count as
    /// aggregate counts of remaining categories.
    #[builder(setter(into))]
    #[serde(default)]
    pub category_counts: ::std::vec::Vec<CategoryCount>,
}

/// Represents the count of a single category within the cluster.
#[derive(::serde::Deserialize, ::serde::Serialize, ::typed_builder::TypedBuilder, Debug, PartialEq, Clone)]
#[derive(Default)]
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

/// Information about base table and clone time of a table clone.
#[derive(::serde::Deserialize, ::serde::Serialize, ::typed_builder::TypedBuilder, Debug, PartialEq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CloneDefinition {
    /// Required. Reference describing the ID of the table that was cloned.
    #[builder(setter(into))]
    #[serde(default)]
    pub base_table_reference: TableReference,
    /// Required. The time at which the base table was cloned. This value is reported in the JSON
    /// response using RFC3339 format.
    #[builder(setter(into))]
    pub clone_time: ::timestamp::Timestamp,
}

/// Message containing the information about one cluster.
#[derive(::serde::Deserialize, ::serde::Serialize, ::typed_builder::TypedBuilder, Debug, PartialEq, Clone)]
#[derive(Default)]
#[serde(rename_all = "camelCase")]
pub struct Cluster {
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
    /// Values of highly variant features for this cluster.
    #[builder(setter(into))]
    #[serde(default)]
    pub feature_values: ::std::vec::Vec<FeatureValue>,
}

/// Information about a single cluster for clustering model.
#[derive(::serde::Deserialize, ::serde::Serialize, ::typed_builder::TypedBuilder, Debug, PartialEq, Clone)]
#[derive(Default)]
#[serde(rename_all = "camelCase")]
pub struct ClusterInfo {
    /// Centroid id.
    #[builder(setter(into))]
    #[serde(with = "with::int64")]
    #[serde(default)]
    pub centroid_id: i64,
    /// Cluster radius, the average distance from centroid to each point assigned to the cluster.
    #[builder(setter(into))]
    #[serde(default)]
    pub cluster_radius: f64,
    /// Cluster size, the total number of points assigned to the cluster.
    #[builder(setter(into))]
    #[serde(with = "with::int64")]
    #[serde(default)]
    pub cluster_size: i64,
}

/// Configures table clustering.
#[derive(::serde::Deserialize, ::serde::Serialize, ::typed_builder::TypedBuilder, Debug, PartialEq, Clone)]
#[derive(Default)]
#[serde(rename_all = "camelCase")]
pub struct Clustering {
    /// One or more fields on which data should be clustered. Only top-level, non-repeated, simple-type
    /// fields are supported. The ordering of the clustering fields should be prioritized from most to
    /// least important for filtering purposes. Additional information on limitations can be found here:
    /// https://cloud.google.com/bigquery/docs/creating-clustered-tables#limitations
    #[builder(setter(into))]
    #[serde(default)]
    pub fields: ::std::vec::Vec<::std::string::String>,
}

/// Evaluation metrics for clustering models.
#[derive(::serde::Deserialize, ::serde::Serialize, ::typed_builder::TypedBuilder, Debug, PartialEq, Clone)]
#[derive(Default)]
#[serde(rename_all = "camelCase")]
pub struct ClusteringMetrics {
    /// Information for all clusters.
    #[builder(setter(into))]
    #[serde(default)]
    pub clusters: ::std::vec::Vec<Cluster>,
    /// Davies-Bouldin index.
    #[builder(setter(into))]
    #[serde(default)]
    pub davies_bouldin_index: f64,
    /// Mean of squared distances between each sample to its cluster centroid.
    #[builder(setter(into))]
    #[serde(default)]
    pub mean_squared_distance: f64,
}

/// Confusion matrix for multi-class classification models.
#[derive(::serde::Deserialize, ::serde::Serialize, ::typed_builder::TypedBuilder, Debug, PartialEq, Clone)]
#[derive(Default)]
#[serde(rename_all = "camelCase")]
pub struct ConfusionMatrix {
    /// Confidence threshold used when computing the entries of the confusion matrix.
    #[builder(setter(into))]
    #[serde(default)]
    pub confidence_threshold: f64,
    /// One row per actual label.
    #[builder(setter(into))]
    #[serde(default)]
    pub rows: ::std::vec::Vec<Row>,
}

/// A connection-level property to customize query behavior. Under JDBC, these correspond directly
/// to connection properties passed to the DriverManager. Under ODBC, these correspond to properties
/// in the connection string. Currently supported connection properties: * **dataset_project_id**:
/// represents the default project for datasets that are used in the query. Setting the system
/// variable `@@dataset_project_id` achieves the same behavior. For more information about system
/// variables, see: https://cloud.google.com/bigquery/docs/reference/system-variables *
/// **time_zone**: represents the default timezone used to run the query. * **session_id**:
/// associates the query with a given session. * **query_label**: associates the query with a given
/// job label. If set, all subsequent queries in a script or session will have this label. For the
/// format in which a you can specify a query label, see labels in the JobConfiguration resource
/// type: https://cloud.google.com/bigquery/docs/reference/rest/v2/Job#jobconfiguration Additional
/// properties are allowed, but ignored. Specifying multiple connection properties with the same key
/// returns an error.
#[derive(::serde::Deserialize, ::serde::Serialize, ::typed_builder::TypedBuilder, Debug, PartialEq, Clone)]
#[derive(Default)]
#[serde(rename_all = "camelCase")]
pub struct ConnectionProperty {
    /// The key of the property to set.
    #[builder(setter(into))]
    #[serde(default)]
    pub key: ::std::string::String,
    /// The value of the property to set.
    #[builder(setter(into))]
    #[serde(default)]
    pub value: ::std::string::String,
}

/// Information related to a CSV data source.
#[derive(::serde::Deserialize, ::serde::Serialize, ::typed_builder::TypedBuilder, Debug, PartialEq, Clone)]
#[derive(Default)]
#[serde(rename_all = "camelCase")]
pub struct CsvOptions {
    /// Optional. Indicates if BigQuery should accept rows that are missing trailing optional columns.
    /// If true, BigQuery treats missing trailing columns as null values. If false, records with missing
    /// trailing columns are treated as bad records, and if there are too many bad records, an invalid
    /// error is returned in the job result. The default value is false.
    #[builder(setter(into))]
    #[serde(default)]
    pub allow_jagged_rows: ::std::option::Option<bool>,
    /// Optional. Indicates if BigQuery should allow quoted data sections that contain newline
    /// characters in a CSV file. The default value is false.
    #[builder(setter(into))]
    #[serde(default)]
    pub allow_quoted_newlines: ::std::option::Option<bool>,
    /// Optional. The character encoding of the data. The supported values are UTF-8, ISO-8859-1,
    /// UTF-16BE, UTF-16LE, UTF-32BE, and UTF-32LE. The default value is UTF-8. BigQuery decodes the
    /// data after the raw, binary data has been split using the values of the quote and fieldDelimiter
    /// properties.
    #[builder(setter(into))]
    #[serde(default)]
    pub encoding: ::std::option::Option<::std::string::String>,
    /// Optional. The separator character for fields in a CSV file. The separator is interpreted as a
    /// single byte. For files encoded in ISO-8859-1, any single character can be used as a separator.
    /// For files encoded in UTF-8, characters represented in decimal range 1-127 (U+0001-U+007F) can be
    /// used without any modification. UTF-8 characters encoded with multiple bytes (i.e. U+0080 and
    /// above) will have only the first byte used for separating fields. The remaining bytes will be
    /// treated as a part of the field. BigQuery also supports the escape sequence "\t" (U+0009) to
    /// specify a tab separator. The default value is comma (",", U+002C).
    #[builder(setter(into))]
    #[serde(default)]
    pub field_delimiter: ::std::option::Option<::std::string::String>,
    /// [Optional] A custom string that will represent a NULL value in CSV import data.
    #[builder(setter(into))]
    #[serde(default)]
    pub null_marker: ::std::option::Option<::std::string::String>,
    /// Optional. Indicates if the embedded ASCII control characters (the first 32 characters in the
    /// ASCII-table, from '\x00' to '\x1F') are preserved.
    #[builder(setter(into))]
    #[serde(default)]
    pub preserve_ascii_control_characters: ::std::option::Option<bool>,
    /// Optional. The value that is used to quote data sections in a CSV file. BigQuery converts the
    /// string to ISO-8859-1 encoding, and then uses the first byte of the encoded string to split the
    /// data in its raw, binary state. The default value is a double-quote ("). If your data does not
    /// contain quoted sections, set the property value to an empty string. If your data contains quoted
    /// newline characters, you must also set the allowQuotedNewlines property to true. To include the
    /// specific quote character within a quoted value, precede it with an additional matching quote
    /// character. For example, if you want to escape the default character ' " ', use ' "" '.
    #[builder(setter(into))]
    #[serde(default)]
    pub quote: ::std::option::Option<::std::string::String>,
    /// Optional. The number of rows at the top of a CSV file that BigQuery will skip when reading the
    /// data. The default value is 0. This property is useful if you have header rows in the file that
    /// should be skipped. When autodetect is on, the behavior is the following: * skipLeadingRows
    /// unspecified - Autodetect tries to detect headers in the first row. If they are not detected, the
    /// row is read as data. Otherwise data is read starting from the second row. * skipLeadingRows is 0
    /// - Instructs autodetect that there are no headers and data should be read starting from the first
    /// row. * skipLeadingRows = N > 0 - Autodetect skips N-1 rows and tries to detect headers in row N.
    /// If headers are not detected, row N is just skipped. Otherwise row N is used to extract column
    /// names for the detected schema.
    #[builder(setter(into))]
    #[serde(with = "with::int64::option")]
    #[serde(default)]
    pub skip_leading_rows: ::std::option::Option<i64>,
}

/// Options for data format adjustments.
#[derive(::serde::Deserialize, ::serde::Serialize, ::typed_builder::TypedBuilder, Debug, PartialEq, Clone)]
#[derive(Default)]
#[serde(rename_all = "camelCase")]
pub struct DataFormatOptions {
    /// Optional. Output timestamp as usec int64. Default is false.
    #[builder(setter(into))]
    #[serde(default)]
    pub use_int_64_timestamp: ::std::option::Option<bool>,
}

/// Statistics for data-masking.
#[derive(::serde::Deserialize, ::serde::Serialize, ::typed_builder::TypedBuilder, Debug, PartialEq, Clone)]
#[derive(Default)]
#[serde(rename_all = "camelCase")]
pub struct DataMaskingStatistics {
    /// Whether any accessed data was protected by the data masking.
    #[builder(setter(into))]
    #[serde(default)]
    pub data_masking_applied: bool,
}

/// Data split result. This contains references to the training and evaluation data tables that were
/// used to train the model.
#[derive(::serde::Deserialize, ::serde::Serialize, ::typed_builder::TypedBuilder, Debug, PartialEq, Clone)]
#[derive(Default)]
#[serde(rename_all = "camelCase")]
pub struct DataSplitResult {
    /// Table reference of the evaluation data after split.
    #[builder(setter(into))]
    #[serde(default)]
    pub evaluation_table: TableReference,
    /// Table reference of the test data after split.
    #[builder(setter(into))]
    #[serde(default)]
    pub test_table: TableReference,
    /// Table reference of the training data after split.
    #[builder(setter(into))]
    #[serde(default)]
    pub training_table: TableReference,
}

/// An object that defines dataset access for an entity.
#[derive(::serde::Deserialize, ::serde::Serialize, ::typed_builder::TypedBuilder, Debug, PartialEq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Access {
    /// [Pick one] A grant authorizing all resources of a particular type in a particular dataset access
    /// to this dataset. Only views are supported for now. The role field is not required when this
    /// field is set. If that dataset is deleted and re-created, its access needs to be granted again
    /// via an update operation.
    #[builder(setter(into))]
    pub dataset: DatasetAccessEntry,
    /// [Pick one] A domain to grant access to. Any users signed in with the domain specified will be
    /// granted the specified access. Example: "example.com". Maps to IAM policy member "domain:DOMAIN".
    #[builder(setter(into))]
    #[serde(default)]
    pub domain: ::std::string::String,
    /// [Pick one] An email address of a Google Group to grant access to. Maps to IAM policy member
    /// "group:GROUP".
    #[builder(setter(into))]
    #[serde(default)]
    pub group_by_email: ::std::string::String,
    /// [Pick one] Some other type of member that appears in the IAM Policy but isn't a user, group,
    /// domain, or special group.
    #[builder(setter(into))]
    #[serde(default)]
    pub iam_member: ::std::string::String,
    /// An IAM role ID that should be granted to the user, group, or domain specified in this access
    /// entry. The following legacy mappings will be applied: OWNER <=> roles/bigquery.dataOwner WRITER
    /// <=> roles/bigquery.dataEditor READER <=> roles/bigquery.dataViewer This field will accept any of
    /// the above formats, but will return only the legacy format. For example, if you set this field to
    /// "roles/bigquery.dataOwner", it will be returned back as "OWNER".
    #[builder(setter(into))]
    #[serde(default)]
    pub role: ::std::string::String,
    /// [Pick one] A routine from a different dataset to grant access to. Queries executed against that
    /// routine will have read access to views/tables/routines in this dataset. Only UDF is supported
    /// for now. The role field is not required when this field is set. If that routine is updated by
    /// any user, access to the routine needs to be granted again via an update operation.
    #[builder(setter(into))]
    #[serde(default)]
    pub routine: RoutineReference,
    /// [Pick one] A special group to grant access to. Possible values include: projectOwners: Owners of
    /// the enclosing project. projectReaders: Readers of the enclosing project. projectWriters: Writers
    /// of the enclosing project. allAuthenticatedUsers: All authenticated BigQuery users. Maps to
    /// similarly-named IAM members.
    #[builder(setter(into))]
    #[serde(default)]
    pub special_group: ::std::string::String,
    /// [Pick one] An email address of a user to grant access to. For example: fred@example.com. Maps to
    /// IAM policy member "user:EMAIL" or "serviceAccount:EMAIL".
    #[builder(setter(into))]
    #[serde(default)]
    pub user_by_email: ::std::string::String,
    /// [Pick one] A view from a different dataset to grant access to. Queries executed against that
    /// view will have read access to views/tables/routines in this dataset. The role field is not
    /// required when this field is set. If that view is updated by any user, access to the view needs
    /// to be granted again via an update operation.
    #[builder(setter(into))]
    #[serde(default)]
    pub view: TableReference,
}

/// Optional. Defines the default rounding mode specification of new decimal fields (NUMERIC OR
/// BIGNUMERIC) in the table. During table creation or update, if a decimal field is added to this
/// table without an explicit rounding mode specified, then the field inherits the table default
/// rounding mode. Changing this field doesn't affect existing fields.
#[derive(::serde::Deserialize, ::serde::Serialize, Debug, PartialEq, Clone)]
pub enum DefaultRoundingMode {
        /// ROUND_HALF_AWAY_FROM_ZERO rounds half values away from zero when applying precision and
        /// scale upon writing of NUMERIC and BIGNUMERIC values. For Scale: 0 1.1, 1.2, 1.3, 1.4 => 1
        /// 1.5, 1.6, 1.7, 1.8, 1.9 => 2
    #[serde(rename = "ROUND_HALF_AWAY_FROM_ZERO")]
    RoundHalfAwayFromZero,
        /// ROUND_HALF_EVEN rounds half values to the nearest even value when applying precision and
        /// scale upon writing of NUMERIC and BIGNUMERIC values. For Scale: 0 1.1, 1.2, 1.3, 1.4 => 1
        /// 1.5 => 2 1.6, 1.7, 1.8, 1.9 => 2 2.5 => 2
    #[serde(rename = "ROUND_HALF_EVEN")]
    RoundHalfEven,
}

/// The labels associated with this dataset. You can use these to organize and group your datasets.
/// You can set this property when inserting or updating a dataset. See Creating and Updating
/// Dataset Labels for more information.
pub type Labels = ::std::collections::HashMap<::std::string::String, ::std::string::String>;

/// Optional. Updates storage_billing_model for the dataset.
#[derive(::serde::Deserialize, ::serde::Serialize, Debug, PartialEq, Clone)]
pub enum StorageBillingModel {
        /// Billing for logical bytes.
    #[serde(rename = "LOGICAL")]
    Logical,
        /// Billing for physical bytes.
    #[serde(rename = "PHYSICAL")]
    Physical,
}

/// A global tag managed by Resource Manager.
/// https://cloud.google.com/iam/docs/tags-access-control#definitions
#[derive(::serde::Deserialize, ::serde::Serialize, ::typed_builder::TypedBuilder, Debug, PartialEq, Clone)]
#[derive(Default)]
#[serde(rename_all = "camelCase")]
pub struct Tags {
    /// Required. The namespaced friendly name of the tag key, e.g. "12345/environment" where 12345 is
    /// org id.
    #[builder(setter(into))]
    #[serde(default)]
    pub tag_key: ::std::string::String,
    /// Required. The friendly short name of the tag value, e.g. "production".
    #[builder(setter(into))]
    #[serde(default)]
    pub tag_value: ::std::string::String,
}

#[derive(::serde::Deserialize, ::serde::Serialize, ::typed_builder::TypedBuilder, Debug, PartialEq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Dataset {
    /// Optional. An array of objects that define dataset access for one or more entities. You can set
    /// this property when inserting or updating a dataset in order to control who is allowed to access
    /// the data. If unspecified at dataset creation time, BigQuery adds default dataset access for the
    /// following entities: access.specialGroup: projectReaders; access.role: READER;
    /// access.specialGroup: projectWriters; access.role: WRITER; access.specialGroup: projectOwners;
    /// access.role: OWNER; access.userByEmail: [dataset creator email]; access.role: OWNER;
    #[builder(setter(into))]
    #[serde(default)]
    pub access: ::std::vec::Vec<Access>,
    /// Output only. The time when this dataset was created, in milliseconds since the epoch.
    #[builder(setter(into))]
    #[serde(with = "with::int64")]
    #[serde(default)]
    pub creation_time: i64,
    /// Required. A reference that identifies the dataset.
    #[builder(setter(into))]
    #[serde(default)]
    pub dataset_reference: DatasetReference,
    /// Optional. Defines the default collation specification of future tables created in the dataset.
    /// If a table is created in this dataset without table-level default collation, then the table
    /// inherits the dataset default collation, which is applied to the string fields that do not have
    /// explicit collation specified. A change to this field affects only tables created afterwards, and
    /// does not alter the existing tables. The following values are supported: * 'und:ci': undetermined
    /// locale, case insensitive. * '': empty string. Default to case-sensitive behavior.
    #[builder(setter(into))]
    #[serde(default)]
    pub default_collation: ::std::option::Option<::std::string::String>,
    /// The default encryption key for all tables in the dataset. Once this property is set, all
    /// newly-created partitioned tables in the dataset will have encryption key set to this value,
    /// unless table creation request (or query) overrides the key.
    #[builder(setter(into))]
    #[serde(default)]
    pub default_encryption_configuration: EncryptionConfiguration,
    /// This default partition expiration, expressed in milliseconds. When new time-partitioned tables
    /// are created in a dataset where this property is set, the table will inherit this value,
    /// propagated as the `TimePartitioning.expirationMs` property on the new table. If you set
    /// `TimePartitioning.expirationMs` explicitly when creating a table, the
    /// `defaultPartitionExpirationMs` of the containing dataset is ignored. When creating a partitioned
    /// table, if `defaultPartitionExpirationMs` is set, the `defaultTableExpirationMs` value is ignored
    /// and the table will not be inherit a table expiration deadline.
    #[builder(setter(into))]
    #[serde(with = "with::int64")]
    #[serde(default)]
    pub default_partition_expiration_ms: i64,
    /// Optional. Defines the default rounding mode specification of new tables created within this
    /// dataset. During table creation, if this field is specified, the table within this dataset will
    /// inherit the default rounding mode of the dataset. Setting the default rounding mode on a table
    /// overrides this option. Existing tables in the dataset are unaffected. If columns are defined
    /// during that table creation, they will immediately inherit the table's default rounding mode,
    /// unless otherwise specified.
    #[builder(setter(into))]
    pub default_rounding_mode: DefaultRoundingMode,
    /// Optional. The default lifetime of all tables in the dataset, in milliseconds. The minimum
    /// lifetime value is 3600000 milliseconds (one hour). To clear an existing default expiration with
    /// a PATCH request, set to 0. Once this property is set, all newly-created tables in the dataset
    /// will have an expirationTime property set to the creation time plus the value in this property,
    /// and changing the value will only affect new tables, not existing ones. When the expirationTime
    /// for a given table is reached, that table will be deleted automatically. If a table's
    /// expirationTime is modified or removed before the table expires, or if you provide an explicit
    /// expirationTime when creating a table, that value takes precedence over the default expiration
    /// time indicated by this property.
    #[builder(setter(into))]
    #[serde(with = "with::int64::option")]
    #[serde(default)]
    pub default_table_expiration_ms: ::std::option::Option<i64>,
    /// Optional. A user-friendly description of the dataset.
    #[builder(setter(into))]
    #[serde(default)]
    pub description: ::std::option::Option<::std::string::String>,
    /// Output only. A hash of the resource.
    #[builder(setter(into))]
    #[serde(default)]
    pub etag: ::std::string::String,
    /// Optional. Reference to a read-only external dataset defined in data catalogs outside of
    /// BigQuery. Filled out when the dataset type is EXTERNAL.
    #[builder(setter(into))]
    #[serde(default)]
    pub external_dataset_reference: ::std::option::Option<ExternalDatasetReference>,
    /// Optional. A descriptive name for the dataset.
    #[builder(setter(into))]
    #[serde(default)]
    pub friendly_name: ::std::option::Option<::std::string::String>,
    /// Output only. The fully-qualified unique name of the dataset in the format projectId:datasetId.
    /// The dataset name without the project name is given in the datasetId field. When creating a new
    /// dataset, leave this field blank, and instead specify the datasetId field.
    #[builder(setter(into))]
    #[serde(default)]
    pub id: ::std::string::String,
    /// Optional. TRUE if the dataset and its table names are case-insensitive, otherwise FALSE. By
    /// default, this is FALSE, which means the dataset and its table names are case-sensitive. This
    /// field does not affect routine references.
    #[builder(setter(into))]
    #[serde(default)]
    pub is_case_insensitive: ::std::option::Option<bool>,
    /// Output only. The resource type.
    #[builder(setter(into))]
    #[serde(default)]
    pub kind: ::std::string::String,
    /// The labels associated with this dataset. You can use these to organize and group your datasets.
    /// You can set this property when inserting or updating a dataset. See Creating and Updating
    /// Dataset Labels for more information.
    #[builder(setter(into))]
    #[serde(default)]
    pub labels: Labels,
    /// Output only. The date when this dataset was last modified, in milliseconds since the epoch.
    #[builder(setter(into))]
    #[serde(with = "with::int64")]
    #[serde(default)]
    pub last_modified_time: i64,
    /// Optional. The source dataset reference when the dataset is of type LINKED. For all other dataset
    /// types it is not set. This field cannot be updated once it is set. Any attempt to update this
    /// field using Update and Patch API Operations will be ignored.
    #[builder(setter(into))]
    #[serde(default)]
    pub linked_dataset_source: ::std::option::Option<LinkedDatasetSource>,
    /// The geographic location where the dataset should reside. See
    /// https://cloud.google.com/bigquery/docs/locations for supported locations.
    #[builder(setter(into))]
    #[serde(default)]
    pub location: ::std::string::String,
    /// Optional. Defines the time travel window in hours. The value can be from 48 to 168 hours (2 to 7
    /// days). The default value is 168 hours if this is not set.
    #[builder(setter(into))]
    #[serde(with = "with::int64::option")]
    #[serde(default)]
    pub max_time_travel_hours: ::std::option::Option<i64>,
    /// Output only. Reserved for future use.
    #[builder(setter(into))]
    #[serde(default)]
    pub satisfies_pzi: bool,
    /// Output only. Reserved for future use.
    #[builder(setter(into))]
    #[serde(default)]
    pub satisfies_pzs: bool,
    /// Output only. A URL that can be used to access the resource again. You can use this URL in Get or
    /// Update requests to the resource.
    #[builder(setter(into))]
    #[serde(default)]
    pub self_link: ::std::string::String,
    /// Optional. Updates storage_billing_model for the dataset.
    #[builder(setter(into))]
    pub storage_billing_model: StorageBillingModel,
    /// Output only. Tags for the Dataset.
    #[builder(setter(into))]
    #[serde(default)]
    pub tags: ::std::vec::Vec<Tags>,
    /// Output only. Same as `type` in `ListFormatDataset`. The type of the dataset, one of: * DEFAULT -
    /// only accessible by owner and authorized accounts, * PUBLIC - accessible by everyone, * LINKED -
    /// linked dataset, * EXTERNAL - dataset with definition in external metadata catalog. --
    /// *BIGLAKE_METASTORE - dataset that references a database created in BigLakeMetastore service. --
    #[builder(setter(into))]
    #[serde(rename = "type")]
    #[serde(default)]
    pub ty: ::std::string::String,
}

#[derive(::serde::Deserialize, ::serde::Serialize, Debug, PartialEq, Clone)]
pub enum TargetTypes {
        /// This entry applies to views in the dataset.
    #[serde(rename = "VIEWS")]
    Views,
        /// This entry applies to routines in the dataset.
    #[serde(rename = "ROUTINES")]
    Routines,
}

/// Grants all resources of particular types in a particular dataset read access to the current
/// dataset. Similar to how individually authorized views work, updates to any resource granted
/// through its dataset (including creation of new resources) requires read permission to referenced
/// resources, plus write permission to the authorizing dataset.
#[derive(::serde::Deserialize, ::serde::Serialize, ::typed_builder::TypedBuilder, Debug, PartialEq, Clone)]
#[derive(Default)]
#[serde(rename_all = "camelCase")]
pub struct DatasetAccessEntry {
    /// The dataset this entry applies to
    #[builder(setter(into))]
    #[serde(default)]
    pub dataset: DatasetReference,
    /// Which resources in the dataset this entry applies to. Currently, only views are supported, but
    /// additional target types may be added in the future.
    #[builder(setter(into))]
    #[serde(default)]
    pub target_types: ::std::vec::Vec<TargetTypes>,
}

/// A dataset resource with only a subset of fields, to be returned in a list of datasets.
#[derive(::serde::Deserialize, ::serde::Serialize, ::typed_builder::TypedBuilder, Debug, PartialEq, Clone)]
#[derive(Default)]
#[serde(rename_all = "camelCase")]
pub struct Datasets {
    /// The dataset reference. Use this property to access specific parts of the dataset's ID, such as
    /// project ID or dataset ID.
    #[builder(setter(into))]
    #[serde(default)]
    pub dataset_reference: DatasetReference,
    /// An alternate name for the dataset. The friendly name is purely decorative in nature.
    #[builder(setter(into))]
    #[serde(default)]
    pub friendly_name: ::std::string::String,
    /// The fully-qualified, unique, opaque ID of the dataset.
    #[builder(setter(into))]
    #[serde(default)]
    pub id: ::std::string::String,
    /// The resource type. This property always returns the value "bigquery#dataset"
    #[builder(setter(into))]
    #[serde(default)]
    pub kind: ::std::string::String,
    /// The labels associated with this dataset. You can use these to organize and group your datasets.
    #[builder(setter(into))]
    #[serde(default)]
    pub labels: Labels,
    /// The geographic location where the dataset resides.
    #[builder(setter(into))]
    #[serde(default)]
    pub location: ::std::string::String,
}

/// Response format for a page of results when listing datasets.
#[derive(::serde::Deserialize, ::serde::Serialize, ::typed_builder::TypedBuilder, Debug, PartialEq, Clone)]
#[derive(Default)]
#[serde(rename_all = "camelCase")]
pub struct DatasetList {
    /// An array of the dataset resources in the project. Each resource contains basic information. For
    /// full information about a particular dataset resource, use the Datasets: get method. This
    /// property is omitted when there are no datasets in the project.
    #[builder(setter(into))]
    #[serde(default)]
    pub datasets: ::std::vec::Vec<Datasets>,
    /// Output only. A hash value of the results page. You can use this property to determine if the
    /// page has changed since the last request.
    #[builder(setter(into))]
    #[serde(default)]
    pub etag: ::std::string::String,
    /// Output only. The resource type. This property always returns the value "bigquery#datasetList"
    #[builder(setter(into))]
    #[serde(default)]
    pub kind: ::std::string::String,
    /// A token that can be used to request the next results page. This property is omitted on the final
    /// results page.
    #[builder(setter(into))]
    #[serde(default)]
    pub next_page_token: ::std::string::String,
    /// A list of skipped locations that were unreachable. For more information about BigQuery
    /// locations, see: https://cloud.google.com/bigquery/docs/locations. Example: "europe-west5"
    #[builder(setter(into))]
    #[serde(default)]
    pub unreachable: ::std::vec::Vec<::std::string::String>,
}

#[derive(::serde::Deserialize, ::serde::Serialize, ::typed_builder::TypedBuilder, Debug, PartialEq, Clone)]
#[derive(Default)]
#[serde(rename_all = "camelCase")]
pub struct DatasetReference {
    /// Required. A unique ID for this dataset, without the project name. The ID must contain only
    /// letters (a-z, A-Z), numbers (0-9), or underscores (_). The maximum length is 1,024 characters.
    #[builder(setter(into))]
    #[serde(default)]
    pub dataset_id: ::std::string::String,
    /// Optional. The ID of the project containing this dataset.
    #[builder(setter(into))]
    #[serde(default)]
    pub project_id: ::std::option::Option<::std::string::String>,
}

/// Properties for the destination table.
#[derive(::serde::Deserialize, ::serde::Serialize, ::typed_builder::TypedBuilder, Debug, PartialEq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct DestinationTableProperties {
    /// Optional. The description for the destination table. This will only be used if the destination
    /// table is newly created. If the table already exists and a value different than the current
    /// description is provided, the job will fail.
    #[builder(setter(into))]
    #[serde(default)]
    pub description: ::std::option::Option<::std::string::String>,
    /// Internal use only.
    #[builder(setter(into))]
    pub expiration_time: ::timestamp::Timestamp,
    /// Optional. Friendly name for the destination table. If the table already exists, it should be
    /// same as the existing friendly name.
    #[builder(setter(into))]
    #[serde(default)]
    pub friendly_name: ::std::option::Option<::std::string::String>,
    /// Optional. The labels associated with this table. You can use these to organize and group your
    /// tables. This will only be used if the destination table is newly created. If the table already
    /// exists and labels are different than the current labels are provided, the job will fail.
    #[builder(setter(into))]
    #[serde(default)]
    pub labels: Labels,
}

/// Model evaluation metrics for dimensionality reduction models.
#[derive(::serde::Deserialize, ::serde::Serialize, ::typed_builder::TypedBuilder, Debug, PartialEq, Clone)]
#[derive(Default)]
#[serde(rename_all = "camelCase")]
pub struct DimensionalityReductionMetrics {
    /// Total percentage of variance explained by the selected principal components.
    #[builder(setter(into))]
    #[serde(default)]
    pub total_explained_variance_ratio: f64,
}

/// Detailed statistics for DML statements
#[derive(::serde::Deserialize, ::serde::Serialize, ::typed_builder::TypedBuilder, Debug, PartialEq, Clone)]
#[derive(Default)]
#[serde(rename_all = "camelCase")]
pub struct DmlStatistics {
    /// Output only. Number of deleted Rows. populated by DML DELETE, MERGE and TRUNCATE statements.
    #[builder(setter(into))]
    #[serde(with = "with::int64")]
    #[serde(default)]
    pub deleted_row_count: i64,
    /// Output only. Number of inserted Rows. Populated by DML INSERT and MERGE statements
    #[builder(setter(into))]
    #[serde(with = "with::int64")]
    #[serde(default)]
    pub inserted_row_count: i64,
    /// Output only. Number of updated Rows. Populated by DML UPDATE and MERGE statements.
    #[builder(setter(into))]
    #[serde(with = "with::int64")]
    #[serde(default)]
    pub updated_row_count: i64,
}

/// Discrete candidates of a double hyperparameter.
#[derive(::serde::Deserialize, ::serde::Serialize, ::typed_builder::TypedBuilder, Debug, PartialEq, Clone)]
#[derive(Default)]
#[serde(rename_all = "camelCase")]
pub struct DoubleCandidates {
    /// Candidates for the double parameter in increasing order.
    #[builder(setter(into))]
    #[serde(default)]
    pub candidates: ::std::vec::Vec<f64>,
}

/// Search space for a double hyperparameter.
#[derive(::serde::Deserialize, ::serde::Serialize, ::typed_builder::TypedBuilder, Debug, PartialEq, Clone)]
#[derive(Default)]
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

/// Range of a double hyperparameter.
#[derive(::serde::Deserialize, ::serde::Serialize, ::typed_builder::TypedBuilder, Debug, PartialEq, Clone)]
#[derive(Default)]
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

#[derive(::serde::Deserialize, ::serde::Serialize, ::typed_builder::TypedBuilder, Debug, PartialEq, Clone)]
#[derive(Default)]
#[serde(rename_all = "camelCase")]
pub struct EncryptionConfiguration {
    /// Optional. Describes the Cloud KMS encryption key that will be used to protect destination
    /// BigQuery table. The BigQuery Service Account associated with your project requires access to
    /// this encryption key.
    #[builder(setter(into))]
    #[serde(default)]
    pub kms_key_name: ::std::option::Option<::std::string::String>,
}

/// A single entry in the confusion matrix.
#[derive(::serde::Deserialize, ::serde::Serialize, ::typed_builder::TypedBuilder, Debug, PartialEq, Clone)]
#[derive(Default)]
#[serde(rename_all = "camelCase")]
pub struct Entry {
    /// Number of items being predicted as this label.
    #[builder(setter(into))]
    #[serde(with = "with::int64")]
    #[serde(default)]
    pub item_count: i64,
    /// The predicted label. For confidence_threshold > 0, we will also add an entry indicating the
    /// number of items under the confidence threshold.
    #[builder(setter(into))]
    #[serde(default)]
    pub predicted_label: ::std::string::String,
}

/// Error details.
#[derive(::serde::Deserialize, ::serde::Serialize, ::typed_builder::TypedBuilder, Debug, PartialEq, Clone)]
#[derive(Default)]
#[serde(rename_all = "camelCase")]
pub struct ErrorProto {
    /// Debugging information. This property is internal to Google and should not be used.
    #[builder(setter(into))]
    #[serde(default)]
    pub debug_info: ::std::string::String,
    /// Specifies where the error occurred, if present.
    #[builder(setter(into))]
    #[serde(default)]
    pub location: ::std::string::String,
    /// A human-readable description of the error.
    #[builder(setter(into))]
    #[serde(default)]
    pub message: ::std::string::String,
    /// A short error code that summarizes the error.
    #[builder(setter(into))]
    #[serde(default)]
    pub reason: ::std::string::String,
}

/// Evaluation metrics of a model. These are either computed on all training data or just the eval
/// data based on whether eval data was used during training. These are not present for imported
/// models.
#[derive(::serde::Deserialize, ::serde::Serialize, ::typed_builder::TypedBuilder, Debug, PartialEq, Clone)]
#[derive(Default)]
#[serde(rename_all = "camelCase")]
pub struct EvaluationMetrics {
    /// Populated for ARIMA models.
    #[builder(setter(into))]
    #[serde(default)]
    pub arima_forecasting_metrics: ArimaForecastingMetrics,
    /// Populated for binary classification/classifier models.
    #[builder(setter(into))]
    #[serde(default)]
    pub binary_classification_metrics: BinaryClassificationMetrics,
    /// Populated for clustering models.
    #[builder(setter(into))]
    #[serde(default)]
    pub clustering_metrics: ClusteringMetrics,
    /// Evaluation metrics when the model is a dimensionality reduction model, which currently includes
    /// PCA.
    #[builder(setter(into))]
    #[serde(default)]
    pub dimensionality_reduction_metrics: DimensionalityReductionMetrics,
    /// Populated for multi-class classification/classifier models.
    #[builder(setter(into))]
    #[serde(default)]
    pub multi_class_classification_metrics: MultiClassClassificationMetrics,
    /// Populated for implicit feedback type matrix factorization models.
    #[builder(setter(into))]
    #[serde(default)]
    pub ranking_metrics: RankingMetrics,
    /// Populated for regression models and explicit feedback type matrix factorization models.
    #[builder(setter(into))]
    #[serde(default)]
    pub regression_metrics: RegressionMetrics,
}

/// Output only. Compute mode for this stage.
#[derive(::serde::Deserialize, ::serde::Serialize, Debug, PartialEq, Clone)]
pub enum ComputeMode {
        /// This stage was processed using BigQuery slots.
    #[serde(rename = "BIGQUERY")]
    Bigquery,
        /// This stage was processed using BI Engine compute.
    #[serde(rename = "BI_ENGINE")]
    BiEngine,
}

/// A single stage of query execution.
#[derive(::serde::Deserialize, ::serde::Serialize, ::typed_builder::TypedBuilder, Debug, PartialEq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ExplainQueryStage {
    /// Number of parallel input segments completed.
    #[builder(setter(into))]
    #[serde(with = "with::int64")]
    #[serde(default)]
    pub completed_parallel_inputs: i64,
    /// Output only. Compute mode for this stage.
    #[builder(setter(into))]
    pub compute_mode: ComputeMode,
    /// Milliseconds the average shard spent on CPU-bound tasks.
    #[builder(setter(into))]
    #[serde(with = "with::int64")]
    #[serde(default)]
    pub compute_ms_avg: i64,
    /// Milliseconds the slowest shard spent on CPU-bound tasks.
    #[builder(setter(into))]
    #[serde(with = "with::int64")]
    #[serde(default)]
    pub compute_ms_max: i64,
    /// Relative amount of time the average shard spent on CPU-bound tasks.
    #[builder(setter(into))]
    #[serde(default)]
    pub compute_ratio_avg: f64,
    /// Relative amount of time the slowest shard spent on CPU-bound tasks.
    #[builder(setter(into))]
    #[serde(default)]
    pub compute_ratio_max: f64,
    /// Stage end time represented as milliseconds since the epoch.
    #[builder(setter(into))]
    #[serde(with = "with::int64")]
    #[serde(default)]
    pub end_ms: i64,
    /// Unique ID for the stage within the plan.
    #[builder(setter(into))]
    #[serde(with = "with::int64")]
    #[serde(default)]
    pub id: i64,
    /// IDs for stages that are inputs to this stage.
    #[builder(setter(into))]
    #[serde(default)]
    pub input_stages: ::std::vec::Vec<i64>,
    /// Human-readable name for the stage.
    #[builder(setter(into))]
    #[serde(default)]
    pub name: ::std::string::String,
    /// Number of parallel input segments to be processed
    #[builder(setter(into))]
    #[serde(with = "with::int64")]
    #[serde(default)]
    pub parallel_inputs: i64,
    /// Milliseconds the average shard spent reading input.
    #[builder(setter(into))]
    #[serde(with = "with::int64")]
    #[serde(default)]
    pub read_ms_avg: i64,
    /// Milliseconds the slowest shard spent reading input.
    #[builder(setter(into))]
    #[serde(with = "with::int64")]
    #[serde(default)]
    pub read_ms_max: i64,
    /// Relative amount of time the average shard spent reading input.
    #[builder(setter(into))]
    #[serde(default)]
    pub read_ratio_avg: f64,
    /// Relative amount of time the slowest shard spent reading input.
    #[builder(setter(into))]
    #[serde(default)]
    pub read_ratio_max: f64,
    /// Number of records read into the stage.
    #[builder(setter(into))]
    #[serde(with = "with::int64")]
    #[serde(default)]
    pub records_read: i64,
    /// Number of records written by the stage.
    #[builder(setter(into))]
    #[serde(with = "with::int64")]
    #[serde(default)]
    pub records_written: i64,
    /// Total number of bytes written to shuffle.
    #[builder(setter(into))]
    #[serde(with = "with::int64")]
    #[serde(default)]
    pub shuffle_output_bytes: i64,
    /// Total number of bytes written to shuffle and spilled to disk.
    #[builder(setter(into))]
    #[serde(with = "with::int64")]
    #[serde(default)]
    pub shuffle_output_bytes_spilled: i64,
    /// Slot-milliseconds used by the stage.
    #[builder(setter(into))]
    #[serde(with = "with::int64")]
    #[serde(default)]
    pub slot_ms: i64,
    /// Stage start time represented as milliseconds since the epoch.
    #[builder(setter(into))]
    #[serde(with = "with::int64")]
    #[serde(default)]
    pub start_ms: i64,
    /// Current status for this stage.
    #[builder(setter(into))]
    #[serde(default)]
    pub status: ::std::string::String,
    /// List of operations within the stage in dependency order (approximately chronological).
    #[builder(setter(into))]
    #[serde(default)]
    pub steps: ::std::vec::Vec<ExplainQueryStep>,
    /// Milliseconds the average shard spent waiting to be scheduled.
    #[builder(setter(into))]
    #[serde(with = "with::int64")]
    #[serde(default)]
    pub wait_ms_avg: i64,
    /// Milliseconds the slowest shard spent waiting to be scheduled.
    #[builder(setter(into))]
    #[serde(with = "with::int64")]
    #[serde(default)]
    pub wait_ms_max: i64,
    /// Relative amount of time the average shard spent waiting to be scheduled.
    #[builder(setter(into))]
    #[serde(default)]
    pub wait_ratio_avg: f64,
    /// Relative amount of time the slowest shard spent waiting to be scheduled.
    #[builder(setter(into))]
    #[serde(default)]
    pub wait_ratio_max: f64,
    /// Milliseconds the average shard spent on writing output.
    #[builder(setter(into))]
    #[serde(with = "with::int64")]
    #[serde(default)]
    pub write_ms_avg: i64,
    /// Milliseconds the slowest shard spent on writing output.
    #[builder(setter(into))]
    #[serde(with = "with::int64")]
    #[serde(default)]
    pub write_ms_max: i64,
    /// Relative amount of time the average shard spent on writing output.
    #[builder(setter(into))]
    #[serde(default)]
    pub write_ratio_avg: f64,
    /// Relative amount of time the slowest shard spent on writing output.
    #[builder(setter(into))]
    #[serde(default)]
    pub write_ratio_max: f64,
}

/// An operation within a stage.
#[derive(::serde::Deserialize, ::serde::Serialize, ::typed_builder::TypedBuilder, Debug, PartialEq, Clone)]
#[derive(Default)]
#[serde(rename_all = "camelCase")]
pub struct ExplainQueryStep {
    /// Machine-readable operation type.
    #[builder(setter(into))]
    #[serde(default)]
    pub kind: ::std::string::String,
    /// Human-readable description of the step(s).
    #[builder(setter(into))]
    #[serde(default)]
    pub substeps: ::std::vec::Vec<::std::string::String>,
}

/// Explanation for a single feature.
#[derive(::serde::Deserialize, ::serde::Serialize, ::typed_builder::TypedBuilder, Debug, PartialEq, Clone)]
#[derive(Default)]
#[serde(rename_all = "camelCase")]
pub struct Explanation {
    /// Attribution of feature.
    #[builder(setter(into))]
    #[serde(default)]
    pub attribution: f64,
    /// The full feature name. For non-numerical features, will be formatted like `.`. Overall size of
    /// feature name will always be truncated to first 120 characters.
    #[builder(setter(into))]
    #[serde(default)]
    pub feature_name: ::std::string::String,
}

/// Statistics for the EXPORT DATA statement as part of Query Job. EXTRACT JOB statistics are
/// populated in JobStatistics4.
#[derive(::serde::Deserialize, ::serde::Serialize, ::typed_builder::TypedBuilder, Debug, PartialEq, Clone)]
#[derive(Default)]
#[serde(rename_all = "camelCase")]
pub struct ExportDataStatistics {
    /// Number of destination files generated in case of EXPORT DATA statement only.
    #[builder(setter(into))]
    #[serde(with = "with::int64")]
    #[serde(default)]
    pub file_count: i64,
    /// [Alpha] Number of destination rows generated in case of EXPORT DATA statement only.
    #[builder(setter(into))]
    #[serde(with = "with::int64")]
    #[serde(default)]
    pub row_count: i64,
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
#[derive(::serde::Deserialize, ::serde::Serialize, ::typed_builder::TypedBuilder, Debug, PartialEq, Clone)]
#[derive(Default)]
#[serde(rename_all = "camelCase")]
pub struct Expr {
    /// Optional. Description of the expression. This is a longer text which describes the expression,
    /// e.g. when hovered over it in a UI.
    #[builder(setter(into))]
    #[serde(default)]
    pub description: ::std::option::Option<::std::string::String>,
    /// Textual representation of an expression in Common Expression Language syntax.
    #[builder(setter(into))]
    #[serde(default)]
    pub expression: ::std::string::String,
    /// Optional. String indicating the location of the expression for error reporting, e.g. a file name
    /// and a position in the file.
    #[builder(setter(into))]
    #[serde(default)]
    pub location: ::std::option::Option<::std::string::String>,
    /// Optional. Title for the expression, i.e. a short string describing its purpose. This can be used
    /// e.g. in UIs which allow to enter the expression.
    #[builder(setter(into))]
    #[serde(default)]
    pub title: ::std::option::Option<::std::string::String>,
}

#[derive(::serde::Deserialize, ::serde::Serialize, Debug, PartialEq, Clone)]
pub enum DecimalTargetTypes {
        /// Decimal values could be converted to NUMERIC type.
    #[serde(rename = "NUMERIC")]
    Numeric,
        /// Decimal values could be converted to BIGNUMERIC type.
    #[serde(rename = "BIGNUMERIC")]
    Bignumeric,
        /// Decimal values could be converted to STRING type.
    #[serde(rename = "STRING")]
    String,
}

/// Optional. Specifies how source URIs are interpreted for constructing the file set to load. By
/// default, source URIs are expanded against the underlying storage. You can also specify manifest
/// files to control how the file set is constructed. This option is only applicable to object
/// storage systems.
#[derive(::serde::Deserialize, ::serde::Serialize, Debug, PartialEq, Clone)]
pub enum FileSetSpecType {
        /// This option expands source URIs by listing files from the object store. It is the default
        /// behavior if FileSetSpecType is not set.
    #[serde(rename = "FILE_SET_SPEC_TYPE_FILE_SYSTEM_MATCH")]
    FileSetSpecTypeFileSystemMatch,
        /// This option indicates that the provided URIs are newline-delimited manifest files, with one
        /// URI per line. Wildcard URIs are not supported.
    #[serde(rename = "FILE_SET_SPEC_TYPE_NEW_LINE_DELIMITED_MANIFEST")]
    FileSetSpecTypeNewLineDelimitedManifest,
}

/// Optional. Load option to be used together with source_format newline-delimited JSON to indicate
/// that a variant of JSON is being loaded. To load newline-delimited GeoJSON, specify GEOJSON (and
/// source_format must be set to NEWLINE_DELIMITED_JSON).
#[derive(::serde::Deserialize, ::serde::Serialize, Debug, PartialEq, Clone)]
pub enum JsonExtension {
        /// Use GeoJSON variant of JSON. See https://tools.ietf.org/html/rfc7946.
    #[serde(rename = "GEOJSON")]
    Geojson,
}

/// Optional. Metadata Cache Mode for the table. Set this to enable caching of metadata from
/// external data source.
#[derive(::serde::Deserialize, ::serde::Serialize, Debug, PartialEq, Clone)]
pub enum MetadataCacheMode {
        /// Set this mode to trigger automatic background refresh of metadata cache from the external
        /// source. Queries will use the latest available cache version within the table's maxStaleness
        /// interval.
    #[serde(rename = "AUTOMATIC")]
    Automatic,
        /// Set this mode to enable triggering manual refresh of the metadata cache from external
        /// source. Queries will use the latest manually triggered cache version within the table's
        /// maxStaleness interval.
    #[serde(rename = "MANUAL")]
    Manual,
}

/// Optional. ObjectMetadata is used to create Object Tables. Object Tables contain a listing of
/// objects (with their metadata) found at the source_uris. If ObjectMetadata is set, source_format
/// should be omitted. Currently SIMPLE is the only supported Object Metadata type.
#[derive(::serde::Deserialize, ::serde::Serialize, Debug, PartialEq, Clone)]
pub enum ObjectMetadata {
        /// A synonym for `SIMPLE`.
    #[serde(rename = "DIRECTORY")]
    Directory,
        /// Directory listing of objects.
    #[serde(rename = "SIMPLE")]
    Simple,
}

#[derive(::serde::Deserialize, ::serde::Serialize, ::typed_builder::TypedBuilder, Debug, PartialEq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ExternalDataConfiguration {
    /// Try to detect schema and format options automatically. Any option specified explicitly will be
    /// honored.
    #[builder(setter(into))]
    #[serde(default)]
    pub autodetect: bool,
    /// Optional. Additional properties to set if sourceFormat is set to AVRO.
    #[builder(setter(into))]
    #[serde(default)]
    pub avro_options: AvroOptions,
    /// Optional. Additional options if sourceFormat is set to BIGTABLE.
    #[builder(setter(into))]
    #[serde(default)]
    pub bigtable_options: BigtableOptions,
    /// Optional. The compression type of the data source. Possible values include GZIP and NONE. The
    /// default value is NONE. This setting is ignored for Google Cloud Bigtable, Google Cloud Datastore
    /// backups, Avro, ORC and Parquet formats. An empty string is an invalid value.
    #[builder(setter(into))]
    #[serde(default)]
    pub compression: ::std::option::Option<::std::string::String>,
    /// Optional. The connection specifying the credentials to be used to read external storage, such as
    /// Azure Blob, Cloud Storage, or S3. The connection_id can have the form
    /// "<project\_id>.<location\_id>.<connection\_id>" or
    /// "projects/<project\_id>/locations/<location\_id>/connections/<connection\_id>".
    #[builder(setter(into))]
    #[serde(default)]
    pub connection_id: ::std::option::Option<::std::string::String>,
    /// Optional. Additional properties to set if sourceFormat is set to CSV.
    #[builder(setter(into))]
    #[serde(default)]
    pub csv_options: CsvOptions,
    /// Defines the list of possible SQL data types to which the source decimal values are converted.
    /// This list and the precision and the scale parameters of the decimal field determine the target
    /// type. In the order of NUMERIC, BIGNUMERIC, and STRING, a type is picked if it is in the
    /// specified list and if it supports the precision and the scale. STRING supports all precision and
    /// scale values. If none of the listed types supports the precision and the scale, the type
    /// supporting the widest range in the specified list is picked, and if a value exceeds the
    /// supported range when reading the data, an error will be thrown. Example: Suppose the value of
    /// this field is ["NUMERIC", "BIGNUMERIC"]. If (precision,scale) is: * (38,9) -> NUMERIC; * (39,9)
    /// -> BIGNUMERIC (NUMERIC cannot hold 30 integer digits); * (38,10) -> BIGNUMERIC (NUMERIC cannot
    /// hold 10 fractional digits); * (76,38) -> BIGNUMERIC; * (77,38) -> BIGNUMERIC (error if value
    /// exeeds supported range). This field cannot contain duplicate types. The order of the types in
    /// this field is ignored. For example, ["BIGNUMERIC", "NUMERIC"] is the same as ["NUMERIC",
    /// "BIGNUMERIC"] and NUMERIC always takes precedence over BIGNUMERIC. Defaults to ["NUMERIC",
    /// "STRING"] for ORC and ["NUMERIC"] for the other file formats.
    #[builder(setter(into))]
    #[serde(default)]
    pub decimal_target_types: ::std::vec::Vec<DecimalTargetTypes>,
    /// Optional. Specifies how source URIs are interpreted for constructing the file set to load. By
    /// default source URIs are expanded against the underlying storage. Other options include
    /// specifying manifest files. Only applicable to object storage systems.
    #[builder(setter(into))]
    pub file_set_spec_type: FileSetSpecType,
    /// Optional. Additional options if sourceFormat is set to GOOGLE_SHEETS.
    #[builder(setter(into))]
    #[serde(default)]
    pub google_sheets_options: ::std::option::Option<GoogleSheetsOptions>,
    /// Optional. When set, configures hive partitioning support. Not all storage formats support hive
    /// partitioning -- requesting hive partitioning on an unsupported format will lead to an error, as
    /// will providing an invalid specification.
    #[builder(setter(into))]
    #[serde(default)]
    pub hive_partitioning_options: ::std::option::Option<HivePartitioningOptions>,
    /// Optional. Indicates if BigQuery should allow extra values that are not represented in the table
    /// schema. If true, the extra values are ignored. If false, records with extra columns are treated
    /// as bad records, and if there are too many bad records, an invalid error is returned in the job
    /// result. The default value is false. The sourceFormat property determines what BigQuery treats as
    /// an extra value: CSV: Trailing columns JSON: Named values that don't match any column names
    /// Google Cloud Bigtable: This setting is ignored. Google Cloud Datastore backups: This setting is
    /// ignored. Avro: This setting is ignored. ORC: This setting is ignored. Parquet: This setting is
    /// ignored.
    #[builder(setter(into))]
    #[serde(default)]
    pub ignore_unknown_values: ::std::option::Option<bool>,
    /// Optional. Load option to be used together with source_format newline-delimited JSON to indicate
    /// that a variant of JSON is being loaded. To load newline-delimited GeoJSON, specify GEOJSON (and
    /// source_format must be set to NEWLINE_DELIMITED_JSON).
    #[builder(setter(into))]
    pub json_extension: JsonExtension,
    /// Optional. Additional properties to set if sourceFormat is set to JSON.
    #[builder(setter(into))]
    #[serde(default)]
    pub json_options: ::std::option::Option<JsonOptions>,
    /// Optional. The maximum number of bad records that BigQuery can ignore when reading data. If the
    /// number of bad records exceeds this value, an invalid error is returned in the job result. The
    /// default value is 0, which requires that all records are valid. This setting is ignored for
    /// Google Cloud Bigtable, Google Cloud Datastore backups, Avro, ORC and Parquet formats.
    #[builder(setter(into))]
    #[serde(default)]
    pub max_bad_records: ::std::option::Option<i64>,
    /// Optional. Metadata Cache Mode for the table. Set this to enable caching of metadata from
    /// external data source.
    #[builder(setter(into))]
    pub metadata_cache_mode: MetadataCacheMode,
    /// Optional. ObjectMetadata is used to create Object Tables. Object Tables contain a listing of
    /// objects (with their metadata) found at the source_uris. If ObjectMetadata is set, source_format
    /// should be omitted. Currently SIMPLE is the only supported Object Metadata type.
    #[builder(setter(into))]
    pub object_metadata: ObjectMetadata,
    /// Optional. Additional properties to set if sourceFormat is set to PARQUET.
    #[builder(setter(into))]
    #[serde(default)]
    pub parquet_options: ::std::option::Option<ParquetOptions>,
    /// Optional. When creating an external table, the user can provide a reference file with the table
    /// schema. This is enabled for the following formats: AVRO, PARQUET, ORC.
    #[builder(setter(into))]
    #[serde(default)]
    pub reference_file_schema_uri: ::std::option::Option<::std::string::String>,
    /// Optional. The schema for the data. Schema is required for CSV and JSON formats if autodetect is
    /// not on. Schema is disallowed for Google Cloud Bigtable, Cloud Datastore backups, Avro, ORC and
    /// Parquet formats.
    #[builder(setter(into))]
    #[serde(default)]
    pub schema: ::std::option::Option<TableSchema>,
    /// [Required] The data format. For CSV files, specify "CSV". For Google sheets, specify
    /// "GOOGLE_SHEETS". For newline-delimited JSON, specify "NEWLINE_DELIMITED_JSON". For Avro files,
    /// specify "AVRO". For Google Cloud Datastore backups, specify "DATASTORE_BACKUP". For Apache
    /// Iceberg tables, specify "ICEBERG". For ORC files, specify "ORC". For Parquet files, specify
    /// "PARQUET". [Beta] For Google Cloud Bigtable, specify "BIGTABLE".
    #[builder(setter(into))]
    #[serde(default)]
    pub source_format: ::std::string::String,
    /// [Required] The fully-qualified URIs that point to your data in Google Cloud. For Google Cloud
    /// Storage URIs: Each URI can contain one '*' wildcard character and it must come after the
    /// 'bucket' name. Size limits related to load jobs apply to external data sources. For Google Cloud
    /// Bigtable URIs: Exactly one URI can be specified and it has be a fully specified and valid HTTPS
    /// URL for a Google Cloud Bigtable table. For Google Cloud Datastore backups, exactly one URI can
    /// be specified. Also, the '*' wildcard character is not allowed.
    #[builder(setter(into))]
    #[serde(default)]
    pub source_uris: ::std::vec::Vec<::std::string::String>,
}

/// Configures the access a dataset defined in an external metadata storage.
#[derive(::serde::Deserialize, ::serde::Serialize, ::typed_builder::TypedBuilder, Debug, PartialEq, Clone)]
#[derive(Default)]
#[serde(rename_all = "camelCase")]
pub struct ExternalDatasetReference {
    /// Required. The connection id that is used to access the external_source. Format:
    /// projects/{project_id}/locations/{location_id}/connections/{connection_id}
    #[builder(setter(into))]
    #[serde(default)]
    pub connection: ::std::string::String,
    /// Required. External source that backs this dataset.
    #[builder(setter(into))]
    #[serde(default)]
    pub external_source: ::std::string::String,
}

/// The external service cost is a portion of the total cost, these costs are not additive with
/// total_bytes_billed. Moreover, this field only track external service costs that will show up as
/// BigQuery costs (e.g. training BigQuery ML job with google cloud CAIP or Automl Tables services),
/// not other costs which may be accrued by running the query (e.g. reading from Bigtable or Cloud
/// Storage). The external service costs with different billing sku (e.g. CAIP job is charged based
/// on VM usage) are converted to BigQuery billed_bytes and slot_ms with equivalent amount of US
/// dollars. Services may not directly correlate to these metrics, but these are the equivalents for
/// billing purposes. Output only.
#[derive(::serde::Deserialize, ::serde::Serialize, ::typed_builder::TypedBuilder, Debug, PartialEq, Clone)]
#[derive(Default)]
#[serde(rename_all = "camelCase")]
pub struct ExternalServiceCost {
    /// External service cost in terms of bigquery bytes billed.
    #[builder(setter(into))]
    #[serde(with = "with::int64")]
    #[serde(default)]
    pub bytes_billed: i64,
    /// External service cost in terms of bigquery bytes processed.
    #[builder(setter(into))]
    #[serde(with = "with::int64")]
    #[serde(default)]
    pub bytes_processed: i64,
    /// External service name.
    #[builder(setter(into))]
    #[serde(default)]
    pub external_service: ::std::string::String,
    /// Non-preemptable reserved slots used for external job. For example, reserved slots for Cloua AI
    /// Platform job are the VM usages converted to BigQuery slot with equivalent mount of price.
    #[builder(setter(into))]
    #[serde(with = "with::int64")]
    #[serde(default)]
    pub reserved_slot_count: i64,
    /// External service cost in terms of bigquery slot milliseconds.
    #[builder(setter(into))]
    #[serde(with = "with::int64")]
    #[serde(default)]
    pub slot_ms: i64,
}

/// Representative value of a single feature within the cluster.
#[derive(::serde::Deserialize, ::serde::Serialize, ::typed_builder::TypedBuilder, Debug, PartialEq, Clone)]
#[derive(Default)]
#[serde(rename_all = "camelCase")]
pub struct FeatureValue {
    /// The categorical feature value.
    #[builder(setter(into))]
    #[serde(default)]
    pub categorical_value: CategoricalValue,
    /// The feature column name.
    #[builder(setter(into))]
    #[serde(default)]
    pub feature_column: ::std::string::String,
    /// The numerical feature value. This is the centroid value for this feature.
    #[builder(setter(into))]
    #[serde(default)]
    pub numerical_value: f64,
}

/// Request message for `GetIamPolicy` method.
#[derive(::serde::Deserialize, ::serde::Serialize, ::typed_builder::TypedBuilder, Debug, PartialEq, Clone)]
#[derive(Default)]
#[serde(rename_all = "camelCase")]
pub struct GetIamPolicyRequest {
    /// OPTIONAL: A `GetPolicyOptions` object for specifying options to `GetIamPolicy`.
    #[builder(setter(into))]
    #[serde(default)]
    pub options: ::std::option::Option<GetPolicyOptions>,
}

/// Encapsulates settings provided to GetIamPolicy.
#[derive(::serde::Deserialize, ::serde::Serialize, ::typed_builder::TypedBuilder, Debug, PartialEq, Clone)]
#[derive(Default)]
#[serde(rename_all = "camelCase")]
pub struct GetPolicyOptions {
    /// Optional. The maximum policy version that will be used to format the policy. Valid values are 0,
    /// 1, and 3. Requests specifying an invalid value will be rejected. Requests for policies with any
    /// conditional role bindings must specify version 3. Policies with no conditional role bindings may
    /// specify any valid value or leave the field unset. The policy in the response might use the
    /// policy version that you specified, or it might use a lower policy version. For example, if you
    /// specify version 3, but the policy has no conditional role bindings, the response uses version 1.
    /// To learn which resources support conditions in their IAM policies, see the [IAM
    /// documentation](https://cloud.google.com/iam/help/conditions/resource-policies).
    #[builder(setter(into))]
    #[serde(default)]
    pub requested_policy_version: ::std::option::Option<i64>,
}

/// Response object of GetQueryResults.
#[derive(::serde::Deserialize, ::serde::Serialize, ::typed_builder::TypedBuilder, Debug, PartialEq, Clone)]
#[derive(Default)]
#[serde(rename_all = "camelCase")]
pub struct GetQueryResultsResponse {
    /// Whether the query result was fetched from the query cache.
    #[builder(setter(into))]
    #[serde(default)]
    pub cache_hit: bool,
    /// Output only. The first errors or warnings encountered during the running of the job. The final
    /// message includes the number of errors that caused the process to stop. Errors here do not
    /// necessarily mean that the job has completed or was unsuccessful. For more information about
    /// error messages, see [Error messages](https://cloud.google.com/bigquery/docs/error-messages).
    #[builder(setter(into))]
    #[serde(default)]
    pub errors: ::std::vec::Vec<ErrorProto>,
    /// A hash of this response.
    #[builder(setter(into))]
    #[serde(default)]
    pub etag: ::std::string::String,
    /// Whether the query has completed or not. If rows or totalRows are present, this will always be
    /// true. If this is false, totalRows will not be available.
    #[builder(setter(into))]
    #[serde(default)]
    pub job_complete: bool,
    /// Reference to the BigQuery Job that was created to run the query. This field will be present even
    /// if the original request timed out, in which case GetQueryResults can be used to read the results
    /// once the query has completed. Since this API only returns the first page of results, subsequent
    /// pages can be fetched via the same mechanism (GetQueryResults).
    #[builder(setter(into))]
    #[serde(default)]
    pub job_reference: JobReference,
    /// The resource type of the response.
    #[builder(setter(into))]
    #[serde(default)]
    pub kind: ::std::string::String,
    /// Output only. The number of rows affected by a DML statement. Present only for DML statements
    /// INSERT, UPDATE or DELETE.
    #[builder(setter(into))]
    #[serde(with = "with::int64")]
    #[serde(default)]
    pub num_dml_affected_rows: i64,
    /// A token used for paging results. When this token is non-empty, it indicates additional results
    /// are available.
    #[builder(setter(into))]
    #[serde(default)]
    pub page_token: ::std::string::String,
    /// An object with as many results as can be contained within the maximum permitted reply size. To
    /// get any additional rows, you can call GetQueryResults and specify the jobReference returned
    /// above. Present only when the query completes successfully. The REST-based representation of this
    /// data leverages a series of JSON f,v objects for indicating fields and values.
    #[builder(setter(into))]
    #[serde(default)]
    pub rows: ::std::vec::Vec<TableRow>,
    /// The schema of the results. Present only when the query completes successfully.
    #[builder(setter(into))]
    #[serde(default)]
    pub schema: TableSchema,
    /// The total number of bytes processed for this query.
    #[builder(setter(into))]
    #[serde(with = "with::int64")]
    #[serde(default)]
    pub total_bytes_processed: i64,
    /// The total number of rows in the complete query result set, which can be more than the number of
    /// rows in this single page of results. Present only when the query completes successfully.
    #[builder(setter(into))]
    #[serde(with = "with::uint64")]
    #[serde(default)]
    pub total_rows: u64,
}

/// Response object of GetServiceAccount
#[derive(::serde::Deserialize, ::serde::Serialize, ::typed_builder::TypedBuilder, Debug, PartialEq, Clone)]
#[derive(Default)]
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

/// Global explanations containing the top most important features after training.
#[derive(::serde::Deserialize, ::serde::Serialize, ::typed_builder::TypedBuilder, Debug, PartialEq, Clone)]
#[derive(Default)]
#[serde(rename_all = "camelCase")]
pub struct GlobalExplanation {
    /// Class label for this set of global explanations. Will be empty/null for binary logistic and
    /// linear regression models. Sorted alphabetically in descending order.
    #[builder(setter(into))]
    #[serde(default)]
    pub class_label: ::std::string::String,
    /// A list of the top global explanations. Sorted by absolute value of attribution in descending
    /// order.
    #[builder(setter(into))]
    #[serde(default)]
    pub explanations: ::std::vec::Vec<Explanation>,
}

/// Options specific to Google Sheets data sources.
#[derive(::serde::Deserialize, ::serde::Serialize, ::typed_builder::TypedBuilder, Debug, PartialEq, Clone)]
#[derive(Default)]
#[serde(rename_all = "camelCase")]
pub struct GoogleSheetsOptions {
    /// Optional. Range of a sheet to query from. Only used when non-empty. Typical format:
    /// sheet_name!top_left_cell_id:bottom_right_cell_id For example: sheet1!A1:B20
    #[builder(setter(into))]
    #[serde(default)]
    pub range: ::std::option::Option<::std::string::String>,
    /// Optional. The number of rows at the top of a sheet that BigQuery will skip when reading the
    /// data. The default value is 0. This property is useful if you have header rows that should be
    /// skipped. When autodetect is on, the behavior is the following: * skipLeadingRows unspecified -
    /// Autodetect tries to detect headers in the first row. If they are not detected, the row is read
    /// as data. Otherwise data is read starting from the second row. * skipLeadingRows is 0 - Instructs
    /// autodetect that there are no headers and data should be read starting from the first row. *
    /// skipLeadingRows = N > 0 - Autodetect skips N-1 rows and tries to detect headers in row N. If
    /// headers are not detected, row N is just skipped. Otherwise row N is used to extract column names
    /// for the detected schema.
    #[builder(setter(into))]
    #[serde(with = "with::int64::option")]
    #[serde(default)]
    pub skip_leading_rows: ::std::option::Option<i64>,
}

/// High cardinality join detailed information.
#[derive(::serde::Deserialize, ::serde::Serialize, ::typed_builder::TypedBuilder, Debug, PartialEq, Clone)]
#[derive(Default)]
#[serde(rename_all = "camelCase")]
pub struct HighCardinalityJoin {
    /// Output only. Count of left input rows.
    #[builder(setter(into))]
    #[serde(with = "with::int64")]
    #[serde(default)]
    pub left_rows: i64,
    /// Output only. Count of the output rows.
    #[builder(setter(into))]
    #[serde(with = "with::int64")]
    #[serde(default)]
    pub output_rows: i64,
    /// Output only. Count of right input rows.
    #[builder(setter(into))]
    #[serde(with = "with::int64")]
    #[serde(default)]
    pub right_rows: i64,
    /// Output only. The index of the join operator in the ExplainQueryStep lists.
    #[builder(setter(into))]
    #[serde(default)]
    pub step_index: i64,
}

/// Options for configuring hive partitioning detect.
#[derive(::serde::Deserialize, ::serde::Serialize, ::typed_builder::TypedBuilder, Debug, PartialEq, Clone)]
#[derive(Default)]
#[serde(rename_all = "camelCase")]
pub struct HivePartitioningOptions {
    /// Output only. For permanent external tables, this field is populated with the hive partition keys
    /// in the order they were inferred. The types of the partition keys can be deduced by checking the
    /// table schema (which will include the partition keys). Not every API will populate this field in
    /// the output. For example, Tables.Get will populate it, but Tables.List will not contain this
    /// field.
    #[builder(setter(into))]
    #[serde(default)]
    pub fields: ::std::vec::Vec<::std::string::String>,
    /// Optional. When set, what mode of hive partitioning to use when reading data. The following modes
    /// are supported: * AUTO: automatically infer partition key name(s) and type(s). * STRINGS:
    /// automatically infer partition key name(s). All types are strings. * CUSTOM: partition key schema
    /// is encoded in the source URI prefix. Not all storage formats support hive partitioning.
    /// Requesting hive partitioning on an unsupported format will lead to an error. Currently supported
    /// formats are: JSON, CSV, ORC, Avro and Parquet.
    #[builder(setter(into))]
    #[serde(default)]
    pub mode: ::std::option::Option<::std::string::String>,
    /// Optional. If set to true, queries over this table require a partition filter that can be used
    /// for partition elimination to be specified. Note that this field should only be true when
    /// creating a permanent external table or querying a temporary external table. Hive-partitioned
    /// loads with require_partition_filter explicitly set to true will fail.
    #[builder(setter(into))]
    #[serde(default)]
    pub require_partition_filter: ::std::option::Option<bool>,
    /// Optional. When hive partition detection is requested, a common prefix for all source uris must
    /// be required. The prefix must end immediately before the partition key encoding begins. For
    /// example, consider files following this data layout:
    /// gs://bucket/path_to_table/dt=2019-06-01/country=USA/id=7/file.avro
    /// gs://bucket/path_to_table/dt=2019-05-31/country=CA/id=3/file.avro When hive partitioning is
    /// requested with either AUTO or STRINGS detection, the common prefix can be either of
    /// gs://bucket/path_to_table or gs://bucket/path_to_table/. CUSTOM detection requires encoding the
    /// partitioning schema immediately after the common prefix. For CUSTOM, any of *
    /// gs://bucket/path_to_table/{dt:DATE}/{country:STRING}/{id:INTEGER} *
    /// gs://bucket/path_to_table/{dt:STRING}/{country:STRING}/{id:INTEGER} *
    /// gs://bucket/path_to_table/{dt:DATE}/{country:STRING}/{id:STRING} would all be valid source URI
    /// prefixes.
    #[builder(setter(into))]
    #[serde(default)]
    pub source_uri_prefix: ::std::option::Option<::std::string::String>,
}

/// Hyperparameter search spaces. These should be a subset of training_options.
#[derive(::serde::Deserialize, ::serde::Serialize, ::typed_builder::TypedBuilder, Debug, PartialEq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct HparamSearchSpaces {
    /// Activation functions of neural network models.
    #[builder(setter(into))]
    #[serde(default)]
    pub activation_fn: StringHparamSearchSpace,
    /// Mini batch sample size.
    #[builder(setter(into))]
    pub batch_size: IntHparamSearchSpace,
    /// Booster type for boosted tree models.
    #[builder(setter(into))]
    #[serde(default)]
    pub booster_type: StringHparamSearchSpace,
    /// Subsample ratio of columns for each level for boosted tree models.
    #[builder(setter(into))]
    pub colsample_bylevel: DoubleHparamSearchSpace,
    /// Subsample ratio of columns for each node(split) for boosted tree models.
    #[builder(setter(into))]
    pub colsample_bynode: DoubleHparamSearchSpace,
    /// Subsample ratio of columns when constructing each tree for boosted tree models.
    #[builder(setter(into))]
    pub colsample_bytree: DoubleHparamSearchSpace,
    /// Dart normalization type for boosted tree models.
    #[builder(setter(into))]
    #[serde(default)]
    pub dart_normalize_type: StringHparamSearchSpace,
    /// Dropout probability for dnn model training and boosted tree models using dart booster.
    #[builder(setter(into))]
    pub dropout: DoubleHparamSearchSpace,
    /// Hidden units for neural network models.
    #[builder(setter(into))]
    #[serde(default)]
    pub hidden_units: IntArrayHparamSearchSpace,
    /// L1 regularization coefficient.
    #[builder(setter(into))]
    pub l_1_reg: DoubleHparamSearchSpace,
    /// L2 regularization coefficient.
    #[builder(setter(into))]
    pub l_2_reg: DoubleHparamSearchSpace,
    /// Learning rate of training jobs.
    #[builder(setter(into))]
    pub learn_rate: DoubleHparamSearchSpace,
    /// Maximum depth of a tree for boosted tree models.
    #[builder(setter(into))]
    pub max_tree_depth: IntHparamSearchSpace,
    /// Minimum split loss for boosted tree models.
    #[builder(setter(into))]
    pub min_split_loss: DoubleHparamSearchSpace,
    /// Minimum sum of instance weight needed in a child for boosted tree models.
    #[builder(setter(into))]
    pub min_tree_child_weight: IntHparamSearchSpace,
    /// Number of clusters for k-means.
    #[builder(setter(into))]
    pub num_clusters: IntHparamSearchSpace,
    /// Number of latent factors to train on.
    #[builder(setter(into))]
    pub num_factors: IntHparamSearchSpace,
    /// Number of parallel trees for boosted tree models.
    #[builder(setter(into))]
    pub num_parallel_tree: IntHparamSearchSpace,
    /// Optimizer of TF models.
    #[builder(setter(into))]
    #[serde(default)]
    pub optimizer: StringHparamSearchSpace,
    /// Subsample the training data to grow tree to prevent overfitting for boosted tree models.
    #[builder(setter(into))]
    pub subsample: DoubleHparamSearchSpace,
    /// Tree construction algorithm for boosted tree models.
    #[builder(setter(into))]
    #[serde(default)]
    pub tree_method: StringHparamSearchSpace,
    /// Hyperparameter for matrix factoration when implicit feedback type is specified.
    #[builder(setter(into))]
    pub wals_alpha: DoubleHparamSearchSpace,
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
#[derive(::serde::Deserialize, ::serde::Serialize, ::typed_builder::TypedBuilder, Debug, PartialEq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct HparamTuningTrial {
    /// Ending time of the trial.
    #[builder(setter(into))]
    #[serde(with = "with::int64")]
    #[serde(default)]
    pub end_time_ms: i64,
    /// Error message for FAILED and INFEASIBLE trial.
    #[builder(setter(into))]
    #[serde(default)]
    pub error_message: ::std::string::String,
    /// Loss computed on the eval data at the end of trial.
    #[builder(setter(into))]
    #[serde(default)]
    pub eval_loss: f64,
    /// Evaluation metrics of this trial calculated on the test data. Empty in Job API.
    #[builder(setter(into))]
    pub evaluation_metrics: EvaluationMetrics,
    /// Hyperparameter tuning evaluation metrics of this trial calculated on the eval data. Unlike
    /// evaluation_metrics, only the fields corresponding to the hparam_tuning_objectives are set.
    #[builder(setter(into))]
    pub hparam_tuning_evaluation_metrics: EvaluationMetrics,
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
    /// Loss computed on the training data at the end of trial.
    #[builder(setter(into))]
    #[serde(default)]
    pub training_loss: f64,
    /// 1-based index of the trial.
    #[builder(setter(into))]
    #[serde(with = "with::int64")]
    #[serde(default)]
    pub trial_id: i64,
}

/// Reason about why no search index was used in the search query (or sub-query).
#[derive(::serde::Deserialize, ::serde::Serialize, ::typed_builder::TypedBuilder, Debug, PartialEq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct IndexUnusedReason {
    /// Specifies the base table involved in the reason that no search index was used.
    #[builder(setter(into))]
    #[serde(default)]
    pub base_table: TableReference,
    /// Specifies the high-level reason for the scenario when no search index was used.
    #[builder(setter(into))]
    pub code: Code,
    /// Specifies the name of the unused search index, if available.
    #[builder(setter(into))]
    #[serde(default)]
    pub index_name: ::std::string::String,
    /// Free form human-readable reason for the scenario when no search index was used.
    #[builder(setter(into))]
    #[serde(default)]
    pub message: ::std::string::String,
}

/// Details about the input data change insight.
#[derive(::serde::Deserialize, ::serde::Serialize, ::typed_builder::TypedBuilder, Debug, PartialEq, Clone)]
#[derive(Default)]
#[serde(rename_all = "camelCase")]
pub struct InputDataChange {
    /// Output only. Records read difference percentage compared to a previous run.
    #[builder(setter(into))]
    #[serde(default)]
    pub records_read_diff_percentage: f64,
}

/// An array of int.
#[derive(::serde::Deserialize, ::serde::Serialize, ::typed_builder::TypedBuilder, Debug, PartialEq, Clone)]
#[derive(Default)]
#[serde(rename_all = "camelCase")]
pub struct IntArray {
    /// Elements in the int array.
    #[builder(setter(into))]
    #[serde(default)]
    pub elements: ::std::vec::Vec<i64>,
}

/// Search space for int array.
#[derive(::serde::Deserialize, ::serde::Serialize, ::typed_builder::TypedBuilder, Debug, PartialEq, Clone)]
#[derive(Default)]
#[serde(rename_all = "camelCase")]
pub struct IntArrayHparamSearchSpace {
    /// Candidates for the int array parameter.
    #[builder(setter(into))]
    #[serde(default)]
    pub candidates: ::std::vec::Vec<IntArray>,
}

/// Discrete candidates of an int hyperparameter.
#[derive(::serde::Deserialize, ::serde::Serialize, ::typed_builder::TypedBuilder, Debug, PartialEq, Clone)]
#[derive(Default)]
#[serde(rename_all = "camelCase")]
pub struct IntCandidates {
    /// Candidates for the int parameter in increasing order.
    #[builder(setter(into))]
    #[serde(default)]
    pub candidates: ::std::vec::Vec<i64>,
}

/// Search space for an int hyperparameter.
#[derive(::serde::Deserialize, ::serde::Serialize, ::typed_builder::TypedBuilder, Debug, PartialEq, Clone)]
#[derive(Default)]
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

/// Range of an int hyperparameter.
#[derive(::serde::Deserialize, ::serde::Serialize, ::typed_builder::TypedBuilder, Debug, PartialEq, Clone)]
#[derive(Default)]
#[serde(rename_all = "camelCase")]
pub struct IntRange {
    /// Max value of the int parameter.
    #[builder(setter(into))]
    #[serde(with = "with::int64")]
    #[serde(default)]
    pub max: i64,
    /// Min value of the int parameter.
    #[builder(setter(into))]
    #[serde(with = "with::int64")]
    #[serde(default)]
    pub min: i64,
}

/// Information about a single iteration of the training run.
#[derive(::serde::Deserialize, ::serde::Serialize, ::typed_builder::TypedBuilder, Debug, PartialEq, Clone)]
#[derive(Default)]
#[serde(rename_all = "camelCase")]
pub struct IterationResult {
    /// Arima result.
    #[builder(setter(into))]
    #[serde(default)]
    pub arima_result: ArimaResult,
    /// Information about top clusters for clustering models.
    #[builder(setter(into))]
    #[serde(default)]
    pub cluster_infos: ::std::vec::Vec<ClusterInfo>,
    /// Time taken to run the iteration in milliseconds.
    #[builder(setter(into))]
    #[serde(with = "with::int64")]
    #[serde(default)]
    pub duration_ms: i64,
    /// Loss computed on the eval data at the end of iteration.
    #[builder(setter(into))]
    #[serde(default)]
    pub eval_loss: f64,
    /// Index of the iteration, 0 based.
    #[builder(setter(into))]
    #[serde(default)]
    pub index: i64,
    /// Learn rate used for this iteration.
    #[builder(setter(into))]
    #[serde(default)]
    pub learn_rate: f64,
    /// The information of the principal components.
    #[builder(setter(into))]
    #[serde(default)]
    pub principal_component_infos: ::std::vec::Vec<PrincipalComponentInfo>,
    /// Loss computed on the training data at the end of iteration.
    #[builder(setter(into))]
    #[serde(default)]
    pub training_loss: f64,
}

#[derive(::serde::Deserialize, ::serde::Serialize, ::typed_builder::TypedBuilder, Debug, PartialEq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Job {
    /// Required. Describes the job configuration.
    #[builder(setter(into))]
    pub configuration: JobConfiguration,
    /// Output only. A hash of this resource.
    #[builder(setter(into))]
    #[serde(default)]
    pub etag: ::std::string::String,
    /// Output only. Opaque ID field of the job.
    #[builder(setter(into))]
    #[serde(default)]
    pub id: ::std::string::String,
    /// Output only. If set, it provides the reason why a Job was created. If not set, it should be
    /// treated as the default: REQUESTED. This feature is not yet available. Jobs will always be
    /// created.
    #[builder(setter(into))]
    pub job_creation_reason: JobCreationReason,
    /// Optional. Reference describing the unique-per-user name of the job.
    #[builder(setter(into))]
    #[serde(default)]
    pub job_reference: ::std::option::Option<JobReference>,
    /// Output only. The type of the resource.
    #[builder(setter(into))]
    #[serde(default)]
    pub kind: ::std::string::String,
    /// Output only. [Full-projection-only] String representation of identity of requesting party.
    /// Populated for both first- and third-party identities. Only present for APIs that support
    /// third-party identities.
    #[builder(setter(into))]
    #[serde(default)]
    pub principal_subject: ::std::string::String,
    /// Output only. A URL that can be used to access the resource again.
    #[builder(setter(into))]
    #[serde(default)]
    pub self_link: ::std::string::String,
    /// Output only. Information about the job, including starting time and ending time of the job.
    #[builder(setter(into))]
    pub statistics: JobStatistics,
    /// Output only. The status of this job. Examine this value when polling an asynchronous job to see
    /// if the job is complete.
    #[builder(setter(into))]
    #[serde(default)]
    pub status: JobStatus,
    /// Output only. Email address of the user who ran the job.
    #[builder(setter(into))]
    #[serde(default)]
    pub user_email: ::std::string::String,
}

/// Describes format of a jobs cancellation response.
#[derive(::serde::Deserialize, ::serde::Serialize, ::typed_builder::TypedBuilder, Debug, PartialEq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct JobCancelResponse {
    /// The final state of the job.
    #[builder(setter(into))]
    pub job: Job,
    /// The resource type of the response.
    #[builder(setter(into))]
    #[serde(default)]
    pub kind: ::std::string::String,
}

#[derive(::serde::Deserialize, ::serde::Serialize, ::typed_builder::TypedBuilder, Debug, PartialEq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct JobConfiguration {
    /// [Pick one] Copies a table.
    #[builder(setter(into))]
    pub copy: JobConfigurationTableCopy,
    /// Optional. If set, don't actually run this job. A valid query will return a mostly empty response
    /// with some processing statistics, while an invalid query will return the same error it would if
    /// it wasn't a dry run. Behavior of non-query jobs is undefined.
    #[builder(setter(into))]
    #[serde(default)]
    pub dry_run: ::std::option::Option<bool>,
    /// [Pick one] Configures an extract job.
    #[builder(setter(into))]
    pub extract: JobConfigurationExtract,
    /// Optional. Job timeout in milliseconds. If this time limit is exceeded, BigQuery might attempt to
    /// stop the job.
    #[builder(setter(into))]
    #[serde(with = "with::int64::option")]
    #[serde(default)]
    pub job_timeout_ms: ::std::option::Option<i64>,
    /// Output only. The type of the job. Can be QUERY, LOAD, EXTRACT, COPY or UNKNOWN.
    #[builder(setter(into))]
    #[serde(default)]
    pub job_type: ::std::string::String,
    /// The labels associated with this job. You can use these to organize and group your jobs. Label
    /// keys and values can be no longer than 63 characters, can only contain lowercase letters, numeric
    /// characters, underscores and dashes. International characters are allowed. Label values are
    /// optional. Label keys must start with a letter and each label in the list must have a different
    /// key.
    #[builder(setter(into))]
    #[serde(default)]
    pub labels: Labels,
    /// [Pick one] Configures a load job.
    #[builder(setter(into))]
    pub load: JobConfigurationLoad,
    /// [Pick one] Configures a query job.
    #[builder(setter(into))]
    pub query: JobConfigurationQuery,
}

/// JobConfigurationExtract configures a job that exports data from a BigQuery table into Google
/// Cloud Storage.
#[derive(::serde::Deserialize, ::serde::Serialize, ::typed_builder::TypedBuilder, Debug, PartialEq, Clone)]
#[derive(Default)]
#[serde(rename_all = "camelCase")]
pub struct JobConfigurationExtract {
    /// Optional. The compression type to use for exported files. Possible values include DEFLATE, GZIP,
    /// NONE, SNAPPY, and ZSTD. The default value is NONE. Not all compression formats are support for
    /// all file formats. DEFLATE is only supported for Avro. ZSTD is only supported for Parquet. Not
    /// applicable when extracting models.
    #[builder(setter(into))]
    #[serde(default)]
    pub compression: ::std::option::Option<::std::string::String>,
    /// Optional. The exported file format. Possible values include CSV, NEWLINE_DELIMITED_JSON,
    /// PARQUET, or AVRO for tables and ML_TF_SAVED_MODEL or ML_XGBOOST_BOOSTER for models. The default
    /// value for tables is CSV. Tables with nested or repeated fields cannot be exported as CSV. The
    /// default value for models is ML_TF_SAVED_MODEL.
    #[builder(setter(into))]
    #[serde(default)]
    pub destination_format: ::std::option::Option<::std::string::String>,
    /// [Pick one] DEPRECATED: Use destinationUris instead, passing only one URI as necessary. The
    /// fully-qualified Google Cloud Storage URI where the extracted table should be written.
    #[builder(setter(into))]
    #[serde(default)]
    pub destination_uri: ::std::string::String,
    /// [Pick one] A list of fully-qualified Google Cloud Storage URIs where the extracted table should
    /// be written.
    #[builder(setter(into))]
    #[serde(default)]
    pub destination_uris: ::std::vec::Vec<::std::string::String>,
    /// Optional. When extracting data in CSV format, this defines the delimiter to use between fields
    /// in the exported data. Default is ','. Not applicable when extracting models.
    #[builder(setter(into))]
    #[serde(default)]
    pub field_delimiter: ::std::option::Option<::std::string::String>,
    /// Optional. Model extract options only applicable when extracting models.
    #[builder(setter(into))]
    #[serde(default)]
    pub model_extract_options: ::std::option::Option<ModelExtractOptions>,
    /// Optional. Whether to print out a header row in the results. Default is true. Not applicable when
    /// extracting models.
    #[builder(setter(into))]
    #[serde(default)]
    pub print_header: ::std::option::Option<bool>,
    /// A reference to the model being exported.
    #[builder(setter(into))]
    #[serde(default)]
    pub source_model: ModelReference,
    /// A reference to the table being exported.
    #[builder(setter(into))]
    #[serde(default)]
    pub source_table: TableReference,
    /// Whether to use logical types when extracting to AVRO format. Not applicable when extracting
    /// models.
    #[builder(setter(into))]
    #[serde(default)]
    pub use_avro_logical_types: bool,
}

/// JobConfigurationLoad contains the configuration properties for loading data into a destination
/// table.
#[derive(::serde::Deserialize, ::serde::Serialize, ::typed_builder::TypedBuilder, Debug, PartialEq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct JobConfigurationLoad {
    /// Optional. Accept rows that are missing trailing optional columns. The missing values are treated
    /// as nulls. If false, records with missing trailing columns are treated as bad records, and if
    /// there are too many bad records, an invalid error is returned in the job result. The default
    /// value is false. Only applicable to CSV, ignored for other formats.
    #[builder(setter(into))]
    #[serde(default)]
    pub allow_jagged_rows: ::std::option::Option<bool>,
    /// Indicates if BigQuery should allow quoted data sections that contain newline characters in a CSV
    /// file. The default value is false.
    #[builder(setter(into))]
    #[serde(default)]
    pub allow_quoted_newlines: bool,
    /// Optional. Indicates if we should automatically infer the options and schema for CSV and JSON
    /// sources.
    #[builder(setter(into))]
    #[serde(default)]
    pub autodetect: ::std::option::Option<bool>,
    /// Clustering specification for the destination table.
    #[builder(setter(into))]
    #[serde(default)]
    pub clustering: Clustering,
    /// Optional. Connection properties which can modify the load job behavior. Currently, only the
    /// 'session_id' connection property is supported, and is used to resolve _SESSION appearing as the
    /// dataset id.
    #[builder(setter(into))]
    #[serde(default)]
    pub connection_properties: ::std::vec::Vec<ConnectionProperty>,
    /// Optional. [Experimental] Configures the load job to only copy files to the destination BigLake
    /// managed table with an external storage_uri, without reading file content and writing them to new
    /// files. Copying files only is supported when: * source_uris are in the same external storage
    /// system as the destination table but they do not overlap with storage_uri of the destination
    /// table. * source_format is the same file format as the destination table. * destination_table is
    /// an existing BigLake managed table. Its schema does not have default value expression. It schema
    /// does not have type parameters other than precision and scale. * No options other than the above
    /// are specified.
    #[builder(setter(into))]
    #[serde(default)]
    pub copy_files_only: ::std::option::Option<bool>,
    /// Optional. Specifies whether the job is allowed to create new tables. The following values are
    /// supported: * CREATE_IF_NEEDED: If the table does not exist, BigQuery creates the table. *
    /// CREATE_NEVER: The table must already exist. If it does not, a 'notFound' error is returned in
    /// the job result. The default value is CREATE_IF_NEEDED. Creation, truncation and append actions
    /// occur as one atomic update upon job completion.
    #[builder(setter(into))]
    #[serde(default)]
    pub create_disposition: ::std::option::Option<::std::string::String>,
    /// Optional. If this property is true, the job creates a new session using a randomly generated
    /// session_id. To continue using a created session with subsequent queries, pass the existing
    /// session identifier as a `ConnectionProperty` value. The session identifier is returned as part
    /// of the `SessionInfo` message within the query statistics. The new session's location will be set
    /// to `Job.JobReference.location` if it is present, otherwise it's set to the default location
    /// based on existing routing logic.
    #[builder(setter(into))]
    #[serde(default)]
    pub create_session: ::std::option::Option<bool>,
    /// Defines the list of possible SQL data types to which the source decimal values are converted.
    /// This list and the precision and the scale parameters of the decimal field determine the target
    /// type. In the order of NUMERIC, BIGNUMERIC, and STRING, a type is picked if it is in the
    /// specified list and if it supports the precision and the scale. STRING supports all precision and
    /// scale values. If none of the listed types supports the precision and the scale, the type
    /// supporting the widest range in the specified list is picked, and if a value exceeds the
    /// supported range when reading the data, an error will be thrown. Example: Suppose the value of
    /// this field is ["NUMERIC", "BIGNUMERIC"]. If (precision,scale) is: * (38,9) -> NUMERIC; * (39,9)
    /// -> BIGNUMERIC (NUMERIC cannot hold 30 integer digits); * (38,10) -> BIGNUMERIC (NUMERIC cannot
    /// hold 10 fractional digits); * (76,38) -> BIGNUMERIC; * (77,38) -> BIGNUMERIC (error if value
    /// exeeds supported range). This field cannot contain duplicate types. The order of the types in
    /// this field is ignored. For example, ["BIGNUMERIC", "NUMERIC"] is the same as ["NUMERIC",
    /// "BIGNUMERIC"] and NUMERIC always takes precedence over BIGNUMERIC. Defaults to ["NUMERIC",
    /// "STRING"] for ORC and ["NUMERIC"] for the other file formats.
    #[builder(setter(into))]
    #[serde(default)]
    pub decimal_target_types: ::std::vec::Vec<DecimalTargetTypes>,
    /// Custom encryption configuration (e.g., Cloud KMS keys)
    #[builder(setter(into))]
    #[serde(default)]
    pub destination_encryption_configuration: EncryptionConfiguration,
    /// [Required] The destination table to load the data into.
    #[builder(setter(into))]
    #[serde(default)]
    pub destination_table: TableReference,
    /// Optional. [Experimental] Properties with which to create the destination table if it is new.
    #[builder(setter(into))]
    pub destination_table_properties: DestinationTableProperties,
    /// Optional. The character encoding of the data. The supported values are UTF-8, ISO-8859-1,
    /// UTF-16BE, UTF-16LE, UTF-32BE, and UTF-32LE. The default value is UTF-8. BigQuery decodes the
    /// data after the raw, binary data has been split using the values of the `quote` and
    /// `fieldDelimiter` properties. If you don't specify an encoding, or if you specify a UTF-8
    /// encoding when the CSV file is not UTF-8 encoded, BigQuery attempts to convert the data to UTF-8.
    /// Generally, your data loads successfully, but it may not match byte-for-byte what you expect. To
    /// avoid this, specify the correct encoding by using the `--encoding` flag. If BigQuery can't
    /// convert a character other than the ASCII `0` character, BigQuery converts the character to the
    /// standard Unicode replacement character: �.
    #[builder(setter(into))]
    #[serde(default)]
    pub encoding: ::std::option::Option<::std::string::String>,
    /// Optional. The separator character for fields in a CSV file. The separator is interpreted as a
    /// single byte. For files encoded in ISO-8859-1, any single character can be used as a separator.
    /// For files encoded in UTF-8, characters represented in decimal range 1-127 (U+0001-U+007F) can be
    /// used without any modification. UTF-8 characters encoded with multiple bytes (i.e. U+0080 and
    /// above) will have only the first byte used for separating fields. The remaining bytes will be
    /// treated as a part of the field. BigQuery also supports the escape sequence "\t" (U+0009) to
    /// specify a tab separator. The default value is comma (",", U+002C).
    #[builder(setter(into))]
    #[serde(default)]
    pub field_delimiter: ::std::option::Option<::std::string::String>,
    /// Optional. Specifies how source URIs are interpreted for constructing the file set to load. By
    /// default, source URIs are expanded against the underlying storage. You can also specify manifest
    /// files to control how the file set is constructed. This option is only applicable to object
    /// storage systems.
    #[builder(setter(into))]
    pub file_set_spec_type: FileSetSpecType,
    /// Optional. When set, configures hive partitioning support. Not all storage formats support hive
    /// partitioning -- requesting hive partitioning on an unsupported format will lead to an error, as
    /// will providing an invalid specification.
    #[builder(setter(into))]
    #[serde(default)]
    pub hive_partitioning_options: HivePartitioningOptions,
    /// Optional. Indicates if BigQuery should allow extra values that are not represented in the table
    /// schema. If true, the extra values are ignored. If false, records with extra columns are treated
    /// as bad records, and if there are too many bad records, an invalid error is returned in the job
    /// result. The default value is false. The sourceFormat property determines what BigQuery treats as
    /// an extra value: CSV: Trailing columns JSON: Named values that don't match any column names in
    /// the table schema Avro, Parquet, ORC: Fields in the file schema that don't exist in the table
    /// schema.
    #[builder(setter(into))]
    #[serde(default)]
    pub ignore_unknown_values: ::std::option::Option<bool>,
    /// Optional. Load option to be used together with source_format newline-delimited JSON to indicate
    /// that a variant of JSON is being loaded. To load newline-delimited GeoJSON, specify GEOJSON (and
    /// source_format must be set to NEWLINE_DELIMITED_JSON).
    #[builder(setter(into))]
    pub json_extension: JsonExtension,
    /// Optional. The maximum number of bad records that BigQuery can ignore when running the job. If
    /// the number of bad records exceeds this value, an invalid error is returned in the job result.
    /// The default value is 0, which requires that all records are valid. This is only supported for
    /// CSV and NEWLINE_DELIMITED_JSON file formats.
    #[builder(setter(into))]
    #[serde(default)]
    pub max_bad_records: ::std::option::Option<i64>,
    /// Optional. Specifies a string that represents a null value in a CSV file. For example, if you
    /// specify "\N", BigQuery interprets "\N" as a null value when loading a CSV file. The default
    /// value is the empty string. If you set this property to a custom value, BigQuery throws an error
    /// if an empty string is present for all data types except for STRING and BYTE. For STRING and BYTE
    /// columns, BigQuery interprets the empty string as an empty value.
    #[builder(setter(into))]
    #[serde(default)]
    pub null_marker: ::std::option::Option<::std::string::String>,
    /// Optional. Additional properties to set if sourceFormat is set to PARQUET.
    #[builder(setter(into))]
    #[serde(default)]
    pub parquet_options: ::std::option::Option<ParquetOptions>,
    /// Optional. When sourceFormat is set to "CSV", this indicates whether the embedded ASCII control
    /// characters (the first 32 characters in the ASCII-table, from '\x00' to '\x1F') are preserved.
    #[builder(setter(into))]
    #[serde(default)]
    pub preserve_ascii_control_characters: ::std::option::Option<bool>,
    /// If sourceFormat is set to "DATASTORE_BACKUP", indicates which entity properties to load into
    /// BigQuery from a Cloud Datastore backup. Property names are case sensitive and must be top-level
    /// properties. If no properties are specified, BigQuery loads all properties. If any named property
    /// isn't found in the Cloud Datastore backup, an invalid error is returned in the job result.
    #[builder(setter(into))]
    #[serde(default)]
    pub projection_fields: ::std::vec::Vec<::std::string::String>,
    /// Optional. The value that is used to quote data sections in a CSV file. BigQuery converts the
    /// string to ISO-8859-1 encoding, and then uses the first byte of the encoded string to split the
    /// data in its raw, binary state. The default value is a double-quote ('"'). If your data does not
    /// contain quoted sections, set the property value to an empty string. If your data contains quoted
    /// newline characters, you must also set the allowQuotedNewlines property to true. To include the
    /// specific quote character within a quoted value, precede it with an additional matching quote
    /// character. For example, if you want to escape the default character ' " ', use ' "" '. @default
    /// "
    #[builder(setter(into))]
    #[serde(default)]
    pub quote: ::std::option::Option<::std::string::String>,
    /// Range partitioning specification for the destination table. Only one of timePartitioning and
    /// rangePartitioning should be specified.
    #[builder(setter(into))]
    #[serde(default)]
    pub range_partitioning: RangePartitioning,
    /// Optional. The user can provide a reference file with the reader schema. This file is only loaded
    /// if it is part of source URIs, but is not loaded otherwise. It is enabled for the following
    /// formats: AVRO, PARQUET, ORC.
    #[builder(setter(into))]
    #[serde(default)]
    pub reference_file_schema_uri: ::std::option::Option<::std::string::String>,
    /// Optional. The schema for the destination table. The schema can be omitted if the destination
    /// table already exists, or if you're loading data from Google Cloud Datastore.
    #[builder(setter(into))]
    #[serde(default)]
    pub schema: ::std::option::Option<TableSchema>,
    /// [Deprecated] The inline schema. For CSV schemas, specify as "Field1:Type1[,Field2:Type2]*". For
    /// example, "foo:STRING, bar:INTEGER, baz:FLOAT".
    #[builder(setter(into))]
    #[serde(default)]
    pub schema_inline: ::std::string::String,
    /// [Deprecated] The format of the schemaInline property.
    #[builder(setter(into))]
    #[serde(default)]
    pub schema_inline_format: ::std::string::String,
    /// Allows the schema of the destination table to be updated as a side effect of the load job if a
    /// schema is autodetected or supplied in the job configuration. Schema update options are supported
    /// in two cases: when writeDisposition is WRITE_APPEND; when writeDisposition is WRITE_TRUNCATE and
    /// the destination table is a partition of a table, specified by partition decorators. For normal
    /// tables, WRITE_TRUNCATE will always overwrite the schema. One or more of the following values are
    /// specified: * ALLOW_FIELD_ADDITION: allow adding a nullable field to the schema. *
    /// ALLOW_FIELD_RELAXATION: allow relaxing a required field in the original schema to nullable.
    #[builder(setter(into))]
    #[serde(default)]
    pub schema_update_options: ::std::vec::Vec<::std::string::String>,
    /// Optional. The number of rows at the top of a CSV file that BigQuery will skip when loading the
    /// data. The default value is 0. This property is useful if you have header rows in the file that
    /// should be skipped. When autodetect is on, the behavior is the following: * skipLeadingRows
    /// unspecified - Autodetect tries to detect headers in the first row. If they are not detected, the
    /// row is read as data. Otherwise data is read starting from the second row. * skipLeadingRows is 0
    /// - Instructs autodetect that there are no headers and data should be read starting from the first
    /// row. * skipLeadingRows = N > 0 - Autodetect skips N-1 rows and tries to detect headers in row N.
    /// If headers are not detected, row N is just skipped. Otherwise row N is used to extract column
    /// names for the detected schema.
    #[builder(setter(into))]
    #[serde(default)]
    pub skip_leading_rows: ::std::option::Option<i64>,
    /// Optional. The format of the data files. For CSV files, specify "CSV". For datastore backups,
    /// specify "DATASTORE_BACKUP". For newline-delimited JSON, specify "NEWLINE_DELIMITED_JSON". For
    /// Avro, specify "AVRO". For parquet, specify "PARQUET". For orc, specify "ORC". The default value
    /// is CSV.
    #[builder(setter(into))]
    #[serde(default)]
    pub source_format: ::std::option::Option<::std::string::String>,
    /// [Required] The fully-qualified URIs that point to your data in Google Cloud. For Google Cloud
    /// Storage URIs: Each URI can contain one '*' wildcard character and it must come after the
    /// 'bucket' name. Size limits related to load jobs apply to external data sources. For Google Cloud
    /// Bigtable URIs: Exactly one URI can be specified and it has be a fully specified and valid HTTPS
    /// URL for a Google Cloud Bigtable table. For Google Cloud Datastore backups: Exactly one URI can
    /// be specified. Also, the '*' wildcard character is not allowed.
    #[builder(setter(into))]
    #[serde(default)]
    pub source_uris: ::std::vec::Vec<::std::string::String>,
    /// Time-based partitioning specification for the destination table. Only one of timePartitioning
    /// and rangePartitioning should be specified.
    #[builder(setter(into))]
    #[serde(default)]
    pub time_partitioning: TimePartitioning,
    /// Optional. If sourceFormat is set to "AVRO", indicates whether to interpret logical types as the
    /// corresponding BigQuery data type (for example, TIMESTAMP), instead of using the raw type (for
    /// example, INTEGER).
    #[builder(setter(into))]
    #[serde(default)]
    pub use_avro_logical_types: ::std::option::Option<bool>,
    /// Optional. Specifies the action that occurs if the destination table already exists. The
    /// following values are supported: * WRITE_TRUNCATE: If the table already exists, BigQuery
    /// overwrites the data, removes the constraints and uses the schema from the load job. *
    /// WRITE_APPEND: If the table already exists, BigQuery appends the data to the table. *
    /// WRITE_EMPTY: If the table already exists and contains data, a 'duplicate' error is returned in
    /// the job result. The default value is WRITE_APPEND. Each action is atomic and only occurs if
    /// BigQuery is able to complete the job successfully. Creation, truncation and append actions occur
    /// as one atomic update upon job completion.
    #[builder(setter(into))]
    #[serde(default)]
    pub write_disposition: ::std::option::Option<::std::string::String>,
}

/// Optional. You can specify external table definitions, which operate as ephemeral tables that can
/// be queried. These definitions are configured using a JSON map, where the string key represents
/// the table identifier, and the value is the corresponding external data configuration object.
pub type TableDefinitions = ::std::collections::HashMap<::std::string::String, ExternalDataConfiguration>;

/// JobConfigurationQuery configures a BigQuery query job.
#[derive(::serde::Deserialize, ::serde::Serialize, ::typed_builder::TypedBuilder, Debug, PartialEq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct JobConfigurationQuery {
    /// Optional. If true and query uses legacy SQL dialect, allows the query to produce arbitrarily
    /// large result tables at a slight cost in performance. Requires destinationTable to be set. For
    /// GoogleSQL queries, this flag is ignored and large results are always allowed. However, you must
    /// still set destinationTable when result size exceeds the allowed maximum response size.
    #[builder(setter(into))]
    #[serde(default)]
    pub allow_large_results: ::std::option::Option<bool>,
    /// Clustering specification for the destination table.
    #[builder(setter(into))]
    #[serde(default)]
    pub clustering: Clustering,
    /// Connection properties which can modify the query behavior.
    #[builder(setter(into))]
    #[serde(default)]
    pub connection_properties: ::std::vec::Vec<ConnectionProperty>,
    /// [Optional] Specifies whether the query should be executed as a continuous query. The default
    /// value is false.
    #[builder(setter(into))]
    #[serde(default)]
    pub continuous: ::std::option::Option<bool>,
    /// Optional. Specifies whether the job is allowed to create new tables. The following values are
    /// supported: * CREATE_IF_NEEDED: If the table does not exist, BigQuery creates the table. *
    /// CREATE_NEVER: The table must already exist. If it does not, a 'notFound' error is returned in
    /// the job result. The default value is CREATE_IF_NEEDED. Creation, truncation and append actions
    /// occur as one atomic update upon job completion.
    #[builder(setter(into))]
    #[serde(default)]
    pub create_disposition: ::std::option::Option<::std::string::String>,
    /// If this property is true, the job creates a new session using a randomly generated session_id.
    /// To continue using a created session with subsequent queries, pass the existing session
    /// identifier as a `ConnectionProperty` value. The session identifier is returned as part of the
    /// `SessionInfo` message within the query statistics. The new session's location will be set to
    /// `Job.JobReference.location` if it is present, otherwise it's set to the default location based
    /// on existing routing logic.
    #[builder(setter(into))]
    #[serde(default)]
    pub create_session: bool,
    /// Optional. Specifies the default dataset to use for unqualified table names in the query. This
    /// setting does not alter behavior of unqualified dataset names. Setting the system variable
    /// `@@dataset_id` achieves the same behavior. See
    /// https://cloud.google.com/bigquery/docs/reference/system-variables for more information on system
    /// variables.
    #[builder(setter(into))]
    #[serde(default)]
    pub default_dataset: DatasetReference,
    /// Custom encryption configuration (e.g., Cloud KMS keys)
    #[builder(setter(into))]
    #[serde(default)]
    pub destination_encryption_configuration: EncryptionConfiguration,
    /// Optional. Describes the table where the query results should be stored. This property must be
    /// set for large results that exceed the maximum response size. For queries that produce anonymous
    /// (cached) results, this field will be populated by BigQuery.
    #[builder(setter(into))]
    #[serde(default)]
    pub destination_table: ::std::option::Option<TableReference>,
    /// Optional. If true and query uses legacy SQL dialect, flattens all nested and repeated fields in
    /// the query results. allowLargeResults must be true if this is set to false. For GoogleSQL
    /// queries, this flag is ignored and results are never flattened.
    #[builder(setter(into))]
    #[serde(default)]
    pub flatten_results: ::std::option::Option<bool>,
    /// Optional. [Deprecated] Maximum billing tier allowed for this query. The billing tier controls
    /// the amount of compute resources allotted to the query, and multiplies the on-demand cost of the
    /// query accordingly. A query that runs within its allotted resources will succeed and indicate its
    /// billing tier in statistics.query.billingTier, but if the query exceeds its allotted resources,
    /// it will fail with billingTierLimitExceeded. WARNING: The billed byte amount can be multiplied by
    /// an amount up to this number! Most users should not need to alter this setting, and we recommend
    /// that you avoid introducing new uses of it.
    #[builder(setter(into))]
    #[serde(default)]
    pub maximum_billing_tier: ::std::option::Option<i64>,
    /// Limits the bytes billed for this job. Queries that will have bytes billed beyond this limit will
    /// fail (without incurring a charge). If unspecified, this will be set to your project default.
    #[builder(setter(into))]
    #[serde(with = "with::int64")]
    #[serde(default)]
    pub maximum_bytes_billed: i64,
    /// GoogleSQL only. Set to POSITIONAL to use positional (?) query parameters or to NAMED to use
    /// named (@myparam) query parameters in this query.
    #[builder(setter(into))]
    #[serde(default)]
    pub parameter_mode: ::std::string::String,
    /// [Deprecated] This property is deprecated.
    #[builder(setter(into))]
    #[serde(default)]
    pub preserve_nulls: bool,
    /// Optional. Specifies a priority for the query. Possible values include INTERACTIVE and BATCH. The
    /// default value is INTERACTIVE.
    #[builder(setter(into))]
    #[serde(default)]
    pub priority: ::std::option::Option<::std::string::String>,
    /// [Required] SQL query text to execute. The useLegacySql field can be used to indicate whether the
    /// query uses legacy SQL or GoogleSQL.
    #[builder(setter(into))]
    #[serde(default)]
    pub query: ::std::string::String,
    /// Query parameters for GoogleSQL queries.
    #[builder(setter(into))]
    #[serde(default)]
    pub query_parameters: ::std::vec::Vec<QueryParameter>,
    /// Range partitioning specification for the destination table. Only one of timePartitioning and
    /// rangePartitioning should be specified.
    #[builder(setter(into))]
    #[serde(default)]
    pub range_partitioning: RangePartitioning,
    /// Allows the schema of the destination table to be updated as a side effect of the query job.
    /// Schema update options are supported in two cases: when writeDisposition is WRITE_APPEND; when
    /// writeDisposition is WRITE_TRUNCATE and the destination table is a partition of a table,
    /// specified by partition decorators. For normal tables, WRITE_TRUNCATE will always overwrite the
    /// schema. One or more of the following values are specified: * ALLOW_FIELD_ADDITION: allow adding
    /// a nullable field to the schema. * ALLOW_FIELD_RELAXATION: allow relaxing a required field in the
    /// original schema to nullable.
    #[builder(setter(into))]
    #[serde(default)]
    pub schema_update_options: ::std::vec::Vec<::std::string::String>,
    /// Options controlling the execution of scripts.
    #[builder(setter(into))]
    pub script_options: ScriptOptions,
    /// Output only. System variables for GoogleSQL queries. A system variable is output if the variable
    /// is settable and its value differs from the system default. "@@" prefix is not included in the
    /// name of the System variables.
    #[builder(setter(into))]
    #[serde(default)]
    pub system_variables: SystemVariables,
    /// Optional. You can specify external table definitions, which operate as ephemeral tables that can
    /// be queried. These definitions are configured using a JSON map, where the string key represents
    /// the table identifier, and the value is the corresponding external data configuration object.
    #[builder(setter(into))]
    #[serde(default)]
    pub table_definitions: TableDefinitions,
    /// Time-based partitioning specification for the destination table. Only one of timePartitioning
    /// and rangePartitioning should be specified.
    #[builder(setter(into))]
    #[serde(default)]
    pub time_partitioning: TimePartitioning,
    /// Optional. Specifies whether to use BigQuery's legacy SQL dialect for this query. The default
    /// value is true. If set to false, the query will use BigQuery's GoogleSQL:
    /// https://cloud.google.com/bigquery/sql-reference/ When useLegacySql is set to false, the value of
    /// flattenResults is ignored; query will be run as if flattenResults is false.
    #[builder(setter(into))]
    #[serde(default)]
    pub use_legacy_sql: ::std::option::Option<bool>,
    /// Optional. Whether to look for the result in the query cache. The query cache is a best-effort
    /// cache that will be flushed whenever tables in the query are modified. Moreover, the query cache
    /// is only available when a query does not have a destination table specified. The default value is
    /// true.
    #[builder(setter(into))]
    #[serde(default)]
    pub use_query_cache: ::std::option::Option<bool>,
    /// Describes user-defined function resources used in the query.
    #[builder(setter(into))]
    #[serde(default)]
    pub user_defined_function_resources: ::std::vec::Vec<UserDefinedFunctionResource>,
    /// Optional. Specifies the action that occurs if the destination table already exists. The
    /// following values are supported: * WRITE_TRUNCATE: If the table already exists, BigQuery
    /// overwrites the data, removes the constraints, and uses the schema from the query result. *
    /// WRITE_APPEND: If the table already exists, BigQuery appends the data to the table. *
    /// WRITE_EMPTY: If the table already exists and contains data, a 'duplicate' error is returned in
    /// the job result. The default value is WRITE_EMPTY. Each action is atomic and only occurs if
    /// BigQuery is able to complete the job successfully. Creation, truncation and append actions occur
    /// as one atomic update upon job completion.
    #[builder(setter(into))]
    #[serde(default)]
    pub write_disposition: ::std::option::Option<::std::string::String>,
}

/// Optional. Supported operation types in table copy job.
#[derive(::serde::Deserialize, ::serde::Serialize, Debug, PartialEq, Clone)]
pub enum OperationType {
        /// The source and destination table have the same table type.
    #[serde(rename = "COPY")]
    Copy,
        /// The source table type is TABLE and the destination table type is SNAPSHOT.
    #[serde(rename = "SNAPSHOT")]
    Snapshot,
        /// The source table type is SNAPSHOT and the destination table type is TABLE.
    #[serde(rename = "RESTORE")]
    Restore,
        /// The source and destination table have the same table type, but only bill for unique data.
    #[serde(rename = "CLONE")]
    Clone,
}

/// JobConfigurationTableCopy configures a job that copies data from one table to another. For more
/// information on copying tables, see [Copy a
/// table](https://cloud.google.com/bigquery/docs/managing-tables#copy-table).
#[derive(::serde::Deserialize, ::serde::Serialize, ::typed_builder::TypedBuilder, Debug, PartialEq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct JobConfigurationTableCopy {
    /// Optional. Specifies whether the job is allowed to create new tables. The following values are
    /// supported: * CREATE_IF_NEEDED: If the table does not exist, BigQuery creates the table. *
    /// CREATE_NEVER: The table must already exist. If it does not, a 'notFound' error is returned in
    /// the job result. The default value is CREATE_IF_NEEDED. Creation, truncation and append actions
    /// occur as one atomic update upon job completion.
    #[builder(setter(into))]
    #[serde(default)]
    pub create_disposition: ::std::option::Option<::std::string::String>,
    /// Custom encryption configuration (e.g., Cloud KMS keys).
    #[builder(setter(into))]
    #[serde(default)]
    pub destination_encryption_configuration: EncryptionConfiguration,
    /// Optional. The time when the destination table expires. Expired tables will be deleted and their
    /// storage reclaimed.
    #[builder(setter(into))]
    #[serde(default)]
    pub destination_expiration_time: ::std::option::Option<::std::string::String>,
    /// [Required] The destination table.
    #[builder(setter(into))]
    #[serde(default)]
    pub destination_table: TableReference,
    /// Optional. Supported operation types in table copy job.
    #[builder(setter(into))]
    pub operation_type: OperationType,
    /// [Pick one] Source table to copy.
    #[builder(setter(into))]
    #[serde(default)]
    pub source_table: TableReference,
    /// [Pick one] Source tables to copy.
    #[builder(setter(into))]
    #[serde(default)]
    pub source_tables: ::std::vec::Vec<TableReference>,
    /// Optional. Specifies the action that occurs if the destination table already exists. The
    /// following values are supported: * WRITE_TRUNCATE: If the table already exists, BigQuery
    /// overwrites the table data and uses the schema and table constraints from the source table. *
    /// WRITE_APPEND: If the table already exists, BigQuery appends the data to the table. *
    /// WRITE_EMPTY: If the table already exists and contains data, a 'duplicate' error is returned in
    /// the job result. The default value is WRITE_EMPTY. Each action is atomic and only occurs if
    /// BigQuery is able to complete the job successfully. Creation, truncation and append actions occur
    /// as one atomic update upon job completion.
    #[builder(setter(into))]
    #[serde(default)]
    pub write_disposition: ::std::option::Option<::std::string::String>,
}

/// Reason about why a Job was created from a
/// [`jobs.query`](https://cloud.google.com/bigquery/docs/reference/rest/v2/jobs/query) method when
/// used with `JOB_CREATION_OPTIONAL` Job creation mode. For
/// [`jobs.insert`](https://cloud.google.com/bigquery/docs/reference/rest/v2/jobs/insert) method
/// calls it will always be `REQUESTED`. This feature is not yet available. Jobs will always be
/// created.
#[derive(::serde::Deserialize, ::serde::Serialize, ::typed_builder::TypedBuilder, Debug, PartialEq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct JobCreationReason {
    /// Output only. Specifies the high level reason why a Job was created.
    #[builder(setter(into))]
    pub code: Code,
}

/// ListFormatJob is a partial projection of job information returned as part of a jobs.list
/// response.
#[derive(::serde::Deserialize, ::serde::Serialize, ::typed_builder::TypedBuilder, Debug, PartialEq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Jobs {
    /// Required. Describes the job configuration.
    #[builder(setter(into))]
    pub configuration: JobConfiguration,
    /// A result object that will be present only if the job has failed.
    #[builder(setter(into))]
    #[serde(default)]
    pub error_result: ErrorProto,
    /// Unique opaque ID of the job.
    #[builder(setter(into))]
    #[serde(default)]
    pub id: ::std::string::String,
    /// Unique opaque ID of the job.
    #[builder(setter(into))]
    #[serde(default)]
    pub job_reference: JobReference,
    /// The resource type.
    #[builder(setter(into))]
    #[serde(default)]
    pub kind: ::std::string::String,
    /// [Full-projection-only] String representation of identity of requesting party. Populated for both
    /// first- and third-party identities. Only present for APIs that support third-party identities.
    #[builder(setter(into))]
    #[serde(default)]
    pub principal_subject: ::std::string::String,
    /// Running state of the job. When the state is DONE, errorResult can be checked to determine
    /// whether the job succeeded or failed.
    #[builder(setter(into))]
    #[serde(default)]
    pub state: ::std::string::String,
    /// Output only. Information about the job, including starting time and ending time of the job.
    #[builder(setter(into))]
    pub statistics: JobStatistics,
    /// [Full-projection-only] Describes the status of this job.
    #[builder(setter(into))]
    #[serde(default)]
    pub status: JobStatus,
    /// [Full-projection-only] Email address of the user who ran the job.
    #[builder(setter(into))]
    #[serde(default)]
    pub user_email: ::std::string::String,
}

/// JobList is the response format for a jobs.list call.
#[derive(::serde::Deserialize, ::serde::Serialize, ::typed_builder::TypedBuilder, Debug, PartialEq, Clone)]
#[derive(Default)]
#[serde(rename_all = "camelCase")]
pub struct JobList {
    /// A hash of this page of results.
    #[builder(setter(into))]
    #[serde(default)]
    pub etag: ::std::string::String,
    /// List of jobs that were requested.
    #[builder(setter(into))]
    #[serde(default)]
    pub jobs: ::std::vec::Vec<Jobs>,
    /// The resource type of the response.
    #[builder(setter(into))]
    #[serde(default)]
    pub kind: ::std::string::String,
    /// A token to request the next page of results.
    #[builder(setter(into))]
    #[serde(default)]
    pub next_page_token: ::std::string::String,
    /// A list of skipped locations that were unreachable. For more information about BigQuery
    /// locations, see: https://cloud.google.com/bigquery/docs/locations. Example: "europe-west5"
    #[builder(setter(into))]
    #[serde(default)]
    pub unreachable: ::std::vec::Vec<::std::string::String>,
}

/// A job reference is a fully qualified identifier for referring to a job.
#[derive(::serde::Deserialize, ::serde::Serialize, ::typed_builder::TypedBuilder, Debug, PartialEq, Clone)]
#[derive(Default)]
#[serde(rename_all = "camelCase")]
pub struct JobReference {
    /// Required. The ID of the job. The ID must contain only letters (a-z, A-Z), numbers (0-9),
    /// underscores (_), or dashes (-). The maximum length is 1,024 characters.
    #[builder(setter(into))]
    #[serde(default)]
    pub job_id: ::std::string::String,
    /// Optional. The geographic location of the job. The default value is US. For more information
    /// about BigQuery locations, see: https://cloud.google.com/bigquery/docs/locations
    #[builder(setter(into))]
    #[serde(default)]
    pub location: ::std::option::Option<::std::string::String>,
    /// Required. The ID of the project containing this job.
    #[builder(setter(into))]
    #[serde(default)]
    pub project_id: ::std::string::String,
}

/// Job resource usage breakdown by reservation.
#[derive(::serde::Deserialize, ::serde::Serialize, ::typed_builder::TypedBuilder, Debug, PartialEq, Clone)]
#[derive(Default)]
#[serde(rename_all = "camelCase")]
pub struct ReservationUsage {
    /// Reservation name or "unreserved" for on-demand resources usage.
    #[builder(setter(into))]
    #[serde(default)]
    pub name: ::std::string::String,
    /// Total slot milliseconds used by the reservation for a particular job.
    #[builder(setter(into))]
    #[serde(with = "with::int64")]
    #[serde(default)]
    pub slot_ms: i64,
}

/// Statistics for a single job execution.
#[derive(::serde::Deserialize, ::serde::Serialize, ::typed_builder::TypedBuilder, Debug, PartialEq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct JobStatistics {
    /// Output only. [TrustedTester] Job progress (0.0 -> 1.0) for LOAD and EXTRACT jobs.
    #[builder(setter(into))]
    #[serde(default)]
    pub completion_ratio: f64,
    /// Output only. Statistics for a copy job.
    #[builder(setter(into))]
    #[serde(default)]
    pub copy: JobStatistics5,
    /// Output only. Creation time of this job, in milliseconds since the epoch. This field will be
    /// present on all jobs.
    #[builder(setter(into))]
    #[serde(with = "with::int64")]
    #[serde(default)]
    pub creation_time: i64,
    /// Output only. Statistics for data-masking. Present only for query and extract jobs.
    #[builder(setter(into))]
    #[serde(default)]
    pub data_masking_statistics: DataMaskingStatistics,
    /// Output only. End time of this job, in milliseconds since the epoch. This field will be present
    /// whenever a job is in the DONE state.
    #[builder(setter(into))]
    #[serde(with = "with::int64")]
    #[serde(default)]
    pub end_time: i64,
    /// Output only. Statistics for an extract job.
    #[builder(setter(into))]
    #[serde(default)]
    pub extract: JobStatistics4,
    /// Output only. The duration in milliseconds of the execution of the final attempt of this job, as
    /// BigQuery may internally re-attempt to execute the job.
    #[builder(setter(into))]
    #[serde(with = "with::int64")]
    #[serde(default)]
    pub final_execution_duration_ms: i64,
    /// Output only. Statistics for a load job.
    #[builder(setter(into))]
    #[serde(default)]
    pub load: JobStatistics3,
    /// Output only. Number of child jobs executed.
    #[builder(setter(into))]
    #[serde(with = "with::int64")]
    #[serde(default)]
    pub num_child_jobs: i64,
    /// Output only. If this is a child job, specifies the job ID of the parent.
    #[builder(setter(into))]
    #[serde(default)]
    pub parent_job_id: ::std::string::String,
    /// Output only. Statistics for a query job.
    #[builder(setter(into))]
    pub query: JobStatistics2,
    /// Output only. Quotas which delayed this job's start time.
    #[builder(setter(into))]
    #[serde(default)]
    pub quota_deferments: ::std::vec::Vec<::std::string::String>,
    /// Output only. Job resource usage breakdown by reservation. This field reported misleading
    /// information and will no longer be populated.
    #[builder(setter(into))]
    #[serde(default)]
    pub reservation_usage: ::std::vec::Vec<ReservationUsage>,
    /// Output only. Name of the primary reservation assigned to this job. Note that this could be
    /// different than reservations reported in the reservation usage field if parent reservations were
    /// used to execute this job.
    #[builder(setter(into))]
    #[serde(default)]
    pub reservation_id: ::std::string::String,
    /// Output only. Statistics for row-level security. Present only for query and extract jobs.
    #[builder(setter(into))]
    #[serde(default)]
    pub row_level_security_statistics: RowLevelSecurityStatistics,
    /// Output only. If this a child job of a script, specifies information about the context of this
    /// job within the script.
    #[builder(setter(into))]
    pub script_statistics: ScriptStatistics,
    /// Output only. Information of the session if this job is part of one.
    #[builder(setter(into))]
    #[serde(default)]
    pub session_info: SessionInfo,
    /// Output only. Start time of this job, in milliseconds since the epoch. This field will be present
    /// when the job transitions from the PENDING state to either RUNNING or DONE.
    #[builder(setter(into))]
    #[serde(with = "with::int64")]
    #[serde(default)]
    pub start_time: i64,
    /// Output only. Total bytes processed for the job.
    #[builder(setter(into))]
    #[serde(with = "with::int64")]
    #[serde(default)]
    pub total_bytes_processed: i64,
    /// Output only. Slot-milliseconds for the job.
    #[builder(setter(into))]
    #[serde(with = "with::int64")]
    #[serde(default)]
    pub total_slot_ms: i64,
    /// Output only. [Alpha] Information of the multi-statement transaction if this job is part of one.
    /// This property is only expected on a child job or a job that is in a session. A script parent job
    /// is not part of the transaction started in the script.
    #[builder(setter(into))]
    #[serde(default)]
    pub transaction_info: TransactionInfo,
}

/// Statistics for a query job.
#[derive(::serde::Deserialize, ::serde::Serialize, ::typed_builder::TypedBuilder, Debug, PartialEq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct JobStatistics2 {
    /// Output only. BI Engine specific Statistics.
    #[builder(setter(into))]
    pub bi_engine_statistics: BiEngineStatistics,
    /// Output only. Billing tier for the job. This is a BigQuery-specific concept which is not related
    /// to the Google Cloud notion of "free tier". The value here is a measure of the query's resource
    /// consumption relative to the amount of data scanned. For on-demand queries, the limit is 100, and
    /// all queries within this limit are billed at the standard on-demand rates. On-demand queries that
    /// exceed this limit will fail with a billingTierLimitExceeded error.
    #[builder(setter(into))]
    #[serde(default)]
    pub billing_tier: i64,
    /// Output only. Whether the query result was fetched from the query cache.
    #[builder(setter(into))]
    #[serde(default)]
    pub cache_hit: bool,
    /// Output only. Referenced dataset for DCL statement.
    #[builder(setter(into))]
    #[serde(default)]
    pub dcl_target_dataset: DatasetReference,
    /// Output only. Referenced table for DCL statement.
    #[builder(setter(into))]
    #[serde(default)]
    pub dcl_target_table: TableReference,
    /// Output only. Referenced view for DCL statement.
    #[builder(setter(into))]
    #[serde(default)]
    pub dcl_target_view: TableReference,
    /// Output only. The number of row access policies affected by a DDL statement. Present only for
    /// DROP ALL ROW ACCESS POLICIES queries.
    #[builder(setter(into))]
    #[serde(with = "with::int64")]
    #[serde(default)]
    pub ddl_affected_row_access_policy_count: i64,
    /// Output only. The table after rename. Present only for ALTER TABLE RENAME TO query.
    #[builder(setter(into))]
    #[serde(default)]
    pub ddl_destination_table: TableReference,
    /// Output only. The DDL operation performed, possibly dependent on the pre-existence of the DDL
    /// target.
    #[builder(setter(into))]
    #[serde(default)]
    pub ddl_operation_performed: ::std::string::String,
    /// Output only. The DDL target dataset. Present only for CREATE/ALTER/DROP SCHEMA(dataset) queries.
    #[builder(setter(into))]
    #[serde(default)]
    pub ddl_target_dataset: DatasetReference,
    /// Output only. [Beta] The DDL target routine. Present only for CREATE/DROP FUNCTION/PROCEDURE
    /// queries.
    #[builder(setter(into))]
    #[serde(default)]
    pub ddl_target_routine: RoutineReference,
    /// Output only. The DDL target row access policy. Present only for CREATE/DROP ROW ACCESS POLICY
    /// queries.
    #[builder(setter(into))]
    #[serde(default)]
    pub ddl_target_row_access_policy: RowAccessPolicyReference,
    /// Output only. The DDL target table. Present only for CREATE/DROP TABLE/VIEW and DROP ALL ROW
    /// ACCESS POLICIES queries.
    #[builder(setter(into))]
    #[serde(default)]
    pub ddl_target_table: TableReference,
    /// Output only. Detailed statistics for DML statements INSERT, UPDATE, DELETE, MERGE or TRUNCATE.
    #[builder(setter(into))]
    #[serde(default)]
    pub dml_stats: DmlStatistics,
    /// Output only. The original estimate of bytes processed for the job.
    #[builder(setter(into))]
    #[serde(with = "with::int64")]
    #[serde(default)]
    pub estimated_bytes_processed: i64,
    /// Output only. Stats for EXPORT DATA statement.
    #[builder(setter(into))]
    #[serde(default)]
    pub export_data_statistics: ExportDataStatistics,
    /// Output only. Job cost breakdown as bigquery internal cost and external service costs.
    #[builder(setter(into))]
    #[serde(default)]
    pub external_service_costs: ::std::vec::Vec<ExternalServiceCost>,
    /// Output only. Statistics for a LOAD query.
    #[builder(setter(into))]
    #[serde(default)]
    pub load_query_statistics: LoadQueryStatistics,
    /// Output only. Statistics of materialized views of a query job.
    #[builder(setter(into))]
    #[serde(default)]
    pub materialized_view_statistics: MaterializedViewStatistics,
    /// Output only. Statistics of metadata cache usage in a query for BigLake tables.
    #[builder(setter(into))]
    #[serde(default)]
    pub metadata_cache_statistics: MetadataCacheStatistics,
    /// Output only. Statistics of a BigQuery ML training job.
    #[builder(setter(into))]
    pub ml_statistics: MlStatistics,
    /// Deprecated.
    #[builder(setter(into))]
    #[serde(default)]
    pub model_training: BigQueryModelTraining,
    /// Deprecated.
    #[builder(setter(into))]
    #[serde(default)]
    pub model_training_current_iteration: i64,
    /// Deprecated.
    #[builder(setter(into))]
    #[serde(with = "with::int64")]
    #[serde(default)]
    pub model_training_expected_total_iteration: i64,
    /// Output only. The number of rows affected by a DML statement. Present only for DML statements
    /// INSERT, UPDATE or DELETE.
    #[builder(setter(into))]
    #[serde(with = "with::int64")]
    #[serde(default)]
    pub num_dml_affected_rows: i64,
    /// Output only. Performance insights.
    #[builder(setter(into))]
    #[serde(default)]
    pub performance_insights: PerformanceInsights,
    /// Output only. Query optimization information for a QUERY job.
    #[builder(setter(into))]
    #[serde(default)]
    pub query_info: QueryInfo,
    /// Output only. Describes execution plan for the query.
    #[builder(setter(into))]
    #[serde(default)]
    pub query_plan: ::std::vec::Vec<ExplainQueryStage>,
    /// Output only. Referenced routines for the job.
    #[builder(setter(into))]
    #[serde(default)]
    pub referenced_routines: ::std::vec::Vec<RoutineReference>,
    /// Output only. Referenced tables for the job. Queries that reference more than 50 tables will not
    /// have a complete list.
    #[builder(setter(into))]
    #[serde(default)]
    pub referenced_tables: ::std::vec::Vec<TableReference>,
    /// Output only. Job resource usage breakdown by reservation. This field reported misleading
    /// information and will no longer be populated.
    #[builder(setter(into))]
    #[serde(default)]
    pub reservation_usage: ::std::vec::Vec<ReservationUsage>,
    /// Output only. The schema of the results. Present only for successful dry run of non-legacy SQL
    /// queries.
    #[builder(setter(into))]
    #[serde(default)]
    pub schema: TableSchema,
    /// Output only. Search query specific statistics.
    #[builder(setter(into))]
    pub search_statistics: SearchStatistics,
    /// Output only. Statistics of a Spark procedure job.
    #[builder(setter(into))]
    #[serde(default)]
    pub spark_statistics: SparkStatistics,
    /// Output only. The type of query statement, if valid. Possible values: * `SELECT`:
    /// [`SELECT`](/bigquery/docs/reference/standard-sql/query-syntax#select_list) statement. *
    /// `ASSERT`: [`ASSERT`](/bigquery/docs/reference/standard-sql/debugging-statements#assert)
    /// statement. * `INSERT`:
    /// [`INSERT`](/bigquery/docs/reference/standard-sql/dml-syntax#insert_statement) statement. *
    /// `UPDATE`: [`UPDATE`](/bigquery/docs/reference/standard-sql/query-syntax#update_statement)
    /// statement. * `DELETE`:
    /// [`DELETE`](/bigquery/docs/reference/standard-sql/data-manipulation-language) statement. *
    /// `MERGE`: [`MERGE`](/bigquery/docs/reference/standard-sql/data-manipulation-language) statement.
    /// * `CREATE_TABLE`: [`CREATE
    /// TABLE`](/bigquery/docs/reference/standard-sql/data-definition-language#create_table_statement)
    /// statement, without `AS SELECT`. * `CREATE_TABLE_AS_SELECT`: [`CREATE TABLE AS
    /// SELECT`](/bigquery/docs/reference/standard-sql/data-definition-language#query_statement)
    /// statement. * `CREATE_VIEW`: [`CREATE
    /// VIEW`](/bigquery/docs/reference/standard-sql/data-definition-language#create_view_statement)
    /// statement. * `CREATE_MODEL`: [`CREATE
    /// MODEL`](/bigquery-ml/docs/reference/standard-sql/bigqueryml-syntax-create#create_model_statement)
    /// statement. * `CREATE_MATERIALIZED_VIEW`: [`CREATE MATERIALIZED
    /// VIEW`](/bigquery/docs/reference/standard-sql/data-definition-language#create_materialized_view_statement)
    /// statement. * `CREATE_FUNCTION`: [`CREATE
    /// FUNCTION`](/bigquery/docs/reference/standard-sql/data-definition-language#create_function_statement)
    /// statement. * `CREATE_TABLE_FUNCTION`: [`CREATE TABLE
    /// FUNCTION`](/bigquery/docs/reference/standard-sql/data-definition-language#create_table_function_statement)
    /// statement. * `CREATE_PROCEDURE`: [`CREATE
    /// PROCEDURE`](/bigquery/docs/reference/standard-sql/data-definition-language#create_procedure)
    /// statement. * `CREATE_ROW_ACCESS_POLICY`: [`CREATE ROW ACCESS
    /// POLICY`](/bigquery/docs/reference/standard-sql/data-definition-language#create_row_access_policy_statement)
    /// statement. * `CREATE_SCHEMA`: [`CREATE
    /// SCHEMA`](/bigquery/docs/reference/standard-sql/data-definition-language#create_schema_statement)
    /// statement. * `CREATE_SNAPSHOT_TABLE`: [`CREATE SNAPSHOT
    /// TABLE`](/bigquery/docs/reference/standard-sql/data-definition-language#create_snapshot_table_statement)
    /// statement. * `CREATE_SEARCH_INDEX`: [`CREATE SEARCH
    /// INDEX`](/bigquery/docs/reference/standard-sql/data-definition-language#create_search_index_statement)
    /// statement. * `DROP_TABLE`: [`DROP
    /// TABLE`](/bigquery/docs/reference/standard-sql/data-definition-language#drop_table_statement)
    /// statement. * `DROP_EXTERNAL_TABLE`: [`DROP EXTERNAL
    /// TABLE`](/bigquery/docs/reference/standard-sql/data-definition-language#drop_external_table_statement)
    /// statement. * `DROP_VIEW`: [`DROP
    /// VIEW`](/bigquery/docs/reference/standard-sql/data-definition-language#drop_view_statement)
    /// statement. * `DROP_MODEL`: [`DROP
    /// MODEL`](/bigquery-ml/docs/reference/standard-sql/bigqueryml-syntax-drop-model) statement. *
    /// `DROP_MATERIALIZED_VIEW`: [`DROP MATERIALIZED
    /// VIEW`](/bigquery/docs/reference/standard-sql/data-definition-language#drop_materialized_view_statement)
    /// statement. * `DROP_FUNCTION` : [`DROP
    /// FUNCTION`](/bigquery/docs/reference/standard-sql/data-definition-language#drop_function_statement)
    /// statement. * `DROP_TABLE_FUNCTION` : [`DROP TABLE
    /// FUNCTION`](/bigquery/docs/reference/standard-sql/data-definition-language#drop_table_function)
    /// statement. * `DROP_PROCEDURE`: [`DROP
    /// PROCEDURE`](/bigquery/docs/reference/standard-sql/data-definition-language#drop_procedure_statement)
    /// statement. * `DROP_SEARCH_INDEX`: [`DROP SEARCH
    /// INDEX`](/bigquery/docs/reference/standard-sql/data-definition-language#drop_search_index)
    /// statement. * `DROP_SCHEMA`: [`DROP
    /// SCHEMA`](/bigquery/docs/reference/standard-sql/data-definition-language#drop_schema_statement)
    /// statement. * `DROP_SNAPSHOT_TABLE`: [`DROP SNAPSHOT
    /// TABLE`](/bigquery/docs/reference/standard-sql/data-definition-language#drop_snapshot_table_statement)
    /// statement. * `DROP_ROW_ACCESS_POLICY`: [`DROP [ALL] ROW ACCESS
    /// POLICY|POLICIES`](/bigquery/docs/reference/standard-sql/data-definition-language#drop_row_access_policy_statement)
    /// statement. * `ALTER_TABLE`: [`ALTER
    /// TABLE`](/bigquery/docs/reference/standard-sql/data-definition-language#alter_table_set_options_statement)
    /// statement. * `ALTER_VIEW`: [`ALTER
    /// VIEW`](/bigquery/docs/reference/standard-sql/data-definition-language#alter_view_set_options_statement)
    /// statement. * `ALTER_MATERIALIZED_VIEW`: [`ALTER MATERIALIZED
    /// VIEW`](/bigquery/docs/reference/standard-sql/data-definition-language#alter_materialized_view_set_options_statement)
    /// statement. * `ALTER_SCHEMA`: [`ALTER
    /// SCHEMA`](/bigquery/docs/reference/standard-sql/data-definition-language#aalter_schema_set_options_statement)
    /// statement. * `SCRIPT`: [`SCRIPT`](/bigquery/docs/reference/standard-sql/procedural-language). *
    /// `TRUNCATE_TABLE`: [`TRUNCATE
    /// TABLE`](/bigquery/docs/reference/standard-sql/dml-syntax#truncate_table_statement) statement. *
    /// `CREATE_EXTERNAL_TABLE`: [`CREATE EXTERNAL
    /// TABLE`](/bigquery/docs/reference/standard-sql/data-definition-language#create_external_table_statement)
    /// statement. * `EXPORT_DATA`: [`EXPORT
    /// DATA`](/bigquery/docs/reference/standard-sql/other-statements#export_data_statement) statement.
    /// * `EXPORT_MODEL`: [`EXPORT
    /// MODEL`](/bigquery-ml/docs/reference/standard-sql/bigqueryml-syntax-export-model) statement. *
    /// `LOAD_DATA`: [`LOAD
    /// DATA`](/bigquery/docs/reference/standard-sql/other-statements#load_data_statement) statement. *
    /// `CALL`: [`CALL`](/bigquery/docs/reference/standard-sql/procedural-language#call) statement.
    #[builder(setter(into))]
    #[serde(default)]
    pub statement_type: ::std::string::String,
    /// Output only. Describes a timeline of job execution.
    #[builder(setter(into))]
    #[serde(default)]
    pub timeline: ::std::vec::Vec<QueryTimelineSample>,
    /// Output only. If the project is configured to use on-demand pricing, then this field contains the
    /// total bytes billed for the job. If the project is configured to use flat-rate pricing, then you
    /// are not billed for bytes and this field is informational only.
    #[builder(setter(into))]
    #[serde(with = "with::int64")]
    #[serde(default)]
    pub total_bytes_billed: i64,
    /// Output only. Total bytes processed for the job.
    #[builder(setter(into))]
    #[serde(with = "with::int64")]
    #[serde(default)]
    pub total_bytes_processed: i64,
    /// Output only. For dry-run jobs, totalBytesProcessed is an estimate and this field specifies the
    /// accuracy of the estimate. Possible values can be: UNKNOWN: accuracy of the estimate is unknown.
    /// PRECISE: estimate is precise. LOWER_BOUND: estimate is lower bound of what the query would cost.
    /// UPPER_BOUND: estimate is upper bound of what the query would cost.
    #[builder(setter(into))]
    #[serde(default)]
    pub total_bytes_processed_accuracy: ::std::string::String,
    /// Output only. Total number of partitions processed from all partitioned tables referenced in the
    /// job.
    #[builder(setter(into))]
    #[serde(with = "with::int64")]
    #[serde(default)]
    pub total_partitions_processed: i64,
    /// Output only. Slot-milliseconds for the job.
    #[builder(setter(into))]
    #[serde(with = "with::int64")]
    #[serde(default)]
    pub total_slot_ms: i64,
    /// Output only. Total bytes transferred for cross-cloud queries such as Cross Cloud Transfer and
    /// CREATE TABLE AS SELECT (CTAS).
    #[builder(setter(into))]
    #[serde(with = "with::int64")]
    #[serde(default)]
    pub transferred_bytes: i64,
    /// Output only. GoogleSQL only: list of undeclared query parameters detected during a dry run
    /// validation.
    #[builder(setter(into))]
    #[serde(default)]
    pub undeclared_query_parameters: ::std::vec::Vec<QueryParameter>,
    /// Output only. Vector Search query specific statistics.
    #[builder(setter(into))]
    pub vector_search_statistics: VectorSearchStatistics,
}

/// Statistics for a load job.
#[derive(::serde::Deserialize, ::serde::Serialize, ::typed_builder::TypedBuilder, Debug, PartialEq, Clone)]
#[derive(Default)]
#[serde(rename_all = "camelCase")]
pub struct JobStatistics3 {
    /// Output only. The number of bad records encountered. Note that if the job has failed because of
    /// more bad records encountered than the maximum allowed in the load job configuration, then this
    /// number can be less than the total number of bad records present in the input data.
    #[builder(setter(into))]
    #[serde(with = "with::int64")]
    #[serde(default)]
    pub bad_records: i64,
    /// Output only. Number of bytes of source data in a load job.
    #[builder(setter(into))]
    #[serde(with = "with::int64")]
    #[serde(default)]
    pub input_file_bytes: i64,
    /// Output only. Number of source files in a load job.
    #[builder(setter(into))]
    #[serde(with = "with::int64")]
    #[serde(default)]
    pub input_files: i64,
    /// Output only. Size of the loaded data in bytes. Note that while a load job is in the running
    /// state, this value may change.
    #[builder(setter(into))]
    #[serde(with = "with::int64")]
    #[serde(default)]
    pub output_bytes: i64,
    /// Output only. Number of rows imported in a load job. Note that while an import job is in the
    /// running state, this value may change.
    #[builder(setter(into))]
    #[serde(with = "with::int64")]
    #[serde(default)]
    pub output_rows: i64,
    /// Output only. Describes a timeline of job execution.
    #[builder(setter(into))]
    #[serde(default)]
    pub timeline: ::std::vec::Vec<QueryTimelineSample>,
}

/// Statistics for an extract job.
#[derive(::serde::Deserialize, ::serde::Serialize, ::typed_builder::TypedBuilder, Debug, PartialEq, Clone)]
#[derive(Default)]
#[serde(rename_all = "camelCase")]
pub struct JobStatistics4 {
    /// Output only. Number of files per destination URI or URI pattern specified in the extract
    /// configuration. These values will be in the same order as the URIs specified in the
    /// 'destinationUris' field.
    #[builder(setter(into))]
    #[serde(default)]
    pub destination_uri_file_counts: ::std::vec::Vec<i64>,
    /// Output only. Number of user bytes extracted into the result. This is the byte count as computed
    /// by BigQuery for billing purposes and doesn't have any relationship with the number of actual
    /// result bytes extracted in the desired format.
    #[builder(setter(into))]
    #[serde(with = "with::int64")]
    #[serde(default)]
    pub input_bytes: i64,
    /// Output only. Describes a timeline of job execution.
    #[builder(setter(into))]
    #[serde(default)]
    pub timeline: ::std::vec::Vec<QueryTimelineSample>,
}

/// Statistics for a copy job.
#[derive(::serde::Deserialize, ::serde::Serialize, ::typed_builder::TypedBuilder, Debug, PartialEq, Clone)]
#[derive(Default)]
#[serde(rename_all = "camelCase")]
pub struct JobStatistics5 {
    /// Output only. Number of logical bytes copied to the destination table.
    #[builder(setter(into))]
    #[serde(with = "with::int64")]
    #[serde(default)]
    pub copied_logical_bytes: i64,
    /// Output only. Number of rows copied to the destination table.
    #[builder(setter(into))]
    #[serde(with = "with::int64")]
    #[serde(default)]
    pub copied_rows: i64,
}

#[derive(::serde::Deserialize, ::serde::Serialize, ::typed_builder::TypedBuilder, Debug, PartialEq, Clone)]
#[derive(Default)]
#[serde(rename_all = "camelCase")]
pub struct JobStatus {
    /// Output only. Final error result of the job. If present, indicates that the job has completed and
    /// was unsuccessful.
    #[builder(setter(into))]
    #[serde(default)]
    pub error_result: ErrorProto,
    /// Output only. The first errors encountered during the running of the job. The final message
    /// includes the number of errors that caused the process to stop. Errors here do not necessarily
    /// mean that the job has not completed or was unsuccessful.
    #[builder(setter(into))]
    #[serde(default)]
    pub errors: ::std::vec::Vec<ErrorProto>,
    /// Output only. Running state of the job. Valid states include 'PENDING', 'RUNNING', and 'DONE'.
    #[builder(setter(into))]
    #[serde(default)]
    pub state: ::std::string::String,
}

/// Represents a single JSON object.
pub type JsonObject = ::std::collections::HashMap<::std::string::String, ::serde_json::Value>;

/// Json Options for load and make external tables.
#[derive(::serde::Deserialize, ::serde::Serialize, ::typed_builder::TypedBuilder, Debug, PartialEq, Clone)]
#[derive(Default)]
#[serde(rename_all = "camelCase")]
pub struct JsonOptions {
    /// Optional. The character encoding of the data. The supported values are UTF-8, UTF-16BE,
    /// UTF-16LE, UTF-32BE, and UTF-32LE. The default value is UTF-8.
    #[builder(setter(into))]
    #[serde(default)]
    pub encoding: ::std::option::Option<::std::string::String>,
}

/// A dataset source type which refers to another BigQuery dataset.
#[derive(::serde::Deserialize, ::serde::Serialize, ::typed_builder::TypedBuilder, Debug, PartialEq, Clone)]
#[derive(Default)]
#[serde(rename_all = "camelCase")]
pub struct LinkedDatasetSource {
    /// The source dataset reference contains project numbers and not project ids.
    #[builder(setter(into))]
    #[serde(default)]
    pub source_dataset: DatasetReference,
}

/// Response format for a single page when listing BigQuery ML models.
#[derive(::serde::Deserialize, ::serde::Serialize, ::typed_builder::TypedBuilder, Debug, PartialEq, Clone)]
#[derive(Default)]
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

/// Describes the format of a single result page when listing routines.
#[derive(::serde::Deserialize, ::serde::Serialize, ::typed_builder::TypedBuilder, Debug, PartialEq, Clone)]
#[derive(Default)]
#[serde(rename_all = "camelCase")]
pub struct ListRoutinesResponse {
    /// A token to request the next page of results.
    #[builder(setter(into))]
    #[serde(default)]
    pub next_page_token: ::std::string::String,
    /// Routines in the requested dataset. Unless read_mask is set in the request, only the following
    /// fields are populated: etag, project_id, dataset_id, routine_id, routine_type, creation_time,
    /// last_modified_time, language, and remote_function_options.
    #[builder(setter(into))]
    #[serde(default)]
    pub routines: ::std::vec::Vec<Routine>,
}

/// Response message for the ListRowAccessPolicies method.
#[derive(::serde::Deserialize, ::serde::Serialize, ::typed_builder::TypedBuilder, Debug, PartialEq, Clone)]
#[derive(Default)]
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

/// Statistics for a LOAD query.
#[derive(::serde::Deserialize, ::serde::Serialize, ::typed_builder::TypedBuilder, Debug, PartialEq, Clone)]
#[derive(Default)]
#[serde(rename_all = "camelCase")]
pub struct LoadQueryStatistics {
    /// Output only. The number of bad records encountered while processing a LOAD query. Note that if
    /// the job has failed because of more bad records encountered than the maximum allowed in the load
    /// job configuration, then this number can be less than the total number of bad records present in
    /// the input data.
    #[builder(setter(into))]
    #[serde(with = "with::int64")]
    #[serde(default)]
    pub bad_records: i64,
    /// Output only. This field is deprecated. The number of bytes of source data copied over the
    /// network for a `LOAD` query. `transferred_bytes` has the canonical value for physical transferred
    /// bytes, which is used for BigQuery Omni billing.
    #[builder(setter(into))]
    #[serde(with = "with::int64")]
    #[serde(default)]
    pub bytes_transferred: i64,
    /// Output only. Number of bytes of source data in a LOAD query.
    #[builder(setter(into))]
    #[serde(with = "with::int64")]
    #[serde(default)]
    pub input_file_bytes: i64,
    /// Output only. Number of source files in a LOAD query.
    #[builder(setter(into))]
    #[serde(with = "with::int64")]
    #[serde(default)]
    pub input_files: i64,
    /// Output only. Size of the loaded data in bytes. Note that while a LOAD query is in the running
    /// state, this value may change.
    #[builder(setter(into))]
    #[serde(with = "with::int64")]
    #[serde(default)]
    pub output_bytes: i64,
    /// Output only. Number of rows imported in a LOAD query. Note that while a LOAD query is in the
    /// running state, this value may change.
    #[builder(setter(into))]
    #[serde(with = "with::int64")]
    #[serde(default)]
    pub output_rows: i64,
}

/// BigQuery-specific metadata about a location. This will be set on
/// google.cloud.location.Location.metadata in Cloud Location API responses.
#[derive(::serde::Deserialize, ::serde::Serialize, ::typed_builder::TypedBuilder, Debug, PartialEq, Clone)]
#[derive(Default)]
#[serde(rename_all = "camelCase")]
pub struct LocationMetadata {
    /// The legacy BigQuery location ID, e.g. “EU” for the “europe” location. This is for any
    /// API consumers that need the legacy “US” and “EU” locations.
    #[builder(setter(into))]
    #[serde(default)]
    pub legacy_location_id: ::std::string::String,
}

/// If present, specifies the reason why the materialized view was not chosen for the query.
#[derive(::serde::Deserialize, ::serde::Serialize, Debug, PartialEq, Clone)]
pub enum RejectedReason {
        /// View has no cached data because it has not refreshed yet.
    #[serde(rename = "NO_DATA")]
    NoData,
        /// The estimated cost of the view is more expensive than another view or the base table. Note:
        /// The estimate cost might not match the billed cost.
    #[serde(rename = "COST")]
    Cost,
        /// View has no cached data because a base table is truncated.
    #[serde(rename = "BASE_TABLE_TRUNCATED")]
    BaseTableTruncated,
        /// View is invalidated because of a data change in one or more base tables. It could be any
        /// recent change if the
        /// [`max_staleness`](https://cloud.google.com/bigquery/docs/materialized-views-create#max_staleness)
        /// option is not set for the view, or otherwise any change outside of the staleness window.
    #[serde(rename = "BASE_TABLE_DATA_CHANGE")]
    BaseTableDataChange,
        /// View is invalidated because a base table's partition expiration has changed.
    #[serde(rename = "BASE_TABLE_PARTITION_EXPIRATION_CHANGE")]
    BaseTablePartitionExpirationChange,
        /// View is invalidated because a base table's partition has expired.
    #[serde(rename = "BASE_TABLE_EXPIRED_PARTITION")]
    BaseTableExpiredPartition,
        /// View is invalidated because a base table has an incompatible metadata change.
    #[serde(rename = "BASE_TABLE_INCOMPATIBLE_METADATA_CHANGE")]
    BaseTableIncompatibleMetadataChange,
        /// View is invalidated because it was refreshed with a time zone other than that of the current
        /// job.
    #[serde(rename = "TIME_ZONE")]
    TimeZone,
        /// View is outside the time travel window.
    #[serde(rename = "OUT_OF_TIME_TRAVEL_WINDOW")]
    OutOfTimeTravelWindow,
        /// View is inaccessible to the user because of a fine-grained security policy on one of its
        /// base tables.
    #[serde(rename = "BASE_TABLE_FINE_GRAINED_SECURITY_POLICY")]
    BaseTableFineGrainedSecurityPolicy,
        /// One of the view's base tables is too stale. For example, the cached metadata of a biglake
        /// table needs to be updated.
    #[serde(rename = "BASE_TABLE_TOO_STALE")]
    BaseTableTooStale,
}

/// A materialized view considered for a query job.
#[derive(::serde::Deserialize, ::serde::Serialize, ::typed_builder::TypedBuilder, Debug, PartialEq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct MaterializedView {
    /// Whether the materialized view is chosen for the query. A materialized view can be chosen to
    /// rewrite multiple parts of the same query. If a materialized view is chosen to rewrite any part
    /// of the query, then this field is true, even if the materialized view was not chosen to rewrite
    /// others parts.
    #[builder(setter(into))]
    #[serde(default)]
    pub chosen: bool,
    /// If present, specifies a best-effort estimation of the bytes saved by using the materialized view
    /// rather than its base tables.
    #[builder(setter(into))]
    #[serde(with = "with::int64")]
    #[serde(default)]
    pub estimated_bytes_saved: i64,
    /// If present, specifies the reason why the materialized view was not chosen for the query.
    #[builder(setter(into))]
    pub rejected_reason: RejectedReason,
    /// The candidate materialized view.
    #[builder(setter(into))]
    #[serde(default)]
    pub table_reference: TableReference,
}

/// Definition and configuration of a materialized view.
#[derive(::serde::Deserialize, ::serde::Serialize, ::typed_builder::TypedBuilder, Debug, PartialEq, Clone)]
#[derive(Default)]
#[serde(rename_all = "camelCase")]
pub struct MaterializedViewDefinition {
    /// Optional. This option declares authors intention to construct a materialized view that will not
    /// be refreshed incrementally.
    #[builder(setter(into))]
    #[serde(default)]
    pub allow_non_incremental_definition: ::std::option::Option<bool>,
    /// Optional. Enable automatic refresh of the materialized view when the base table is updated. The
    /// default value is "true".
    #[builder(setter(into))]
    #[serde(default)]
    pub enable_refresh: ::std::option::Option<bool>,
    /// Output only. The time when this materialized view was last refreshed, in milliseconds since the
    /// epoch.
    #[builder(setter(into))]
    #[serde(with = "with::int64")]
    #[serde(default)]
    pub last_refresh_time: i64,
    /// [Optional] Max staleness of data that could be returned when materizlized view is queried
    /// (formatted as Google SQL Interval type).
    #[builder(setter(into))]
    #[serde(default)]
    pub max_staleness: ::std::option::Option<::std::vec::Vec<u8>>,
    /// Required. A query whose results are persisted.
    #[builder(setter(into))]
    #[serde(default)]
    pub query: ::std::string::String,
    /// Optional. The maximum frequency at which this materialized view will be refreshed. The default
    /// value is "1800000" (30 minutes).
    #[builder(setter(into))]
    #[serde(with = "with::int64::option")]
    #[serde(default)]
    pub refresh_interval_ms: ::std::option::Option<i64>,
}

/// Statistics of materialized views considered in a query job.
#[derive(::serde::Deserialize, ::serde::Serialize, ::typed_builder::TypedBuilder, Debug, PartialEq, Clone)]
#[derive(Default)]
#[serde(rename_all = "camelCase")]
pub struct MaterializedViewStatistics {
    /// Materialized views considered for the query job. Only certain materialized views are used. For a
    /// detailed list, see the child message. If many materialized views are considered, then the list
    /// might be incomplete.
    #[builder(setter(into))]
    #[serde(default)]
    pub materialized_view: ::std::vec::Vec<MaterializedView>,
}

/// Status of a materialized view. The last refresh timestamp status is omitted here, but is present
/// in the MaterializedViewDefinition message.
#[derive(::serde::Deserialize, ::serde::Serialize, ::typed_builder::TypedBuilder, Debug, PartialEq, Clone)]
#[derive(Default)]
#[serde(rename_all = "camelCase")]
pub struct MaterializedViewStatus {
    /// Output only. Error result of the last automatic refresh. If present, indicates that the last
    /// automatic refresh was unsuccessful.
    #[builder(setter(into))]
    #[serde(default)]
    pub last_refresh_status: ErrorProto,
    /// Output only. Refresh watermark of materialized view. The base tables' data were collected into
    /// the materialized view cache until this time.
    #[builder(setter(into))]
    #[serde(default)]
    pub refresh_watermark: ::std::string::String,
}

/// Statistics for metadata caching in BigLake tables.
#[derive(::serde::Deserialize, ::serde::Serialize, ::typed_builder::TypedBuilder, Debug, PartialEq, Clone)]
#[derive(Default)]
#[serde(rename_all = "camelCase")]
pub struct MetadataCacheStatistics {
    /// Set for the Metadata caching eligible tables referenced in the query.
    #[builder(setter(into))]
    #[serde(default)]
    pub table_metadata_cache_usage: ::std::vec::Vec<TableMetadataCacheUsage>,
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
        /// An imported XGBoost model.
    #[serde(rename = "XGBOOST")]
    Xgboost,
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
        /// ARIMA with external regressors.
    #[serde(rename = "ARIMA_PLUS_XREG")]
    ArimaPlusXreg,
        /// Random forest regressor model.
    #[serde(rename = "RANDOM_FOREST_REGRESSOR")]
    RandomForestRegressor,
        /// Random forest classifier model.
    #[serde(rename = "RANDOM_FOREST_CLASSIFIER")]
    RandomForestClassifier,
        /// An imported TensorFlow Lite model.
    #[serde(rename = "TENSORFLOW_LITE")]
    TensorflowLite,
        /// An imported ONNX model.
    #[serde(rename = "ONNX")]
    Onnx,
}

/// Output only. Training type of the job.
#[derive(::serde::Deserialize, ::serde::Serialize, Debug, PartialEq, Clone)]
pub enum TrainingType {
        /// Single training with fixed parameter space.
    #[serde(rename = "SINGLE_TRAINING")]
    SingleTraining,
        /// [Hyperparameter tuning
        /// training](/bigquery-ml/docs/reference/standard-sql/bigqueryml-syntax-hp-tuning-overview).
    #[serde(rename = "HPARAM_TUNING")]
    HparamTuning,
}

/// Job statistics specific to a BigQuery ML training job.
#[derive(::serde::Deserialize, ::serde::Serialize, ::typed_builder::TypedBuilder, Debug, PartialEq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct MlStatistics {
    /// Output only. Trials of a [hyperparameter tuning
    /// job](/bigquery-ml/docs/reference/standard-sql/bigqueryml-syntax-hp-tuning-overview) sorted by
    /// trial_id.
    #[builder(setter(into))]
    #[serde(default)]
    pub hparam_trials: ::std::vec::Vec<HparamTuningTrial>,
    /// Results for all completed iterations. Empty for [hyperparameter tuning
    /// jobs](/bigquery-ml/docs/reference/standard-sql/bigqueryml-syntax-hp-tuning-overview).
    #[builder(setter(into))]
    #[serde(default)]
    pub iteration_results: ::std::vec::Vec<IterationResult>,
    /// Output only. Maximum number of iterations specified as max_iterations in the 'CREATE MODEL'
    /// query. The actual number of iterations may be less than this number due to early stop.
    #[builder(setter(into))]
    #[serde(with = "with::int64")]
    #[serde(default)]
    pub max_iterations: i64,
    /// Output only. The type of the model that is being trained.
    #[builder(setter(into))]
    pub model_type: ModelType,
    /// Output only. Training type of the job.
    #[builder(setter(into))]
    pub training_type: TrainingType,
}

#[derive(::serde::Deserialize, ::serde::Serialize, ::typed_builder::TypedBuilder, Debug, PartialEq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Model {
    /// The best trial_id across all training runs.
    #[builder(setter(into))]
    #[serde(with = "with::int64")]
    #[serde(default)]
    pub best_trial_id: i64,
    /// Output only. The time when this model was created, in millisecs since the epoch.
    #[builder(setter(into))]
    #[serde(with = "with::int64")]
    #[serde(default)]
    pub creation_time: i64,
    /// Output only. The default trial_id to use in TVFs when the trial_id is not passed in. For
    /// single-objective [hyperparameter
    /// tuning](/bigquery-ml/docs/reference/standard-sql/bigqueryml-syntax-hp-tuning-overview) models,
    /// this is the best trial ID. For multi-objective [hyperparameter
    /// tuning](/bigquery-ml/docs/reference/standard-sql/bigqueryml-syntax-hp-tuning-overview) models,
    /// this is the smallest trial ID among all Pareto optimal trials.
    #[builder(setter(into))]
    #[serde(with = "with::int64")]
    #[serde(default)]
    pub default_trial_id: i64,
    /// Optional. A user-friendly description of this model.
    #[builder(setter(into))]
    #[serde(default)]
    pub description: ::std::option::Option<::std::string::String>,
    /// Custom encryption configuration (e.g., Cloud KMS keys). This shows the encryption configuration
    /// of the model data while stored in BigQuery storage. This field can be used with PatchModel to
    /// update encryption key for an already encrypted model.
    #[builder(setter(into))]
    #[serde(default)]
    pub encryption_configuration: EncryptionConfiguration,
    /// Output only. A hash of this resource.
    #[builder(setter(into))]
    #[serde(default)]
    pub etag: ::std::string::String,
    /// Optional. The time when this model expires, in milliseconds since the epoch. If not present, the
    /// model will persist indefinitely. Expired models will be deleted and their storage reclaimed. The
    /// defaultTableExpirationMs property of the encapsulating dataset can be used to set a default
    /// expirationTime on newly created models.
    #[builder(setter(into))]
    #[serde(with = "with::int64::option")]
    #[serde(default)]
    pub expiration_time: ::std::option::Option<i64>,
    /// Output only. Input feature columns for the model inference. If the model is trained with
    /// TRANSFORM clause, these are the input of the TRANSFORM clause.
    #[builder(setter(into))]
    #[serde(default)]
    pub feature_columns: ::std::vec::Vec<StandardSqlField>,
    /// Optional. A descriptive name for this model.
    #[builder(setter(into))]
    #[serde(default)]
    pub friendly_name: ::std::option::Option<::std::string::String>,
    /// Output only. All hyperparameter search spaces in this model.
    #[builder(setter(into))]
    pub hparam_search_spaces: HparamSearchSpaces,
    /// Output only. Trials of a [hyperparameter
    /// tuning](/bigquery-ml/docs/reference/standard-sql/bigqueryml-syntax-hp-tuning-overview) model
    /// sorted by trial_id.
    #[builder(setter(into))]
    #[serde(default)]
    pub hparam_trials: ::std::vec::Vec<HparamTuningTrial>,
    /// Output only. Label columns that were used to train this model. The output of the model will have
    /// a "predicted_" prefix to these columns.
    #[builder(setter(into))]
    #[serde(default)]
    pub label_columns: ::std::vec::Vec<StandardSqlField>,
    /// The labels associated with this model. You can use these to organize and group your models.
    /// Label keys and values can be no longer than 63 characters, can only contain lowercase letters,
    /// numeric characters, underscores and dashes. International characters are allowed. Label values
    /// are optional. Label keys must start with a letter and each label in the list must have a
    /// different key.
    #[builder(setter(into))]
    #[serde(default)]
    pub labels: Labels,
    /// Output only. The time when this model was last modified, in millisecs since the epoch.
    #[builder(setter(into))]
    #[serde(with = "with::int64")]
    #[serde(default)]
    pub last_modified_time: i64,
    /// Output only. The geographic location where the model resides. This value is inherited from the
    /// dataset.
    #[builder(setter(into))]
    #[serde(default)]
    pub location: ::std::string::String,
    /// Required. Unique identifier for this model.
    #[builder(setter(into))]
    #[serde(default)]
    pub model_reference: ModelReference,
    /// Output only. Type of the model resource.
    #[builder(setter(into))]
    pub model_type: ModelType,
    /// Output only. For single-objective [hyperparameter
    /// tuning](/bigquery-ml/docs/reference/standard-sql/bigqueryml-syntax-hp-tuning-overview) models,
    /// it only contains the best trial. For multi-objective [hyperparameter
    /// tuning](/bigquery-ml/docs/reference/standard-sql/bigqueryml-syntax-hp-tuning-overview) models,
    /// it contains all Pareto optimal trials sorted by trial_id.
    #[builder(setter(into))]
    #[serde(default)]
    pub optimal_trial_ids: ::std::vec::Vec<i64>,
    /// Output only. Remote model info
    #[builder(setter(into))]
    pub remote_model_info: RemoteModelInfo,
    /// Information for all training runs in increasing order of start_time.
    #[builder(setter(into))]
    #[serde(default)]
    pub training_runs: ::std::vec::Vec<TrainingRun>,
    /// Output only. This field will be populated if a TRANSFORM clause was used to train a model.
    /// TRANSFORM clause (if used) takes feature_columns as input and outputs transform_columns.
    /// transform_columns then are used to train the model.
    #[builder(setter(into))]
    #[serde(default)]
    pub transform_columns: ::std::vec::Vec<TransformColumn>,
}

/// Deprecated.
#[derive(::serde::Deserialize, ::serde::Serialize, ::typed_builder::TypedBuilder, Debug, PartialEq, Clone)]
#[derive(Default)]
#[serde(rename_all = "camelCase")]
pub struct ModelOptions {
    #[builder(setter(into))]
    #[serde(default)]
    pub labels: ::std::vec::Vec<::std::string::String>,
    #[builder(setter(into))]
    #[serde(default)]
    pub loss_type: ::std::string::String,
    #[builder(setter(into))]
    #[serde(default)]
    pub model_type: ::std::string::String,
}

#[derive(::serde::Deserialize, ::serde::Serialize, ::typed_builder::TypedBuilder, Debug, PartialEq, Clone)]
#[derive(Default)]
#[serde(rename_all = "camelCase")]
pub struct ModelDefinition {
    /// Deprecated.
    #[builder(setter(into))]
    #[serde(default)]
    pub model_options: ModelOptions,
    /// Deprecated.
    #[builder(setter(into))]
    #[serde(default)]
    pub training_runs: ::std::vec::Vec<BqmlTrainingRun>,
}

/// Options related to model extraction.
#[derive(::serde::Deserialize, ::serde::Serialize, ::typed_builder::TypedBuilder, Debug, PartialEq, Clone)]
#[derive(Default)]
#[serde(rename_all = "camelCase")]
pub struct ModelExtractOptions {
    /// The 1-based ID of the trial to be exported from a hyperparameter tuning model. If not specified,
    /// the trial with id =
    /// [Model](/bigquery/docs/reference/rest/v2/models#resource:-model).defaultTrialId is exported.
    /// This field is ignored for models not trained with hyperparameter tuning.
    #[builder(setter(into))]
    #[serde(with = "with::int64")]
    #[serde(default)]
    pub trial_id: i64,
}

/// Id path of a model.
#[derive(::serde::Deserialize, ::serde::Serialize, ::typed_builder::TypedBuilder, Debug, PartialEq, Clone)]
#[derive(Default)]
#[serde(rename_all = "camelCase")]
pub struct ModelReference {
    /// Required. The ID of the dataset containing this model.
    #[builder(setter(into))]
    #[serde(default)]
    pub dataset_id: ::std::string::String,
    /// Required. The ID of the model. The ID must contain only letters (a-z, A-Z), numbers (0-9), or
    /// underscores (_). The maximum length is 1,024 characters.
    #[builder(setter(into))]
    #[serde(default)]
    pub model_id: ::std::string::String,
    /// Required. The ID of the project containing this model.
    #[builder(setter(into))]
    #[serde(default)]
    pub project_id: ::std::string::String,
}

/// Evaluation metrics for multi-class classification/classifier models.
#[derive(::serde::Deserialize, ::serde::Serialize, ::typed_builder::TypedBuilder, Debug, PartialEq, Clone)]
#[derive(Default)]
#[serde(rename_all = "camelCase")]
pub struct MultiClassClassificationMetrics {
    /// Aggregate classification metrics.
    #[builder(setter(into))]
    #[serde(default)]
    pub aggregate_classification_metrics: AggregateClassificationMetrics,
    /// Confusion matrix at different thresholds.
    #[builder(setter(into))]
    #[serde(default)]
    pub confusion_matrix_list: ::std::vec::Vec<ConfusionMatrix>,
}

/// Parquet Options for load and make external tables.
#[derive(::serde::Deserialize, ::serde::Serialize, ::typed_builder::TypedBuilder, Debug, PartialEq, Clone)]
#[derive(Default)]
#[serde(rename_all = "camelCase")]
pub struct ParquetOptions {
    /// Optional. Indicates whether to use schema inference specifically for Parquet LIST logical type.
    #[builder(setter(into))]
    #[serde(default)]
    pub enable_list_inference: ::std::option::Option<bool>,
    /// Optional. Indicates whether to infer Parquet ENUM logical type as STRING instead of BYTES by
    /// default.
    #[builder(setter(into))]
    #[serde(default)]
    pub enum_as_string: ::std::option::Option<bool>,
}

/// Performance insights for the job.
#[derive(::serde::Deserialize, ::serde::Serialize, ::typed_builder::TypedBuilder, Debug, PartialEq, Clone)]
#[derive(Default)]
#[serde(rename_all = "camelCase")]
pub struct PerformanceInsights {
    /// Output only. Average execution ms of previous runs. Indicates the job ran slow compared to
    /// previous executions. To find previous executions, use INFORMATION_SCHEMA tables and filter jobs
    /// with same query hash.
    #[builder(setter(into))]
    #[serde(with = "with::int64")]
    #[serde(default)]
    pub avg_previous_execution_ms: i64,
    /// Output only. Query stage performance insights compared to previous runs, for diagnosing
    /// performance regression.
    #[builder(setter(into))]
    #[serde(default)]
    pub stage_performance_change_insights: ::std::vec::Vec<StagePerformanceChangeInsight>,
    /// Output only. Standalone query stage performance insights, for exploring potential improvements.
    #[builder(setter(into))]
    #[serde(default)]
    pub stage_performance_standalone_insights: ::std::vec::Vec<StagePerformanceStandaloneInsight>,
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
/// example:** ``` { "bindings": [ { "role": "roles/resourcemanager.organizationAdmin", "members": [
/// "user:mike@example.com", "group:admins@example.com", "domain:google.com",
/// "serviceAccount:my-project-id@appspot.gserviceaccount.com" ] }, { "role":
/// "roles/resourcemanager.organizationViewer", "members": [ "user:eve@example.com" ], "condition":
/// { "title": "expirable access", "description": "Does not grant access after Sep 2020",
/// "expression": "request.time < timestamp('2020-10-01T00:00:00.000Z')", } } ], "etag":
/// "BwWWja0YfJA=", "version": 3 } ``` **YAML example:** ``` bindings: - members: -
/// user:mike@example.com - group:admins@example.com - domain:google.com -
/// serviceAccount:my-project-id@appspot.gserviceaccount.com role:
/// roles/resourcemanager.organizationAdmin - members: - user:eve@example.com role:
/// roles/resourcemanager.organizationViewer condition: title: expirable access description: Does
/// not grant access after Sep 2020 expression: request.time < timestamp('2020-10-01T00:00:00.000Z')
/// etag: BwWWja0YfJA= version: 3 ``` For a description of IAM and its features, see the [IAM
/// documentation](https://cloud.google.com/iam/docs/).
#[derive(::serde::Deserialize, ::serde::Serialize, ::typed_builder::TypedBuilder, Debug, PartialEq, Clone)]
#[derive(Default)]
#[serde(rename_all = "camelCase")]
pub struct Policy {
    /// Specifies cloud audit logging configuration for this policy.
    #[builder(setter(into))]
    #[serde(default)]
    pub audit_configs: ::std::vec::Vec<AuditConfig>,
    /// Associates a list of `members`, or principals, with a `role`. Optionally, may specify a
    /// `condition` that determines how and when the `bindings` are applied. Each of the `bindings` must
    /// contain at least one principal. The `bindings` in a `Policy` can refer to up to 1,500
    /// principals; up to 250 of these principals can be Google groups. Each occurrence of a principal
    /// counts towards these limits. For example, if the `bindings` grant 50 different roles to
    /// `user:alice@example.com`, and not to any other principal, then you can add another 1,450
    /// principals to the `bindings` in the `Policy`.
    #[builder(setter(into))]
    #[serde(default)]
    pub bindings: ::std::vec::Vec<Binding>,
    /// `etag` is used for optimistic concurrency control as a way to help prevent simultaneous updates
    /// of a policy from overwriting each other. It is strongly suggested that systems make use of the
    /// `etag` in the read-modify-write cycle to perform policy updates in order to avoid race
    /// conditions: An `etag` is returned in the response to `getIamPolicy`, and systems are expected to
    /// put that etag in the request to `setIamPolicy` to ensure that their change will be applied to
    /// the same version of the policy. **Important:** If you use IAM Conditions, you must include the
    /// `etag` field whenever you call `setIamPolicy`. If you omit this field, then IAM allows you to
    /// overwrite a version `3` policy with a version `1` policy, and all of the conditions in the
    /// version `3` policy are lost.
    #[builder(setter(into))]
    #[serde(default)]
    pub etag: ::std::vec::Vec<u8>,
    /// Specifies the format of the policy. Valid values are `0`, `1`, and `3`. Requests that specify an
    /// invalid value are rejected. Any operation that affects conditional role bindings must specify
    /// version `3`. This requirement applies to the following operations: * Getting a policy that
    /// includes a conditional role binding * Adding a conditional role binding to a policy * Changing a
    /// conditional role binding in a policy * Removing any role binding, with or without a condition,
    /// from a policy that includes conditions **Important:** If you use IAM Conditions, you must
    /// include the `etag` field whenever you call `setIamPolicy`. If you omit this field, then IAM
    /// allows you to overwrite a version `3` policy with a version `1` policy, and all of the
    /// conditions in the version `3` policy are lost. If a policy does not include any conditions,
    /// operations on that policy may specify any valid version or leave the field unset. To learn which
    /// resources support conditions in their IAM policies, see the [IAM
    /// documentation](https://cloud.google.com/iam/help/conditions/resource-policies).
    #[builder(setter(into))]
    #[serde(default)]
    pub version: i64,
}

/// Principal component infos, used only for eigen decomposition based models, e.g., PCA. Ordered by
/// explained_variance in the descending order.
#[derive(::serde::Deserialize, ::serde::Serialize, ::typed_builder::TypedBuilder, Debug, PartialEq, Clone)]
#[derive(Default)]
#[serde(rename_all = "camelCase")]
pub struct PrincipalComponentInfo {
    /// The explained_variance is pre-ordered in the descending order to compute the cumulative
    /// explained variance ratio.
    #[builder(setter(into))]
    #[serde(default)]
    pub cumulative_explained_variance_ratio: f64,
    /// Explained variance by this principal component, which is simply the eigenvalue.
    #[builder(setter(into))]
    #[serde(default)]
    pub explained_variance: f64,
    /// Explained_variance over the total explained variance.
    #[builder(setter(into))]
    #[serde(default)]
    pub explained_variance_ratio: f64,
    /// Id of the principal component.
    #[builder(setter(into))]
    #[serde(with = "with::int64")]
    #[serde(default)]
    pub principal_component_id: i64,
}

/// Represents privacy policy that contains the privacy requirements specified by the data owner.
/// Currently, this is only supported on views.
#[derive(::serde::Deserialize, ::serde::Serialize, ::typed_builder::TypedBuilder, Debug, PartialEq, Clone)]
#[derive(Default)]
#[serde(rename_all = "camelCase")]
pub struct PrivacyPolicy {
    /// Optional. Policy used for aggregation thresholds.
    #[builder(setter(into))]
    #[serde(default)]
    pub aggregation_threshold_policy: AggregationThresholdPolicy,
}

/// Information about a single project.
#[derive(::serde::Deserialize, ::serde::Serialize, ::typed_builder::TypedBuilder, Debug, PartialEq, Clone)]
#[derive(Default)]
#[serde(rename_all = "camelCase")]
pub struct Projects {
    /// A descriptive name for this project. A wrapper is used here because friendlyName can be set to
    /// the empty string.
    #[builder(setter(into))]
    #[serde(default)]
    pub friendly_name: ::std::string::String,
    /// An opaque ID of this project.
    #[builder(setter(into))]
    #[serde(default)]
    pub id: ::std::string::String,
    /// The resource type.
    #[builder(setter(into))]
    #[serde(default)]
    pub kind: ::std::string::String,
    /// The numeric ID of this project.
    #[builder(setter(into))]
    #[serde(with = "with::uint64")]
    #[serde(default)]
    pub numeric_id: u64,
    /// A unique reference to this project.
    #[builder(setter(into))]
    #[serde(default)]
    pub project_reference: ProjectReference,
}

/// Response object of ListProjects
#[derive(::serde::Deserialize, ::serde::Serialize, ::typed_builder::TypedBuilder, Debug, PartialEq, Clone)]
#[derive(Default)]
#[serde(rename_all = "camelCase")]
pub struct ProjectList {
    /// A hash of the page of results.
    #[builder(setter(into))]
    #[serde(default)]
    pub etag: ::std::string::String,
    /// The resource type of the response.
    #[builder(setter(into))]
    #[serde(default)]
    pub kind: ::std::string::String,
    /// Use this token to request the next page of results.
    #[builder(setter(into))]
    #[serde(default)]
    pub next_page_token: ::std::string::String,
    /// Projects to which the user has at least READ access.
    #[builder(setter(into))]
    #[serde(default)]
    pub projects: ::std::vec::Vec<Projects>,
    /// The total number of projects in the page. A wrapper is used here because the field should still
    /// be in the response when the value is 0.
    #[builder(setter(into))]
    #[serde(default)]
    pub total_items: i64,
}

/// A unique reference to a project.
#[derive(::serde::Deserialize, ::serde::Serialize, ::typed_builder::TypedBuilder, Debug, PartialEq, Clone)]
#[derive(Default)]
#[serde(rename_all = "camelCase")]
pub struct ProjectReference {
    /// Required. ID of the project. Can be either the numeric ID or the assigned ID of the project.
    #[builder(setter(into))]
    #[serde(default)]
    pub project_id: ::std::string::String,
}

/// Output only. Information about query optimizations.
pub type OptimizationDetails = ::std::collections::HashMap<::std::string::String, ::serde_json::Value>;

/// Query optimization information for a QUERY job.
#[derive(::serde::Deserialize, ::serde::Serialize, ::typed_builder::TypedBuilder, Debug, PartialEq, Clone)]
#[derive(Default)]
#[serde(rename_all = "camelCase")]
pub struct QueryInfo {
    /// Output only. Information about query optimizations.
    #[builder(setter(into))]
    #[serde(default)]
    pub optimization_details: OptimizationDetails,
}

/// A parameter given to a query.
#[derive(::serde::Deserialize, ::serde::Serialize, ::typed_builder::TypedBuilder, Debug, PartialEq, Clone)]
#[derive(Default)]
#[serde(rename_all = "camelCase")]
pub struct QueryParameter {
    /// Optional. If unset, this is a positional parameter. Otherwise, should be unique within a query.
    #[builder(setter(into))]
    #[serde(default)]
    pub name: ::std::option::Option<::std::string::String>,
    /// Required. The type of this parameter.
    #[builder(setter(into))]
    #[serde(default)]
    pub parameter_type: QueryParameterType,
    /// Required. The value of this parameter.
    #[builder(setter(into))]
    #[serde(default)]
    pub parameter_value: QueryParameterValue,
}

/// The type of a struct parameter.
#[derive(::serde::Deserialize, ::serde::Serialize, ::typed_builder::TypedBuilder, Debug, PartialEq, Clone)]
#[derive(Default)]
#[serde(rename_all = "camelCase")]
pub struct StructTypes {
    /// Optional. Human-oriented description of the field.
    #[builder(setter(into))]
    #[serde(default)]
    pub description: ::std::option::Option<::std::string::String>,
    /// Optional. The name of this field.
    #[builder(setter(into))]
    #[serde(default)]
    pub name: ::std::option::Option<::std::string::String>,
    /// Required. The type of this field.
    #[builder(setter(into))]
    #[serde(rename = "type")]
    #[serde(default)]
    pub ty: QueryParameterType,
}

/// The type of a query parameter.
#[derive(::serde::Deserialize, ::serde::Serialize, ::typed_builder::TypedBuilder, Debug, PartialEq, Clone)]
#[derive(Default)]
#[serde(rename_all = "camelCase")]
pub struct QueryParameterType {
    /// Optional. The type of the array's elements, if this is an array.
    #[builder(setter(into))]
    #[serde(default)]
    pub array_type: ::std::option::Option<::std::boxed::Box<QueryParameterType>>,
    /// Optional. The element type of the range, if this is a range.
    #[builder(setter(into))]
    #[serde(default)]
    pub range_element_type: ::std::option::Option<::std::boxed::Box<QueryParameterType>>,
    /// Optional. The types of the fields of this struct, in order, if this is a struct.
    #[builder(setter(into))]
    #[serde(default)]
    pub struct_types: ::std::vec::Vec<StructTypes>,
    /// Required. The top level type of this field.
    #[builder(setter(into))]
    #[serde(rename = "type")]
    #[serde(default)]
    pub ty: ::std::string::String,
}

/// The struct field values.
pub type StructValues = ::std::collections::HashMap<::std::string::String, QueryParameterValue>;

/// The value of a query parameter.
#[derive(::serde::Deserialize, ::serde::Serialize, ::typed_builder::TypedBuilder, Debug, PartialEq, Clone)]
#[derive(Default)]
#[serde(rename_all = "camelCase")]
pub struct QueryParameterValue {
    /// Optional. The array values, if this is an array type.
    #[builder(setter(into))]
    #[serde(default)]
    pub array_values: ::std::vec::Vec<QueryParameterValue>,
    /// Optional. The range value, if this is a range type.
    #[builder(setter(into))]
    #[serde(default)]
    pub range_value: ::std::option::Option<RangeValue>,
    /// The struct field values.
    #[builder(setter(into))]
    #[serde(default)]
    pub struct_values: StructValues,
    /// Optional. The value of this value, if a simple scalar type.
    #[builder(setter(into))]
    #[serde(default)]
    pub value: ::std::option::Option<::std::string::String>,
}

/// Optional. If not set, jobs are always required. If set, the query request will follow the
/// behavior described JobCreationMode. This feature is not yet available. Jobs will always be
/// created.
#[derive(::serde::Deserialize, ::serde::Serialize, Debug, PartialEq, Clone)]
pub enum JobCreationMode {
        /// Default. Job creation is always required.
    #[serde(rename = "JOB_CREATION_REQUIRED")]
    JobCreationRequired,
        /// Job creation is optional. Returning immediate results is prioritized. BigQuery will
        /// automatically determine if a Job needs to be created. The conditions under which BigQuery
        /// can decide to not create a Job are subject to change. If Job creation is required,
        /// JOB_CREATION_REQUIRED mode should be used, which is the default.
    #[serde(rename = "JOB_CREATION_OPTIONAL")]
    JobCreationOptional,
}

/// Describes the format of the jobs.query request.
#[derive(::serde::Deserialize, ::serde::Serialize, ::typed_builder::TypedBuilder, Debug, PartialEq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct QueryRequest {
    /// Optional. Connection properties which can modify the query behavior.
    #[builder(setter(into))]
    #[serde(default)]
    pub connection_properties: ::std::vec::Vec<ConnectionProperty>,
    /// [Optional] Specifies whether the query should be executed as a continuous query. The default
    /// value is false.
    #[builder(setter(into))]
    #[serde(default)]
    pub continuous: ::std::option::Option<bool>,
    /// Optional. If true, creates a new session using a randomly generated session_id. If false, runs
    /// query with an existing session_id passed in ConnectionProperty, otherwise runs query in
    /// non-session mode. The session location will be set to QueryRequest.location if it is present,
    /// otherwise it's set to the default location based on existing routing logic.
    #[builder(setter(into))]
    #[serde(default)]
    pub create_session: ::std::option::Option<bool>,
    /// Optional. Specifies the default datasetId and projectId to assume for any unqualified table
    /// names in the query. If not set, all table names in the query string must be qualified in the
    /// format 'datasetId.tableId'.
    #[builder(setter(into))]
    #[serde(default)]
    pub default_dataset: DatasetReference,
    /// Optional. If set to true, BigQuery doesn't run the job. Instead, if the query is valid, BigQuery
    /// returns statistics about the job such as how many bytes would be processed. If the query is
    /// invalid, an error returns. The default value is false.
    #[builder(setter(into))]
    #[serde(default)]
    pub dry_run: ::std::option::Option<bool>,
    /// Optional. Output format adjustments.
    #[builder(setter(into))]
    #[serde(default)]
    pub format_options: DataFormatOptions,
    /// Optional. If not set, jobs are always required. If set, the query request will follow the
    /// behavior described JobCreationMode. This feature is not yet available. Jobs will always be
    /// created.
    #[builder(setter(into))]
    pub job_creation_mode: JobCreationMode,
    /// The resource type of the request.
    #[builder(setter(into))]
    #[serde(default)]
    pub kind: ::std::string::String,
    /// Optional. The labels associated with this query. Labels can be used to organize and group query
    /// jobs. Label keys and values can be no longer than 63 characters, can only contain lowercase
    /// letters, numeric characters, underscores and dashes. International characters are allowed. Label
    /// keys must start with a letter and each label in the list must have a different key.
    #[builder(setter(into))]
    #[serde(default)]
    pub labels: Labels,
    /// The geographic location where the job should run. See details at
    /// https://cloud.google.com/bigquery/docs/locations#specifying_your_location.
    #[builder(setter(into))]
    #[serde(default)]
    pub location: ::std::string::String,
    /// Optional. The maximum number of rows of data to return per page of results. Setting this flag to
    /// a small value such as 1000 and then paging through results might improve reliability when the
    /// query result set is large. In addition to this limit, responses are also limited to 10 MB. By
    /// default, there is no maximum row count, and only the byte limit applies.
    #[builder(setter(into))]
    #[serde(default)]
    pub max_results: ::std::option::Option<i64>,
    /// Optional. Limits the bytes billed for this query. Queries with bytes billed above this limit
    /// will fail (without incurring a charge). If unspecified, the project default is used.
    #[builder(setter(into))]
    #[serde(with = "with::int64::option")]
    #[serde(default)]
    pub maximum_bytes_billed: ::std::option::Option<i64>,
    /// GoogleSQL only. Set to POSITIONAL to use positional (?) query parameters or to NAMED to use
    /// named (@myparam) query parameters in this query.
    #[builder(setter(into))]
    #[serde(default)]
    pub parameter_mode: ::std::string::String,
    /// This property is deprecated.
    #[builder(setter(into))]
    #[serde(default)]
    pub preserve_nulls: bool,
    /// Required. A query string to execute, using Google Standard SQL or legacy SQL syntax. Example:
    /// "SELECT COUNT(f1) FROM myProjectId.myDatasetId.myTableId".
    #[builder(setter(into))]
    #[serde(default)]
    pub query: ::std::string::String,
    /// Query parameters for GoogleSQL queries.
    #[builder(setter(into))]
    #[serde(default)]
    pub query_parameters: ::std::vec::Vec<QueryParameter>,
    /// Optional. A unique user provided identifier to ensure idempotent behavior for queries. Note that
    /// this is different from the job_id. It has the following properties: 1. It is case-sensitive,
    /// limited to up to 36 ASCII characters. A UUID is recommended. 2. Read only queries can ignore
    /// this token since they are nullipotent by definition. 3. For the purposes of idempotency ensured
    /// by the request_id, a request is considered duplicate of another only if they have the same
    /// request_id and are actually duplicates. When determining whether a request is a duplicate of
    /// another request, all parameters in the request that may affect the result are considered. For
    /// example, query, connection_properties, query_parameters, use_legacy_sql are parameters that
    /// affect the result and are considered when determining whether a request is a duplicate, but
    /// properties like timeout_ms don't affect the result and are thus not considered. Dry run query
    /// requests are never considered duplicate of another request. 4. When a duplicate mutating query
    /// request is detected, it returns: a. the results of the mutation if it completes successfully
    /// within the timeout. b. the running operation if it is still in progress at the end of the
    /// timeout. 5. Its lifetime is limited to 15 minutes. In other words, if two requests are sent with
    /// the same request_id, but more than 15 minutes apart, idempotency is not guaranteed.
    #[builder(setter(into))]
    #[serde(default)]
    pub request_id: ::std::option::Option<::std::string::String>,
    /// Optional. Optional: Specifies the maximum amount of time, in milliseconds, that the client is
    /// willing to wait for the query to complete. By default, this limit is 10 seconds (10,000
    /// milliseconds). If the query is complete, the jobComplete field in the response is true. If the
    /// query has not yet completed, jobComplete is false. You can request a longer timeout period in
    /// the timeoutMs field. However, the call is not guaranteed to wait for the specified timeout; it
    /// typically returns after around 200 seconds (200,000 milliseconds), even if the query is not
    /// complete. If jobComplete is false, you can continue to wait for the query to complete by calling
    /// the getQueryResults method until the jobComplete field in the getQueryResults response is true.
    #[builder(setter(into))]
    #[serde(default)]
    pub timeout_ms: ::std::option::Option<i64>,
    /// Specifies whether to use BigQuery's legacy SQL dialect for this query. The default value is
    /// true. If set to false, the query will use BigQuery's GoogleSQL:
    /// https://cloud.google.com/bigquery/sql-reference/ When useLegacySql is set to false, the value of
    /// flattenResults is ignored; query will be run as if flattenResults is false.
    #[builder(setter(into))]
    #[serde(default)]
    pub use_legacy_sql: bool,
    /// Optional. Whether to look for the result in the query cache. The query cache is a best-effort
    /// cache that will be flushed whenever tables in the query are modified. The default value is true.
    #[builder(setter(into))]
    #[serde(default)]
    pub use_query_cache: ::std::option::Option<bool>,
}

#[derive(::serde::Deserialize, ::serde::Serialize, ::typed_builder::TypedBuilder, Debug, PartialEq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct QueryResponse {
    /// Whether the query result was fetched from the query cache.
    #[builder(setter(into))]
    #[serde(default)]
    pub cache_hit: bool,
    /// Output only. Detailed statistics for DML statements INSERT, UPDATE, DELETE, MERGE or TRUNCATE.
    #[builder(setter(into))]
    #[serde(default)]
    pub dml_stats: DmlStatistics,
    /// Output only. The first errors or warnings encountered during the running of the job. The final
    /// message includes the number of errors that caused the process to stop. Errors here do not
    /// necessarily mean that the job has completed or was unsuccessful. For more information about
    /// error messages, see [Error messages](https://cloud.google.com/bigquery/docs/error-messages).
    #[builder(setter(into))]
    #[serde(default)]
    pub errors: ::std::vec::Vec<ErrorProto>,
    /// Whether the query has completed or not. If rows or totalRows are present, this will always be
    /// true. If this is false, totalRows will not be available.
    #[builder(setter(into))]
    #[serde(default)]
    pub job_complete: bool,
    /// Optional. Only relevant when a job_reference is present in the response. If job_reference is not
    /// present it will always be unset. When job_reference is present, this field should be interpreted
    /// as follows: If set, it will provide the reason of why a Job was created. If not set, it should
    /// be treated as the default: REQUESTED. This feature is not yet available. Jobs will always be
    /// created.
    #[builder(setter(into))]
    pub job_creation_reason: JobCreationReason,
    /// Reference to the Job that was created to run the query. This field will be present even if the
    /// original request timed out, in which case GetQueryResults can be used to read the results once
    /// the query has completed. Since this API only returns the first page of results, subsequent pages
    /// can be fetched via the same mechanism (GetQueryResults).
    #[builder(setter(into))]
    #[serde(default)]
    pub job_reference: JobReference,
    /// The resource type.
    #[builder(setter(into))]
    #[serde(default)]
    pub kind: ::std::string::String,
    /// Output only. The number of rows affected by a DML statement. Present only for DML statements
    /// INSERT, UPDATE or DELETE.
    #[builder(setter(into))]
    #[serde(with = "with::int64")]
    #[serde(default)]
    pub num_dml_affected_rows: i64,
    /// A token used for paging results. A non-empty token indicates that additional results are
    /// available. To see additional results, query the
    /// [`jobs.getQueryResults`](https://cloud.google.com/bigquery/docs/reference/rest/v2/jobs/getQueryResults)
    /// method. For more information, see [Paging through table
    /// data](https://cloud.google.com/bigquery/docs/paging-results).
    #[builder(setter(into))]
    #[serde(default)]
    pub page_token: ::std::string::String,
    /// Query ID for the completed query. This ID will be auto-generated. This field is not yet
    /// available and it is currently not guaranteed to be populated.
    #[builder(setter(into))]
    #[serde(default)]
    pub query_id: ::std::string::String,
    /// An object with as many results as can be contained within the maximum permitted reply size. To
    /// get any additional rows, you can call GetQueryResults and specify the jobReference returned
    /// above.
    #[builder(setter(into))]
    #[serde(default)]
    pub rows: ::std::vec::Vec<TableRow>,
    /// The schema of the results. Present only when the query completes successfully.
    #[builder(setter(into))]
    #[serde(default)]
    pub schema: TableSchema,
    /// Output only. Information of the session if this job is part of one.
    #[builder(setter(into))]
    #[serde(default)]
    pub session_info: SessionInfo,
    /// The total number of bytes processed for this query. If this query was a dry run, this is the
    /// number of bytes that would be processed if the query were run.
    #[builder(setter(into))]
    #[serde(with = "with::int64")]
    #[serde(default)]
    pub total_bytes_processed: i64,
    /// The total number of rows in the complete query result set, which can be more than the number of
    /// rows in this single page of results.
    #[builder(setter(into))]
    #[serde(with = "with::uint64")]
    #[serde(default)]
    pub total_rows: u64,
}

/// Summary of the state of query execution at a given time.
#[derive(::serde::Deserialize, ::serde::Serialize, ::typed_builder::TypedBuilder, Debug, PartialEq, Clone)]
#[derive(Default)]
#[serde(rename_all = "camelCase")]
pub struct QueryTimelineSample {
    /// Total number of active workers. This does not correspond directly to slot usage. This is the
    /// largest value observed since the last sample.
    #[builder(setter(into))]
    #[serde(with = "with::int64")]
    #[serde(default)]
    pub active_units: i64,
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
    /// Units of work that can be scheduled immediately. Providing additional slots for these units of
    /// work will accelerate the query, if no other query in the reservation needs additional slots.
    #[builder(setter(into))]
    #[serde(with = "with::int64")]
    #[serde(default)]
    pub estimated_runnable_units: i64,
    /// Total units of work remaining for the query. This number can be revised (increased or decreased)
    /// while the query is running.
    #[builder(setter(into))]
    #[serde(with = "with::int64")]
    #[serde(default)]
    pub pending_units: i64,
    /// Cumulative slot-ms consumed by the query.
    #[builder(setter(into))]
    #[serde(with = "with::int64")]
    #[serde(default)]
    pub total_slot_ms: i64,
}

/// [Experimental] Defines the ranges for range partitioning.
#[derive(::serde::Deserialize, ::serde::Serialize, ::typed_builder::TypedBuilder, Debug, PartialEq, Clone)]
#[derive(Default)]
#[serde(rename_all = "camelCase")]
pub struct Range {
    /// [Experimental] The end of range partitioning, exclusive.
    #[builder(setter(into))]
    #[serde(with = "with::int64")]
    #[serde(default)]
    pub end: i64,
    /// [Experimental] The width of each interval.
    #[builder(setter(into))]
    #[serde(with = "with::int64")]
    #[serde(default)]
    pub interval: i64,
    /// [Experimental] The start of range partitioning, inclusive.
    #[builder(setter(into))]
    #[serde(with = "with::int64")]
    #[serde(default)]
    pub start: i64,
}

#[derive(::serde::Deserialize, ::serde::Serialize, ::typed_builder::TypedBuilder, Debug, PartialEq, Clone)]
#[derive(Default)]
#[serde(rename_all = "camelCase")]
pub struct RangePartitioning {
    /// Required. [Experimental] The table is partitioned by this field. The field must be a top-level
    /// NULLABLE/REQUIRED field. The only supported type is INTEGER/INT64.
    #[builder(setter(into))]
    #[serde(default)]
    pub field: ::std::string::String,
    /// [Experimental] Defines the ranges for range partitioning.
    #[builder(setter(into))]
    #[serde(default)]
    pub range: Range,
}

/// Represents the value of a range.
#[derive(::serde::Deserialize, ::serde::Serialize, ::typed_builder::TypedBuilder, Debug, PartialEq, Clone)]
#[derive(Default)]
#[serde(rename_all = "camelCase")]
pub struct RangeValue {
    /// Optional. The end value of the range. A missing value represents an unbounded end.
    #[builder(setter(into))]
    #[serde(default)]
    pub end: QueryParameterValue,
    /// Optional. The start value of the range. A missing value represents an unbounded start.
    #[builder(setter(into))]
    #[serde(default)]
    pub start: QueryParameterValue,
}

/// Evaluation metrics used by weighted-ALS models specified by feedback_type=implicit.
#[derive(::serde::Deserialize, ::serde::Serialize, ::typed_builder::TypedBuilder, Debug, PartialEq, Clone)]
#[derive(Default)]
#[serde(rename_all = "camelCase")]
pub struct RankingMetrics {
    /// Determines the goodness of a ranking by computing the percentile rank from the predicted
    /// confidence and dividing it by the original rank.
    #[builder(setter(into))]
    #[serde(default)]
    pub average_rank: f64,
    /// Calculates a precision per user for all the items by ranking them and then averages all the
    /// precisions across all the users.
    #[builder(setter(into))]
    #[serde(default)]
    pub mean_average_precision: f64,
    /// Similar to the mean squared error computed in regression and explicit recommendation models
    /// except instead of computing the rating directly, the output from evaluate is computed against a
    /// preference which is 1 or 0 depending on if the rating exists or not.
    #[builder(setter(into))]
    #[serde(default)]
    pub mean_squared_error: f64,
    /// A metric to determine the goodness of a ranking calculated from the predicted confidence by
    /// comparing it to an ideal rank measured by the original ratings.
    #[builder(setter(into))]
    #[serde(default)]
    pub normalized_discounted_cumulative_gain: f64,
}

/// Evaluation metrics for regression and explicit feedback type matrix factorization models.
#[derive(::serde::Deserialize, ::serde::Serialize, ::typed_builder::TypedBuilder, Debug, PartialEq, Clone)]
#[derive(Default)]
#[serde(rename_all = "camelCase")]
pub struct RegressionMetrics {
    /// Mean absolute error.
    #[builder(setter(into))]
    #[serde(default)]
    pub mean_absolute_error: f64,
    /// Mean squared error.
    #[builder(setter(into))]
    #[serde(default)]
    pub mean_squared_error: f64,
    /// Mean squared log error.
    #[builder(setter(into))]
    #[serde(default)]
    pub mean_squared_log_error: f64,
    /// Median absolute error.
    #[builder(setter(into))]
    #[serde(default)]
    pub median_absolute_error: f64,
    /// R^2 score. This corresponds to r2_score in ML.EVALUATE.
    #[builder(setter(into))]
    #[serde(default)]
    pub r_squared: f64,
}

/// User-defined context as a set of key/value pairs, which will be sent as function invocation
/// context together with batched arguments in the requests to the remote service. The total number
/// of bytes of keys and values must be less than 8KB.
pub type UserDefinedContext = ::std::collections::HashMap<::std::string::String, ::std::string::String>;

/// Options for a remote user-defined function.
#[derive(::serde::Deserialize, ::serde::Serialize, ::typed_builder::TypedBuilder, Debug, PartialEq, Clone)]
#[derive(Default)]
#[serde(rename_all = "camelCase")]
pub struct RemoteFunctionOptions {
    /// Fully qualified name of the user-provided connection object which holds the authentication
    /// information to send requests to the remote service. Format:
    /// ```"projects/{projectId}/locations/{locationId}/connections/{connectionId}"```
    #[builder(setter(into))]
    #[serde(default)]
    pub connection: ::std::string::String,
    /// Endpoint of the user-provided remote service, e.g.
    /// ```https://us-east1-my_gcf_project.cloudfunctions.net/remote_add```
    #[builder(setter(into))]
    #[serde(default)]
    pub endpoint: ::std::string::String,
    /// Max number of rows in each batch sent to the remote service. If absent or if 0, BigQuery
    /// dynamically decides the number of rows in a batch.
    #[builder(setter(into))]
    #[serde(with = "with::int64")]
    #[serde(default)]
    pub max_batching_rows: i64,
    /// User-defined context as a set of key/value pairs, which will be sent as function invocation
    /// context together with batched arguments in the requests to the remote service. The total number
    /// of bytes of keys and values must be less than 8KB.
    #[builder(setter(into))]
    #[serde(default)]
    pub user_defined_context: UserDefinedContext,
}

/// Output only. The remote service type for remote model.
#[derive(::serde::Deserialize, ::serde::Serialize, Debug, PartialEq, Clone)]
pub enum RemoteServiceType {
        /// V3 Cloud AI Translation API. See more details at [Cloud Translation API]
        /// (https://cloud.google.com/translate/docs/reference/rest).
    #[serde(rename = "CLOUD_AI_TRANSLATE_V3")]
    CloudAiTranslateV3,
        /// V1 Cloud AI Vision API See more details at [Cloud Vision API]
        /// (https://cloud.google.com/vision/docs/reference/rest).
    #[serde(rename = "CLOUD_AI_VISION_V1")]
    CloudAiVisionV1,
        /// V1 Cloud AI Natural Language API. See more details at [REST Resource:
        /// documents](https://cloud.google.com/natural-language/docs/reference/rest/v1/documents).
    #[serde(rename = "CLOUD_AI_NATURAL_LANGUAGE_V1")]
    CloudAiNaturalLanguageV1,
        /// V2 Speech-to-Text API. See more details at [Google Cloud Speech-to-Text V2
        /// API](https://cloud.google.com/speech-to-text/v2/docs)
    #[serde(rename = "CLOUD_AI_SPEECH_TO_TEXT_V2")]
    CloudAiSpeechToTextV2,
}

/// Remote Model Info
#[derive(::serde::Deserialize, ::serde::Serialize, ::typed_builder::TypedBuilder, Debug, PartialEq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct RemoteModelInfo {
    /// Output only. Fully qualified name of the user-provided connection object of the remote model.
    /// Format: ```"projects/{project_id}/locations/{location_id}/connections/{connection_id}"```
    #[builder(setter(into))]
    #[serde(default)]
    pub connection: ::std::string::String,
    /// Output only. The endpoint for remote model.
    #[builder(setter(into))]
    #[serde(default)]
    pub endpoint: ::std::string::String,
    /// Output only. Max number of rows in each batch sent to the remote service. If unset, the number
    /// of rows in each batch is set dynamically.
    #[builder(setter(into))]
    #[serde(with = "with::int64")]
    #[serde(default)]
    pub max_batching_rows: i64,
    /// Output only. The model version for LLM.
    #[builder(setter(into))]
    #[serde(default)]
    pub remote_model_version: ::std::string::String,
    /// Output only. The remote service type for remote model.
    #[builder(setter(into))]
    pub remote_service_type: RemoteServiceType,
    /// Output only. The name of the speech recognizer to use for speech recognition. The expected
    /// format is `projects/{project}/locations/{location}/recognizers/{recognizer}`. Customers can
    /// specify this field at model creation. If not specified, a default recognizer `projects/{model
    /// project}/locations/global/recognizers/_` will be used. See more details at
    /// [recognizers](https://cloud.google.com/speech-to-text/v2/docs/reference/rest/v2/projects.locations.recognizers)
    #[builder(setter(into))]
    #[serde(default)]
    pub speech_recognizer: ::std::string::String,
}

/// Optional. If set to `DATA_MASKING`, the function is validated and made available as a masking
/// function. For more information, see [Create custom masking
/// routines](https://cloud.google.com/bigquery/docs/user-defined-functions#custom-mask).
#[derive(::serde::Deserialize, ::serde::Serialize, Debug, PartialEq, Clone)]
pub enum DataGovernanceType {
        /// The data governance type is data masking.
    #[serde(rename = "DATA_MASKING")]
    DataMasking,
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

/// Optional. Defaults to "SQL" if remote_function_options field is absent, not set otherwise.
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
        /// Java language.
    #[serde(rename = "JAVA")]
    Java,
        /// Scala language.
    #[serde(rename = "SCALA")]
    Scala,
}

/// Required. The type of routine.
#[derive(::serde::Deserialize, ::serde::Serialize, Debug, PartialEq, Clone)]
pub enum RoutineType {
        /// Non-built-in persistent scalar function.
    #[serde(rename = "SCALAR_FUNCTION")]
    ScalarFunction,
        /// Stored procedure.
    #[serde(rename = "PROCEDURE")]
    Procedure,
        /// Non-built-in persistent TVF.
    #[serde(rename = "TABLE_VALUED_FUNCTION")]
    TableValuedFunction,
        /// Non-built-in persistent aggregate function.
    #[serde(rename = "AGGREGATE_FUNCTION")]
    AggregateFunction,
}

/// Optional. The security mode of the routine, if defined. If not defined, the security mode is
/// automatically determined from the routine's configuration.
#[derive(::serde::Deserialize, ::serde::Serialize, Debug, PartialEq, Clone)]
pub enum SecurityMode {
        /// The routine is to be executed with the privileges of the user who defines it.
    #[serde(rename = "DEFINER")]
    Definer,
        /// The routine is to be executed with the privileges of the user who invokes it.
    #[serde(rename = "INVOKER")]
    Invoker,
}

/// A user-defined function or a stored procedure.
#[derive(::serde::Deserialize, ::serde::Serialize, ::typed_builder::TypedBuilder, Debug, PartialEq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Routine {
    /// Optional.
    #[builder(setter(into))]
    #[serde(default)]
    pub arguments: ::std::vec::Vec<Argument>,
    /// Output only. The time when this routine was created, in milliseconds since the epoch.
    #[builder(setter(into))]
    #[serde(with = "with::int64")]
    #[serde(default)]
    pub creation_time: i64,
    /// Optional. If set to `DATA_MASKING`, the function is validated and made available as a masking
    /// function. For more information, see [Create custom masking
    /// routines](https://cloud.google.com/bigquery/docs/user-defined-functions#custom-mask).
    #[builder(setter(into))]
    pub data_governance_type: DataGovernanceType,
    /// Required. The body of the routine. For functions, this is the expression in the AS clause. If
    /// language=SQL, it is the substring inside (but excluding) the parentheses. For example, for the
    /// function created with the following statement: `CREATE FUNCTION JoinLines(x string, y string) as
    /// (concat(x, "\n", y))` The definition_body is `concat(x, "\n", y)` (\n is not replaced with
    /// linebreak). If language=JAVASCRIPT, it is the evaluated string in the AS clause. For example,
    /// for the function created with the following statement: `CREATE FUNCTION f() RETURNS STRING
    /// LANGUAGE js AS 'return "\n";\n'` The definition_body is `return "\n";\n` Note that both \n are
    /// replaced with linebreaks.
    #[builder(setter(into))]
    #[serde(default)]
    pub definition_body: ::std::string::String,
    /// Optional. The description of the routine, if defined.
    #[builder(setter(into))]
    #[serde(default)]
    pub description: ::std::option::Option<::std::string::String>,
    /// Optional. The determinism level of the JavaScript UDF, if defined.
    #[builder(setter(into))]
    pub determinism_level: DeterminismLevel,
    /// Output only. A hash of this resource.
    #[builder(setter(into))]
    #[serde(default)]
    pub etag: ::std::string::String,
    /// Optional. If language = "JAVASCRIPT", this field stores the path of the imported JAVASCRIPT
    /// libraries.
    #[builder(setter(into))]
    #[serde(default)]
    pub imported_libraries: ::std::vec::Vec<::std::string::String>,
    /// Optional. Defaults to "SQL" if remote_function_options field is absent, not set otherwise.
    #[builder(setter(into))]
    pub language: Language,
    /// Output only. The time when this routine was last modified, in milliseconds since the epoch.
    #[builder(setter(into))]
    #[serde(with = "with::int64")]
    #[serde(default)]
    pub last_modified_time: i64,
    /// Optional. Remote function specific options.
    #[builder(setter(into))]
    #[serde(default)]
    pub remote_function_options: RemoteFunctionOptions,
    /// Optional. Can be set only if routine_type = "TABLE_VALUED_FUNCTION". If absent, the return table
    /// type is inferred from definition_body at query time in each query that references this routine.
    /// If present, then the columns in the evaluated table result will be cast to match the column
    /// types specified in return table type, at query time.
    #[builder(setter(into))]
    #[serde(default)]
    pub return_table_type: ::std::option::Option<StandardSqlTableType>,
    /// Optional if language = "SQL"; required otherwise. Cannot be set if routine_type =
    /// "TABLE_VALUED_FUNCTION". If absent, the return type is inferred from definition_body at query
    /// time in each query that references this routine. If present, then the evaluated result will be
    /// cast to the specified returned type at query time. For example, for the functions created with
    /// the following statements: * `CREATE FUNCTION Add(x FLOAT64, y FLOAT64) RETURNS FLOAT64 AS (x +
    /// y);` * `CREATE FUNCTION Increment(x FLOAT64) AS (Add(x, 1));` * `CREATE FUNCTION Decrement(x
    /// FLOAT64) RETURNS FLOAT64 AS (Add(x, -1));` The return_type is `{type_kind: "FLOAT64"}` for `Add`
    /// and `Decrement`, and is absent for `Increment` (inferred as FLOAT64 at query time). Suppose the
    /// function `Add` is replaced by `CREATE OR REPLACE FUNCTION Add(x INT64, y INT64) AS (x + y);`
    /// Then the inferred return type of `Increment` is automatically changed to INT64 at query time,
    /// while the return type of `Decrement` remains FLOAT64.
    #[builder(setter(into))]
    #[serde(default)]
    pub return_type: ::std::option::Option<StandardSqlDataType>,
    /// Required. Reference describing the ID of this routine.
    #[builder(setter(into))]
    #[serde(default)]
    pub routine_reference: RoutineReference,
    /// Required. The type of routine.
    #[builder(setter(into))]
    pub routine_type: RoutineType,
    /// Optional. The security mode of the routine, if defined. If not defined, the security mode is
    /// automatically determined from the routine's configuration.
    #[builder(setter(into))]
    pub security_mode: SecurityMode,
    /// Optional. Spark specific options.
    #[builder(setter(into))]
    #[serde(default)]
    pub spark_options: ::std::option::Option<SparkOptions>,
    /// Optional. Use this option to catch many common errors. Error checking is not exhaustive, and
    /// successfully creating a procedure doesn't guarantee that the procedure will successfully execute
    /// at runtime. If `strictMode` is set to `TRUE`, the procedure body is further checked for errors
    /// such as non-existent tables or columns. The `CREATE PROCEDURE` statement fails if the body fails
    /// any of these checks. If `strictMode` is set to `FALSE`, the procedure body is checked only for
    /// syntax. For procedures that invoke themselves recursively, specify `strictMode=FALSE` to avoid
    /// non-existent procedure errors during validation. Default value is `TRUE`.
    #[builder(setter(into))]
    #[serde(default)]
    pub strict_mode: ::std::option::Option<bool>,
}

/// Id path of a routine.
#[derive(::serde::Deserialize, ::serde::Serialize, ::typed_builder::TypedBuilder, Debug, PartialEq, Clone)]
#[derive(Default)]
#[serde(rename_all = "camelCase")]
pub struct RoutineReference {
    /// Required. The ID of the dataset containing this routine.
    #[builder(setter(into))]
    #[serde(default)]
    pub dataset_id: ::std::string::String,
    /// Required. The ID of the project containing this routine.
    #[builder(setter(into))]
    #[serde(default)]
    pub project_id: ::std::string::String,
    /// Required. The ID of the routine. The ID must contain only letters (a-z, A-Z), numbers (0-9), or
    /// underscores (_). The maximum length is 256 characters.
    #[builder(setter(into))]
    #[serde(default)]
    pub routine_id: ::std::string::String,
}

/// A single row in the confusion matrix.
#[derive(::serde::Deserialize, ::serde::Serialize, ::typed_builder::TypedBuilder, Debug, PartialEq, Clone)]
#[derive(Default)]
#[serde(rename_all = "camelCase")]
pub struct Row {
    /// The original label of this row.
    #[builder(setter(into))]
    #[serde(default)]
    pub actual_label: ::std::string::String,
    /// Info describing predicted label distribution.
    #[builder(setter(into))]
    #[serde(default)]
    pub entries: ::std::vec::Vec<Entry>,
}

/// Represents access on a subset of rows on the specified table, defined by its filter predicate.
/// Access to the subset of rows is controlled by its IAM policy.
#[derive(::serde::Deserialize, ::serde::Serialize, ::typed_builder::TypedBuilder, Debug, PartialEq, Clone)]
#[derive(Default)]
#[serde(rename_all = "camelCase")]
pub struct RowAccessPolicy {
    /// Output only. The time when this row access policy was created, in milliseconds since the epoch.
    #[builder(setter(into))]
    #[serde(default)]
    pub creation_time: ::std::string::String,
    /// Output only. A hash of this resource.
    #[builder(setter(into))]
    #[serde(default)]
    pub etag: ::std::string::String,
    /// Required. A SQL boolean expression that represents the rows defined by this row access policy,
    /// similar to the boolean expression in a WHERE clause of a SELECT query on a table. References to
    /// other tables, routines, and temporary functions are not supported. Examples: region="EU"
    /// date_field = CAST('2019-9-27' as DATE) nullable_field is not NULL numeric_field BETWEEN 1.0 AND
    /// 5.0
    #[builder(setter(into))]
    #[serde(default)]
    pub filter_predicate: ::std::string::String,
    /// Output only. The time when this row access policy was last modified, in milliseconds since the
    /// epoch.
    #[builder(setter(into))]
    #[serde(default)]
    pub last_modified_time: ::std::string::String,
    /// Required. Reference describing the ID of this row access policy.
    #[builder(setter(into))]
    #[serde(default)]
    pub row_access_policy_reference: RowAccessPolicyReference,
}

/// Id path of a row access policy.
#[derive(::serde::Deserialize, ::serde::Serialize, ::typed_builder::TypedBuilder, Debug, PartialEq, Clone)]
#[derive(Default)]
#[serde(rename_all = "camelCase")]
pub struct RowAccessPolicyReference {
    /// Required. The ID of the dataset containing this row access policy.
    #[builder(setter(into))]
    #[serde(default)]
    pub dataset_id: ::std::string::String,
    /// Required. The ID of the row access policy. The ID must contain only letters (a-z, A-Z), numbers
    /// (0-9), or underscores (_). The maximum length is 256 characters.
    #[builder(setter(into))]
    #[serde(default)]
    pub policy_id: ::std::string::String,
    /// Required. The ID of the project containing this row access policy.
    #[builder(setter(into))]
    #[serde(default)]
    pub project_id: ::std::string::String,
    /// Required. The ID of the table containing this row access policy.
    #[builder(setter(into))]
    #[serde(default)]
    pub table_id: ::std::string::String,
}

/// Statistics for row-level security.
#[derive(::serde::Deserialize, ::serde::Serialize, ::typed_builder::TypedBuilder, Debug, PartialEq, Clone)]
#[derive(Default)]
#[serde(rename_all = "camelCase")]
pub struct RowLevelSecurityStatistics {
    /// Whether any accessed data was protected by row access policies.
    #[builder(setter(into))]
    #[serde(default)]
    pub row_level_security_applied: bool,
}

/// Determines which statement in the script represents the "key result", used to populate the
/// schema and query results of the script job. Default is LAST.
#[derive(::serde::Deserialize, ::serde::Serialize, Debug, PartialEq, Clone)]
pub enum KeyResultStatement {
        /// The last result determines the key result.
    #[serde(rename = "LAST")]
    Last,
        /// The first SELECT statement determines the key result.
    #[serde(rename = "FIRST_SELECT")]
    FirstSelect,
}

/// Options related to script execution.
#[derive(::serde::Deserialize, ::serde::Serialize, ::typed_builder::TypedBuilder, Debug, PartialEq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ScriptOptions {
    /// Determines which statement in the script represents the "key result", used to populate the
    /// schema and query results of the script job. Default is LAST.
    #[builder(setter(into))]
    pub key_result_statement: KeyResultStatement,
    /// Limit on the number of bytes billed per statement. Exceeding this budget results in an error.
    #[builder(setter(into))]
    #[serde(with = "with::int64")]
    #[serde(default)]
    pub statement_byte_budget: i64,
    /// Timeout period for each statement in a script.
    #[builder(setter(into))]
    #[serde(with = "with::int64")]
    #[serde(default)]
    pub statement_timeout_ms: i64,
}

/// Represents the location of the statement/expression being evaluated. Line and column numbers are
/// defined as follows: - Line and column numbers start with one. That is, line 1 column 1 denotes
/// the start of the script. - When inside a stored procedure, all line/column numbers are relative
/// to the procedure body, not the script in which the procedure was defined. - Start/end positions
/// exclude leading/trailing comments and whitespace. The end position always ends with a ";", when
/// present. - Multi-byte Unicode characters are treated as just one column. - If the original
/// script (or procedure definition) contains TAB characters, a tab "snaps" the indentation forward
/// to the nearest multiple of 8 characters, plus 1. For example, a TAB on column 1, 2, 3, 4, 5, 6 ,
/// or 8 will advance the next character to column 9. A TAB on column 9, 10, 11, 12, 13, 14, 15, or
/// 16 will advance the next character to column 17.
#[derive(::serde::Deserialize, ::serde::Serialize, ::typed_builder::TypedBuilder, Debug, PartialEq, Clone)]
#[derive(Default)]
#[serde(rename_all = "camelCase")]
pub struct ScriptStackFrame {
    /// Output only. One-based end column.
    #[builder(setter(into))]
    #[serde(default)]
    pub end_column: i64,
    /// Output only. One-based end line.
    #[builder(setter(into))]
    #[serde(default)]
    pub end_line: i64,
    /// Output only. Name of the active procedure, empty if in a top-level script.
    #[builder(setter(into))]
    #[serde(default)]
    pub procedure_id: ::std::string::String,
    /// Output only. One-based start column.
    #[builder(setter(into))]
    #[serde(default)]
    pub start_column: i64,
    /// Output only. One-based start line.
    #[builder(setter(into))]
    #[serde(default)]
    pub start_line: i64,
    /// Output only. Text of the current statement/expression.
    #[builder(setter(into))]
    #[serde(default)]
    pub text: ::std::string::String,
}

/// Whether this child job was a statement or expression.
#[derive(::serde::Deserialize, ::serde::Serialize, Debug, PartialEq, Clone)]
pub enum EvaluationKind {
        /// The statement appears directly in the script.
    #[serde(rename = "STATEMENT")]
    Statement,
        /// The statement evaluates an expression that appears in the script.
    #[serde(rename = "EXPRESSION")]
    Expression,
}

/// Job statistics specific to the child job of a script.
#[derive(::serde::Deserialize, ::serde::Serialize, ::typed_builder::TypedBuilder, Debug, PartialEq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ScriptStatistics {
    /// Whether this child job was a statement or expression.
    #[builder(setter(into))]
    pub evaluation_kind: EvaluationKind,
    /// Stack trace showing the line/column/procedure name of each frame on the stack at the point where
    /// the current evaluation happened. The leaf frame is first, the primary script is last. Never
    /// empty.
    #[builder(setter(into))]
    #[serde(default)]
    pub stack_frames: ::std::vec::Vec<ScriptStackFrame>,
}

/// Specifies the index usage mode for the query.
#[derive(::serde::Deserialize, ::serde::Serialize, Debug, PartialEq, Clone)]
pub enum IndexUsageMode {
        /// No vector indexes were used in the vector search query. See [`indexUnusedReasons`]
        /// (/bigquery/docs/reference/rest/v2/Job#IndexUnusedReason) for detailed reasons.
    #[serde(rename = "UNUSED")]
    Unused,
        /// Part of the vector search query used vector indexes. See [`indexUnusedReasons`]
        /// (/bigquery/docs/reference/rest/v2/Job#IndexUnusedReason) for why other parts of the query
        /// did not use vector indexes.
    #[serde(rename = "PARTIALLY_USED")]
    PartiallyUsed,
        /// The entire vector search query used vector indexes.
    #[serde(rename = "FULLY_USED")]
    FullyUsed,
}

/// Statistics for a search query. Populated as part of JobStatistics2.
#[derive(::serde::Deserialize, ::serde::Serialize, ::typed_builder::TypedBuilder, Debug, PartialEq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SearchStatistics {
    /// When `indexUsageMode` is `UNUSED` or `PARTIALLY_USED`, this field explains why indexes were not
    /// used in all or part of the search query. If `indexUsageMode` is `FULLY_USED`, this field is not
    /// populated.
    #[builder(setter(into))]
    #[serde(default)]
    pub index_unused_reasons: ::std::vec::Vec<IndexUnusedReason>,
    /// Specifies the index usage mode for the query.
    #[builder(setter(into))]
    pub index_usage_mode: IndexUsageMode,
}

/// [Preview] Information related to sessions.
#[derive(::serde::Deserialize, ::serde::Serialize, ::typed_builder::TypedBuilder, Debug, PartialEq, Clone)]
#[derive(Default)]
#[serde(rename_all = "camelCase")]
pub struct SessionInfo {
    /// Output only. The id of the session.
    #[builder(setter(into))]
    #[serde(default)]
    pub session_id: ::std::string::String,
}

/// Request message for `SetIamPolicy` method.
#[derive(::serde::Deserialize, ::serde::Serialize, ::typed_builder::TypedBuilder, Debug, PartialEq, Clone)]
#[derive(Default)]
#[serde(rename_all = "camelCase")]
pub struct SetIamPolicyRequest {
    /// REQUIRED: The complete policy to be applied to the `resource`. The size of the policy is limited
    /// to a few 10s of KB. An empty policy is a valid policy but certain Google Cloud services (such as
    /// Projects) might reject them.
    #[builder(setter(into))]
    #[serde(default)]
    pub policy: Policy,
    /// OPTIONAL: A FieldMask specifying which fields of the policy to modify. Only the fields in the
    /// mask will be modified. If no mask is provided, the following default mask is used: `paths:
    /// "bindings, etag"`
    #[builder(setter(into))]
    #[serde(default)]
    pub update_mask: ::std::option::Option<::std::string::String>,
}

/// Information about base table and snapshot time of the snapshot.
#[derive(::serde::Deserialize, ::serde::Serialize, ::typed_builder::TypedBuilder, Debug, PartialEq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SnapshotDefinition {
    /// Required. Reference describing the ID of the table that was snapshot.
    #[builder(setter(into))]
    #[serde(default)]
    pub base_table_reference: TableReference,
    /// Required. The time at which the base table was snapshot. This value is reported in the JSON
    /// response using RFC3339 format.
    #[builder(setter(into))]
    pub snapshot_time: ::timestamp::Timestamp,
}

/// Spark job logs can be filtered by these fields in Cloud Logging.
#[derive(::serde::Deserialize, ::serde::Serialize, ::typed_builder::TypedBuilder, Debug, PartialEq, Clone)]
#[derive(Default)]
#[serde(rename_all = "camelCase")]
pub struct SparkLoggingInfo {
    /// Output only. Project ID where the Spark logs were written.
    #[builder(setter(into))]
    #[serde(default)]
    pub project_id: ::std::string::String,
    /// Output only. Resource type used for logging.
    #[builder(setter(into))]
    #[serde(default)]
    pub resource_type: ::std::string::String,
}

/// Configuration properties as a set of key/value pairs, which will be passed on to the Spark
/// application. For more information, see [Apache
/// Spark](https://spark.apache.org/docs/latest/index.html) and the [procedure option
/// list](https://cloud.google.com/bigquery/docs/reference/standard-sql/data-definition-language#procedure_option_list).
pub type Properties = ::std::collections::HashMap<::std::string::String, ::std::string::String>;

/// Options for a user-defined Spark routine.
#[derive(::serde::Deserialize, ::serde::Serialize, ::typed_builder::TypedBuilder, Debug, PartialEq, Clone)]
#[derive(Default)]
#[serde(rename_all = "camelCase")]
pub struct SparkOptions {
    /// Archive files to be extracted into the working directory of each executor. For more information
    /// about Apache Spark, see [Apache Spark](https://spark.apache.org/docs/latest/index.html).
    #[builder(setter(into))]
    #[serde(default)]
    pub archive_uris: ::std::vec::Vec<::std::string::String>,
    /// Fully qualified name of the user-provided Spark connection object. Format:
    /// ```"projects/{project_id}/locations/{location_id}/connections/{connection_id}"```
    #[builder(setter(into))]
    #[serde(default)]
    pub connection: ::std::string::String,
    /// Custom container image for the runtime environment.
    #[builder(setter(into))]
    #[serde(default)]
    pub container_image: ::std::string::String,
    /// Files to be placed in the working directory of each executor. For more information about Apache
    /// Spark, see [Apache Spark](https://spark.apache.org/docs/latest/index.html).
    #[builder(setter(into))]
    #[serde(default)]
    pub file_uris: ::std::vec::Vec<::std::string::String>,
    /// JARs to include on the driver and executor CLASSPATH. For more information about Apache Spark,
    /// see [Apache Spark](https://spark.apache.org/docs/latest/index.html).
    #[builder(setter(into))]
    #[serde(default)]
    pub jar_uris: ::std::vec::Vec<::std::string::String>,
    /// The fully qualified name of a class in jar_uris, for example, com.example.wordcount. Exactly one
    /// of main_class and main_jar_uri field should be set for Java/Scala language type.
    #[builder(setter(into))]
    #[serde(default)]
    pub main_class: ::std::string::String,
    /// The main file/jar URI of the Spark application. Exactly one of the definition_body field and the
    /// main_file_uri field must be set for Python. Exactly one of main_class and main_file_uri field
    /// should be set for Java/Scala language type.
    #[builder(setter(into))]
    #[serde(default)]
    pub main_file_uri: ::std::string::String,
    /// Configuration properties as a set of key/value pairs, which will be passed on to the Spark
    /// application. For more information, see [Apache
    /// Spark](https://spark.apache.org/docs/latest/index.html) and the [procedure option
    /// list](https://cloud.google.com/bigquery/docs/reference/standard-sql/data-definition-language#procedure_option_list).
    #[builder(setter(into))]
    #[serde(default)]
    pub properties: Properties,
    /// Python files to be placed on the PYTHONPATH for PySpark application. Supported file types:
    /// `.py`, `.egg`, and `.zip`. For more information about Apache Spark, see [Apache
    /// Spark](https://spark.apache.org/docs/latest/index.html).
    #[builder(setter(into))]
    #[serde(default)]
    pub py_file_uris: ::std::vec::Vec<::std::string::String>,
    /// Runtime version. If not specified, the default runtime version is used.
    #[builder(setter(into))]
    #[serde(default)]
    pub runtime_version: ::std::string::String,
}

/// Output only. Endpoints returned from Dataproc. Key list: - history_server_endpoint: A link to
/// Spark job UI.
pub type Endpoints = ::std::collections::HashMap<::std::string::String, ::std::string::String>;

/// Statistics for a BigSpark query. Populated as part of JobStatistics2
#[derive(::serde::Deserialize, ::serde::Serialize, ::typed_builder::TypedBuilder, Debug, PartialEq, Clone)]
#[derive(Default)]
#[serde(rename_all = "camelCase")]
pub struct SparkStatistics {
    /// Output only. Endpoints returned from Dataproc. Key list: - history_server_endpoint: A link to
    /// Spark job UI.
    #[builder(setter(into))]
    #[serde(default)]
    pub endpoints: Endpoints,
    /// Output only. The Google Cloud Storage bucket that is used as the default filesystem by the Spark
    /// application. This fields is only filled when the Spark procedure uses the INVOKER security mode.
    /// It is inferred from the system variable @@spark_proc_properties.staging_bucket if it is
    /// provided. Otherwise, BigQuery creates a default staging bucket for the job and returns the
    /// bucket name in this field. Example: * `gs://[bucket_name]`
    #[builder(setter(into))]
    #[serde(default)]
    pub gcs_staging_bucket: ::std::string::String,
    /// Output only. The Cloud KMS encryption key that is used to protect the resources created by the
    /// Spark job. If the Spark procedure uses DEFINER security mode, the Cloud KMS key is inferred from
    /// the Spark connection associated with the procedure if it is provided. Otherwise the key is
    /// inferred from the default key of the Spark connection's project if the CMEK organization policy
    /// is enforced. If the Spark procedure uses INVOKER security mode, the Cloud KMS encryption key is
    /// inferred from the system variable @@spark_proc_properties.kms_key_name if it is provided.
    /// Otherwise, the key is inferred fromt he default key of the BigQuery job's project if the CMEK
    /// organization policy is enforced. Example: *
    /// `projects/[kms_project_id]/locations/[region]/keyRings/[key_region]/cryptoKeys/[key]`
    #[builder(setter(into))]
    #[serde(default)]
    pub kms_key_name: ::std::string::String,
    /// Output only. Logging info is used to generate a link to Cloud Logging.
    #[builder(setter(into))]
    #[serde(default)]
    pub logging_info: SparkLoggingInfo,
    /// Output only. Spark job ID if a Spark job is created successfully.
    #[builder(setter(into))]
    #[serde(default)]
    pub spark_job_id: ::std::string::String,
    /// Output only. Location where the Spark job is executed. A location is selected by BigQueury for
    /// jobs configured to run in a multi-region.
    #[builder(setter(into))]
    #[serde(default)]
    pub spark_job_location: ::std::string::String,
}

/// Performance insights compared to the previous executions for a specific stage.
#[derive(::serde::Deserialize, ::serde::Serialize, ::typed_builder::TypedBuilder, Debug, PartialEq, Clone)]
#[derive(Default)]
#[serde(rename_all = "camelCase")]
pub struct StagePerformanceChangeInsight {
    /// Output only. Input data change insight of the query stage.
    #[builder(setter(into))]
    #[serde(default)]
    pub input_data_change: InputDataChange,
    /// Output only. The stage id that the insight mapped to.
    #[builder(setter(into))]
    #[serde(with = "with::int64")]
    #[serde(default)]
    pub stage_id: i64,
}

/// Standalone performance insights for a specific stage.
#[derive(::serde::Deserialize, ::serde::Serialize, ::typed_builder::TypedBuilder, Debug, PartialEq, Clone)]
#[derive(Default)]
#[serde(rename_all = "camelCase")]
pub struct StagePerformanceStandaloneInsight {
    /// Output only. If present, the stage had the following reasons for being disqualified from BI
    /// Engine execution.
    #[builder(setter(into))]
    #[serde(default)]
    pub bi_engine_reasons: ::std::vec::Vec<BiEngineReason>,
    /// Output only. High cardinality joins in the stage.
    #[builder(setter(into))]
    #[serde(default)]
    pub high_cardinality_joins: ::std::vec::Vec<HighCardinalityJoin>,
    /// Output only. True if the stage has insufficient shuffle quota.
    #[builder(setter(into))]
    #[serde(default)]
    pub insufficient_shuffle_quota: bool,
    /// Output only. True if the stage has a slot contention issue.
    #[builder(setter(into))]
    #[serde(default)]
    pub slot_contention: bool,
    /// Output only. The stage id that the insight mapped to.
    #[builder(setter(into))]
    #[serde(with = "with::int64")]
    #[serde(default)]
    pub stage_id: i64,
}

/// Required. The top level type of this field. Can be any GoogleSQL data type (e.g., "INT64",
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
        /// Encoded as a pair with types matching range_element_type. Pairs must begin with "[", end
        /// with ")", and be separated by ", ".
    #[serde(rename = "RANGE")]
    Range,
}

/// The data type of a variable such as a function argument. Examples include: * INT64:
/// `{"typeKind": "INT64"}` * ARRAY: { "typeKind": "ARRAY", "arrayElementType": {"typeKind":
/// "STRING"} } * STRUCT>: { "typeKind": "STRUCT", "structType": { "fields": [ { "name": "x",
/// "type": {"typeKind": "STRING"} }, { "name": "y", "type": { "typeKind": "ARRAY",
/// "arrayElementType": {"typeKind": "DATE"} } } ] } }
#[derive(::serde::Deserialize, ::serde::Serialize, ::typed_builder::TypedBuilder, Debug, PartialEq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct StandardSqlDataType {
    /// The type of the array's elements, if type_kind = "ARRAY".
    #[builder(setter(into))]
    pub array_element_type: ::std::boxed::Box<StandardSqlDataType>,
    /// The type of the range's elements, if type_kind = "RANGE".
    #[builder(setter(into))]
    pub range_element_type: ::std::boxed::Box<StandardSqlDataType>,
    /// The fields of this struct, in order, if type_kind = "STRUCT".
    #[builder(setter(into))]
    #[serde(default)]
    pub struct_type: StandardSqlStructType,
    /// Required. The top level type of this field. Can be any GoogleSQL data type (e.g., "INT64",
    /// "DATE", "ARRAY").
    #[builder(setter(into))]
    pub type_kind: TypeKind,
}

/// A field or a column.
#[derive(::serde::Deserialize, ::serde::Serialize, ::typed_builder::TypedBuilder, Debug, PartialEq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct StandardSqlField {
    /// Optional. The name of this field. Can be absent for struct fields.
    #[builder(setter(into))]
    #[serde(default)]
    pub name: ::std::option::Option<::std::string::String>,
    /// Optional. The type of this parameter. Absent if not explicitly specified (e.g., CREATE FUNCTION
    /// statement can omit the return type; in this case the output parameter does not have this "type"
    /// field).
    #[builder(setter(into))]
    #[serde(rename = "type")]
    pub ty: StandardSqlDataType,
}

/// The representation of a SQL STRUCT type.
#[derive(::serde::Deserialize, ::serde::Serialize, ::typed_builder::TypedBuilder, Debug, PartialEq, Clone)]
#[derive(Default)]
#[serde(rename_all = "camelCase")]
pub struct StandardSqlStructType {
    /// Fields within the struct.
    #[builder(setter(into))]
    #[serde(default)]
    pub fields: ::std::vec::Vec<StandardSqlField>,
}

/// A table type
#[derive(::serde::Deserialize, ::serde::Serialize, ::typed_builder::TypedBuilder, Debug, PartialEq, Clone)]
#[derive(Default)]
#[serde(rename_all = "camelCase")]
pub struct StandardSqlTableType {
    /// The columns in this table type
    #[builder(setter(into))]
    #[serde(default)]
    pub columns: ::std::vec::Vec<StandardSqlField>,
}

#[derive(::serde::Deserialize, ::serde::Serialize, ::typed_builder::TypedBuilder, Debug, PartialEq, Clone)]
#[derive(Default)]
#[serde(rename_all = "camelCase")]
pub struct Streamingbuffer {
    /// Output only. A lower-bound estimate of the number of bytes currently in the streaming buffer.
    #[builder(setter(into))]
    #[serde(with = "with::uint64")]
    #[serde(default)]
    pub estimated_bytes: u64,
    /// Output only. A lower-bound estimate of the number of rows currently in the streaming buffer.
    #[builder(setter(into))]
    #[serde(with = "with::uint64")]
    #[serde(default)]
    pub estimated_rows: u64,
    /// Output only. Contains the timestamp of the oldest entry in the streaming buffer, in milliseconds
    /// since the epoch, if the streaming buffer is available.
    #[builder(setter(into))]
    #[serde(with = "with::uint64")]
    #[serde(default)]
    pub oldest_entry_time: u64,
}

/// Search space for string and enum.
#[derive(::serde::Deserialize, ::serde::Serialize, ::typed_builder::TypedBuilder, Debug, PartialEq, Clone)]
#[derive(Default)]
#[serde(rename_all = "camelCase")]
pub struct StringHparamSearchSpace {
    /// Canididates for the string or enum parameter in lower case.
    #[builder(setter(into))]
    #[serde(default)]
    pub candidates: ::std::vec::Vec<::std::string::String>,
}

/// Output only. Data type for each system variable.
pub type Types = ::std::collections::HashMap<::std::string::String, StandardSqlDataType>;

/// Output only. Value for each system variable.
pub type Values = ::std::collections::HashMap<::std::string::String, ::serde_json::Value>;

/// System variables given to a query.
#[derive(::serde::Deserialize, ::serde::Serialize, ::typed_builder::TypedBuilder, Debug, PartialEq, Clone)]
#[derive(Default)]
#[serde(rename_all = "camelCase")]
pub struct SystemVariables {
    /// Output only. Data type for each system variable.
    #[builder(setter(into))]
    #[serde(default)]
    pub types: Types,
    /// Output only. Value for each system variable.
    #[builder(setter(into))]
    #[serde(default)]
    pub values: Values,
}

/// [Optional] The tags associated with this table. Tag keys are globally unique. See additional
/// information on [tags](https://cloud.google.com/iam/docs/tags-access-control#definitions). An
/// object containing a list of "key": value pairs. The key is the namespaced friendly name of the
/// tag key, e.g. "12345/environment" where 12345 is parent id. The value is the friendly short name
/// of the tag value, e.g. "production".
pub type ResourceTags = ::std::collections::HashMap<::std::string::String, ::std::string::String>;

#[derive(::serde::Deserialize, ::serde::Serialize, ::typed_builder::TypedBuilder, Debug, PartialEq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Table {
    /// Optional. Specifies the configuration of a BigLake managed table.
    #[builder(setter(into))]
    pub biglake_configuration: BigLakeConfiguration,
    /// Output only. Contains information about the clone. This value is set via the clone operation.
    #[builder(setter(into))]
    pub clone_definition: CloneDefinition,
    /// Clustering specification for the table. Must be specified with time-based partitioning, data in
    /// the table will be first partitioned and subsequently clustered.
    #[builder(setter(into))]
    #[serde(default)]
    pub clustering: Clustering,
    /// Output only. The time when this table was created, in milliseconds since the epoch.
    #[builder(setter(into))]
    #[serde(with = "with::int64")]
    #[serde(default)]
    pub creation_time: i64,
    /// Optional. Defines the default collation specification of new STRING fields in the table. During
    /// table creation or update, if a STRING field is added to this table without explicit collation
    /// specified, then the table inherits the table default collation. A change to this field affects
    /// only fields added afterwards, and does not alter the existing fields. The following values are
    /// supported: * 'und:ci': undetermined locale, case insensitive. * '': empty string. Default to
    /// case-sensitive behavior.
    #[builder(setter(into))]
    #[serde(default)]
    pub default_collation: ::std::option::Option<::std::string::String>,
    /// Optional. Defines the default rounding mode specification of new decimal fields (NUMERIC OR
    /// BIGNUMERIC) in the table. During table creation or update, if a decimal field is added to this
    /// table without an explicit rounding mode specified, then the field inherits the table default
    /// rounding mode. Changing this field doesn't affect existing fields.
    #[builder(setter(into))]
    pub default_rounding_mode: DefaultRoundingMode,
    /// Optional. A user-friendly description of this table.
    #[builder(setter(into))]
    #[serde(default)]
    pub description: ::std::option::Option<::std::string::String>,
    /// Custom encryption configuration (e.g., Cloud KMS keys).
    #[builder(setter(into))]
    #[serde(default)]
    pub encryption_configuration: EncryptionConfiguration,
    /// Output only. A hash of this resource.
    #[builder(setter(into))]
    #[serde(default)]
    pub etag: ::std::string::String,
    /// Optional. The time when this table expires, in milliseconds since the epoch. If not present, the
    /// table will persist indefinitely. Expired tables will be deleted and their storage reclaimed. The
    /// defaultTableExpirationMs property of the encapsulating dataset can be used to set a default
    /// expirationTime on newly created tables.
    #[builder(setter(into))]
    #[serde(with = "with::int64::option")]
    #[serde(default)]
    pub expiration_time: ::std::option::Option<i64>,
    /// Optional. Describes the data format, location, and other properties of a table stored outside of
    /// BigQuery. By defining these properties, the data source can then be queried as if it were a
    /// standard BigQuery table.
    #[builder(setter(into))]
    pub external_data_configuration: ExternalDataConfiguration,
    /// Optional. A descriptive name for this table.
    #[builder(setter(into))]
    #[serde(default)]
    pub friendly_name: ::std::option::Option<::std::string::String>,
    /// Output only. An opaque ID uniquely identifying the table.
    #[builder(setter(into))]
    #[serde(default)]
    pub id: ::std::string::String,
    /// The type of resource ID.
    #[builder(setter(into))]
    #[serde(default)]
    pub kind: ::std::string::String,
    /// The labels associated with this table. You can use these to organize and group your tables.
    /// Label keys and values can be no longer than 63 characters, can only contain lowercase letters,
    /// numeric characters, underscores and dashes. International characters are allowed. Label values
    /// are optional. Label keys must start with a letter and each label in the list must have a
    /// different key.
    #[builder(setter(into))]
    #[serde(default)]
    pub labels: Labels,
    /// Output only. The time when this table was last modified, in milliseconds since the epoch.
    #[builder(setter(into))]
    #[serde(with = "with::uint64")]
    #[serde(default)]
    pub last_modified_time: u64,
    /// Output only. The geographic location where the table resides. This value is inherited from the
    /// dataset.
    #[builder(setter(into))]
    #[serde(default)]
    pub location: ::std::string::String,
    /// Optional. The materialized view definition.
    #[builder(setter(into))]
    #[serde(default)]
    pub materialized_view: MaterializedViewDefinition,
    /// Output only. The materialized view status.
    #[builder(setter(into))]
    #[serde(default)]
    pub materialized_view_status: MaterializedViewStatus,
    /// Optional. The maximum staleness of data that could be returned when the table (or stale MV) is
    /// queried. Staleness encoded as a string encoding of sql IntervalValue type.
    #[builder(setter(into))]
    #[serde(default)]
    pub max_staleness: ::std::option::Option<::std::string::String>,
    /// Deprecated.
    #[builder(setter(into))]
    #[serde(default)]
    pub model: ModelDefinition,
    /// Output only. Number of logical bytes that are less than 90 days old.
    #[builder(setter(into))]
    #[serde(with = "with::int64")]
    #[serde(default)]
    pub num_active_logical_bytes: i64,
    /// Output only. Number of physical bytes less than 90 days old. This data is not kept in real time,
    /// and might be delayed by a few seconds to a few minutes.
    #[builder(setter(into))]
    #[serde(with = "with::int64")]
    #[serde(default)]
    pub num_active_physical_bytes: i64,
    /// Output only. The size of this table in logical bytes, excluding any data in the streaming
    /// buffer.
    #[builder(setter(into))]
    #[serde(with = "with::int64")]
    #[serde(default)]
    pub num_bytes: i64,
    /// Output only. The number of logical bytes in the table that are considered "long-term storage".
    #[builder(setter(into))]
    #[serde(with = "with::int64")]
    #[serde(default)]
    pub num_long_term_bytes: i64,
    /// Output only. Number of logical bytes that are more than 90 days old.
    #[builder(setter(into))]
    #[serde(with = "with::int64")]
    #[serde(default)]
    pub num_long_term_logical_bytes: i64,
    /// Output only. Number of physical bytes more than 90 days old. This data is not kept in real time,
    /// and might be delayed by a few seconds to a few minutes.
    #[builder(setter(into))]
    #[serde(with = "with::int64")]
    #[serde(default)]
    pub num_long_term_physical_bytes: i64,
    /// Output only. The number of partitions present in the table or materialized view. This data is
    /// not kept in real time, and might be delayed by a few seconds to a few minutes.
    #[builder(setter(into))]
    #[serde(with = "with::int64")]
    #[serde(default)]
    pub num_partitions: i64,
    /// Output only. The physical size of this table in bytes. This includes storage used for time
    /// travel.
    #[builder(setter(into))]
    #[serde(with = "with::int64")]
    #[serde(default)]
    pub num_physical_bytes: i64,
    /// Output only. The number of rows of data in this table, excluding any data in the streaming
    /// buffer.
    #[builder(setter(into))]
    #[serde(with = "with::uint64")]
    #[serde(default)]
    pub num_rows: u64,
    /// Output only. Number of physical bytes used by time travel storage (deleted or changed data).
    /// This data is not kept in real time, and might be delayed by a few seconds to a few minutes.
    #[builder(setter(into))]
    #[serde(with = "with::int64")]
    #[serde(default)]
    pub num_time_travel_physical_bytes: i64,
    /// Output only. Total number of logical bytes in the table or materialized view.
    #[builder(setter(into))]
    #[serde(with = "with::int64")]
    #[serde(default)]
    pub num_total_logical_bytes: i64,
    /// Output only. The physical size of this table in bytes. This also includes storage used for time
    /// travel. This data is not kept in real time, and might be delayed by a few seconds to a few
    /// minutes.
    #[builder(setter(into))]
    #[serde(with = "with::int64")]
    #[serde(default)]
    pub num_total_physical_bytes: i64,
    /// If specified, configures range partitioning for this table.
    #[builder(setter(into))]
    #[serde(default)]
    pub range_partitioning: RangePartitioning,
    /// Optional. Output only. Table references of all replicas currently active on the table.
    #[builder(setter(into))]
    #[serde(default)]
    pub replicas: ::std::vec::Vec<TableReference>,
    /// Optional. If set to true, queries over this table require a partition filter that can be used
    /// for partition elimination to be specified.
    #[builder(setter(into))]
    #[serde(default)]
    pub require_partition_filter: ::std::option::Option<bool>,
    /// [Optional] The tags associated with this table. Tag keys are globally unique. See additional
    /// information on [tags](https://cloud.google.com/iam/docs/tags-access-control#definitions). An
    /// object containing a list of "key": value pairs. The key is the namespaced friendly name of the
    /// tag key, e.g. "12345/environment" where 12345 is parent id. The value is the friendly short name
    /// of the tag value, e.g. "production".
    #[builder(setter(into))]
    #[serde(default)]
    pub resource_tags: ResourceTags,
    /// Optional. Describes the schema of this table.
    #[builder(setter(into))]
    #[serde(default)]
    pub schema: ::std::option::Option<TableSchema>,
    /// Output only. A URL that can be used to access this resource again.
    #[builder(setter(into))]
    #[serde(default)]
    pub self_link: ::std::string::String,
    /// Output only. Contains information about the snapshot. This value is set via snapshot creation.
    #[builder(setter(into))]
    pub snapshot_definition: SnapshotDefinition,
    /// Output only. Contains information regarding this table's streaming buffer, if one is present.
    /// This field will be absent if the table is not being streamed to or if there is no data in the
    /// streaming buffer.
    #[builder(setter(into))]
    #[serde(default)]
    pub streaming_buffer: Streamingbuffer,
    /// Optional. Tables Primary Key and Foreign Key information
    #[builder(setter(into))]
    #[serde(default)]
    pub table_constraints: ::std::option::Option<TableConstraints>,
    /// Required. Reference describing the ID of this table.
    #[builder(setter(into))]
    #[serde(default)]
    pub table_reference: TableReference,
    /// Optional. Table replication info for table created `AS REPLICA` DDL like: `CREATE MATERIALIZED
    /// VIEW mv1 AS REPLICA OF src_mv`
    #[builder(setter(into))]
    #[serde(default)]
    pub table_replication_info: ::std::option::Option<TableReplicationInfo>,
    /// If specified, configures time-based partitioning for this table.
    #[builder(setter(into))]
    #[serde(default)]
    pub time_partitioning: TimePartitioning,
    /// Output only. Describes the table type. The following values are supported: * `TABLE`: A normal
    /// BigQuery table. * `VIEW`: A virtual table defined by a SQL query. * `EXTERNAL`: A table that
    /// references data stored in an external storage system, such as Google Cloud Storage. *
    /// `MATERIALIZED_VIEW`: A precomputed view defined by a SQL query. * `SNAPSHOT`: An immutable
    /// BigQuery table that preserves the contents of a base table at a particular time. See additional
    /// information on [table snapshots](/bigquery/docs/table-snapshots-intro). The default value is
    /// `TABLE`.
    #[builder(setter(into))]
    #[serde(rename = "type")]
    #[serde(default)]
    pub ty: ::std::string::String,
    /// Optional. The view definition.
    #[builder(setter(into))]
    #[serde(default)]
    pub view: ::std::option::Option<ViewDefinition>,
}

#[derive(::serde::Deserialize, ::serde::Serialize, ::typed_builder::TypedBuilder, Debug, PartialEq, Clone)]
#[derive(Default)]
#[serde(rename_all = "camelCase")]
pub struct TableCell {
    #[builder(setter(into))]
    #[serde(default)]
    pub v: ::serde_json::Value,
}

/// The pair of the foreign key column and primary key column.
#[derive(::serde::Deserialize, ::serde::Serialize, ::typed_builder::TypedBuilder, Debug, PartialEq, Clone)]
#[derive(Default)]
#[serde(rename_all = "camelCase")]
pub struct ColumnReferences {
    /// Required. The column in the primary key that are referenced by the referencing_column.
    #[builder(setter(into))]
    #[serde(default)]
    pub referenced_column: ::std::string::String,
    /// Required. The column that composes the foreign key.
    #[builder(setter(into))]
    #[serde(default)]
    pub referencing_column: ::std::string::String,
}

#[derive(::serde::Deserialize, ::serde::Serialize, ::typed_builder::TypedBuilder, Debug, PartialEq, Clone)]
#[derive(Default)]
#[serde(rename_all = "camelCase")]
pub struct ReferencedTable {
    #[builder(setter(into))]
    #[serde(default)]
    pub dataset_id: ::std::string::String,
    #[builder(setter(into))]
    #[serde(default)]
    pub project_id: ::std::string::String,
    #[builder(setter(into))]
    #[serde(default)]
    pub table_id: ::std::string::String,
}

/// Represents a foreign key constraint on a table's columns.
#[derive(::serde::Deserialize, ::serde::Serialize, ::typed_builder::TypedBuilder, Debug, PartialEq, Clone)]
#[derive(Default)]
#[serde(rename_all = "camelCase")]
pub struct ForeignKeys {
    /// Required. The columns that compose the foreign key.
    #[builder(setter(into))]
    #[serde(default)]
    pub column_references: ::std::vec::Vec<ColumnReferences>,
    /// Optional. Set only if the foreign key constraint is named.
    #[builder(setter(into))]
    #[serde(default)]
    pub name: ::std::option::Option<::std::string::String>,
    #[builder(setter(into))]
    #[serde(default)]
    pub referenced_table: ReferencedTable,
}

/// Represents the primary key constraint on a table's columns.
#[derive(::serde::Deserialize, ::serde::Serialize, ::typed_builder::TypedBuilder, Debug, PartialEq, Clone)]
#[derive(Default)]
#[serde(rename_all = "camelCase")]
pub struct PrimaryKey {
    /// Required. The columns that are composed of the primary key constraint.
    #[builder(setter(into))]
    #[serde(default)]
    pub columns: ::std::vec::Vec<::std::string::String>,
}

/// The TableConstraints defines the primary key and foreign key.
#[derive(::serde::Deserialize, ::serde::Serialize, ::typed_builder::TypedBuilder, Debug, PartialEq, Clone)]
#[derive(Default)]
#[serde(rename_all = "camelCase")]
pub struct TableConstraints {
    /// Optional. Present only if the table has a foreign key. The foreign key is not enforced.
    #[builder(setter(into))]
    #[serde(default)]
    pub foreign_keys: ::std::vec::Vec<ForeignKeys>,
    /// Represents the primary key constraint on a table's columns.
    #[builder(setter(into))]
    #[serde(default)]
    pub primary_key: PrimaryKey,
}

/// Data for a single insertion row.
#[derive(::serde::Deserialize, ::serde::Serialize, ::typed_builder::TypedBuilder, Debug, PartialEq, Clone)]
#[derive(Default)]
#[serde(rename_all = "camelCase")]
pub struct Rows {
    /// Insertion ID for best-effort deduplication. This feature is not recommended, and users seeking
    /// stronger insertion semantics are encouraged to use other mechanisms such as the BigQuery Write
    /// API.
    #[builder(setter(into))]
    #[serde(default)]
    pub insert_id: ::std::string::String,
    /// Data for a single row.
    #[builder(setter(into))]
    #[serde(default)]
    pub json: JsonObject,
}

/// Request for sending a single streaming insert.
#[derive(::serde::Deserialize, ::serde::Serialize, ::typed_builder::TypedBuilder, Debug, PartialEq, Clone)]
#[derive(Default)]
#[serde(rename_all = "camelCase")]
pub struct TableDataInsertAllRequest {
    /// Optional. Accept rows that contain values that do not match the schema. The unknown values are
    /// ignored. Default is false, which treats unknown values as errors.
    #[builder(setter(into))]
    #[serde(default)]
    pub ignore_unknown_values: ::std::option::Option<bool>,
    /// Optional. The resource type of the response. The value is not checked at the backend.
    /// Historically, it has been set to "bigquery#tableDataInsertAllRequest" but you are not required
    /// to set it.
    #[builder(setter(into))]
    #[serde(default)]
    pub kind: ::std::option::Option<::std::string::String>,
    #[builder(setter(into))]
    #[serde(default)]
    pub rows: ::std::vec::Vec<Rows>,
    /// Optional. Insert all valid rows of a request, even if invalid rows exist. The default value is
    /// false, which causes the entire request to fail if any invalid rows exist.
    #[builder(setter(into))]
    #[serde(default)]
    pub skip_invalid_rows: ::std::option::Option<bool>,
    /// Optional. If specified, treats the destination table as a base template, and inserts the rows
    /// into an instance table named "{destination}{templateSuffix}". BigQuery will manage creation of
    /// the instance table, using the schema of the base template table. See
    /// https://cloud.google.com/bigquery/streaming-data-into-bigquery#template-tables for
    /// considerations when working with templates tables.
    #[builder(setter(into))]
    #[serde(default)]
    pub template_suffix: ::std::option::Option<::std::string::String>,
    /// Optional. Unique request trace id. Used for debugging purposes only. It is case-sensitive,
    /// limited to up to 36 ASCII characters. A UUID is recommended.
    #[builder(setter(into))]
    #[serde(default)]
    pub trace_id: ::std::option::Option<::std::string::String>,
}

/// Error details about a single row's insertion.
#[derive(::serde::Deserialize, ::serde::Serialize, ::typed_builder::TypedBuilder, Debug, PartialEq, Clone)]
#[derive(Default)]
#[serde(rename_all = "camelCase")]
pub struct InsertErrors {
    /// Error information for the row indicated by the index property.
    #[builder(setter(into))]
    #[serde(default)]
    pub errors: ::std::vec::Vec<ErrorProto>,
    /// The index of the row that error applies to.
    #[builder(setter(into))]
    #[serde(default)]
    pub index: i64,
}

/// Describes the format of a streaming insert response.
#[derive(::serde::Deserialize, ::serde::Serialize, ::typed_builder::TypedBuilder, Debug, PartialEq, Clone)]
#[derive(Default)]
#[serde(rename_all = "camelCase")]
pub struct TableDataInsertAllResponse {
    /// Describes specific errors encountered while processing the request.
    #[builder(setter(into))]
    #[serde(default)]
    pub insert_errors: ::std::vec::Vec<InsertErrors>,
    /// Returns "bigquery#tableDataInsertAllResponse".
    #[builder(setter(into))]
    #[serde(default)]
    pub kind: ::std::string::String,
}

#[derive(::serde::Deserialize, ::serde::Serialize, ::typed_builder::TypedBuilder, Debug, PartialEq, Clone)]
#[derive(Default)]
#[serde(rename_all = "camelCase")]
pub struct TableDataList {
    /// A hash of this page of results.
    #[builder(setter(into))]
    #[serde(default)]
    pub etag: ::std::string::String,
    /// The resource type of the response.
    #[builder(setter(into))]
    #[serde(default)]
    pub kind: ::std::string::String,
    /// A token used for paging results. Providing this token instead of the startIndex parameter can
    /// help you retrieve stable results when an underlying table is changing.
    #[builder(setter(into))]
    #[serde(default)]
    pub page_token: ::std::string::String,
    /// Rows of results.
    #[builder(setter(into))]
    #[serde(default)]
    pub rows: ::std::vec::Vec<TableRow>,
    /// Total rows of the entire table. In order to show default value 0 we have to present it as
    /// string.
    #[builder(setter(into))]
    #[serde(with = "with::int64")]
    #[serde(default)]
    pub total_rows: i64,
}

/// Deprecated.
#[derive(::serde::Deserialize, ::serde::Serialize, ::typed_builder::TypedBuilder, Debug, PartialEq, Clone)]
#[derive(Default)]
#[serde(rename_all = "camelCase")]
pub struct Categories {
    /// Deprecated.
    #[builder(setter(into))]
    #[serde(default)]
    pub names: ::std::vec::Vec<::std::string::String>,
}

/// Optional. The policy tags attached to this field, used for field-level access control. If not
/// set, defaults to empty policy_tags.
#[derive(::serde::Deserialize, ::serde::Serialize, ::typed_builder::TypedBuilder, Debug, PartialEq, Clone)]
#[derive(Default)]
#[serde(rename_all = "camelCase")]
pub struct PolicyTags {
    /// A list of policy tag resource names. For example,
    /// "projects/1/locations/eu/taxonomies/2/policyTags/3". At most 1 policy tag is currently allowed.
    #[builder(setter(into))]
    #[serde(default)]
    pub names: ::std::vec::Vec<::std::string::String>,
}

/// Represents the type of a field element.
#[derive(::serde::Deserialize, ::serde::Serialize, ::typed_builder::TypedBuilder, Debug, PartialEq, Clone)]
#[derive(Default)]
#[serde(rename_all = "camelCase")]
pub struct RangeElementType {
    /// Required. The type of a field element. See TableFieldSchema.type.
    #[builder(setter(into))]
    #[serde(rename = "type")]
    #[serde(default)]
    pub ty: ::std::string::String,
}

/// Optional. Specifies the rounding mode to be used when storing values of NUMERIC and BIGNUMERIC
/// type.
#[derive(::serde::Deserialize, ::serde::Serialize, Debug, PartialEq, Clone)]
pub enum RoundingMode {
        /// ROUND_HALF_AWAY_FROM_ZERO rounds half values away from zero when applying precision and
        /// scale upon writing of NUMERIC and BIGNUMERIC values. For Scale: 0 1.1, 1.2, 1.3, 1.4 => 1
        /// 1.5, 1.6, 1.7, 1.8, 1.9 => 2
    #[serde(rename = "ROUND_HALF_AWAY_FROM_ZERO")]
    RoundHalfAwayFromZero,
        /// ROUND_HALF_EVEN rounds half values to the nearest even value when applying precision and
        /// scale upon writing of NUMERIC and BIGNUMERIC values. For Scale: 0 1.1, 1.2, 1.3, 1.4 => 1
        /// 1.5 => 2 1.6, 1.7, 1.8, 1.9 => 2 2.5 => 2
    #[serde(rename = "ROUND_HALF_EVEN")]
    RoundHalfEven,
}

/// A field in TableSchema
#[derive(::serde::Deserialize, ::serde::Serialize, ::typed_builder::TypedBuilder, Debug, PartialEq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct TableFieldSchema {
    /// Deprecated.
    #[builder(setter(into))]
    #[serde(default)]
    pub categories: Categories,
    /// Optional. Field collation can be set only when the type of field is STRING. The following values
    /// are supported: * 'und:ci': undetermined locale, case insensitive. * '': empty string. Default to
    /// case-sensitive behavior.
    #[builder(setter(into))]
    #[serde(default)]
    pub collation: ::std::option::Option<::std::string::String>,
    /// Optional. A SQL expression to specify the [default value]
    /// (https://cloud.google.com/bigquery/docs/default-values) for this field.
    #[builder(setter(into))]
    #[serde(default)]
    pub default_value_expression: ::std::option::Option<::std::string::String>,
    /// Optional. The field description. The maximum length is 1,024 characters.
    #[builder(setter(into))]
    #[serde(default)]
    pub description: ::std::option::Option<::std::string::String>,
    /// Optional. Describes the nested schema fields if the type property is set to RECORD.
    #[builder(setter(into))]
    #[serde(default)]
    pub fields: ::std::vec::Vec<TableFieldSchema>,
    /// Optional. Maximum length of values of this field for STRINGS or BYTES. If max_length is not
    /// specified, no maximum length constraint is imposed on this field. If type = "STRING", then
    /// max_length represents the maximum UTF-8 length of strings in this field. If type = "BYTES", then
    /// max_length represents the maximum number of bytes in this field. It is invalid to set this field
    /// if type ≠ "STRING" and ≠ "BYTES".
    #[builder(setter(into))]
    #[serde(with = "with::int64::option")]
    #[serde(default)]
    pub max_length: ::std::option::Option<i64>,
    /// Optional. The field mode. Possible values include NULLABLE, REQUIRED and REPEATED. The default
    /// value is NULLABLE.
    #[builder(setter(into))]
    #[serde(default)]
    pub mode: ::std::option::Option<::std::string::String>,
    /// Required. The field name. The name must contain only letters (a-z, A-Z), numbers (0-9), or
    /// underscores (_), and must start with a letter or underscore. The maximum length is 300
    /// characters.
    #[builder(setter(into))]
    #[serde(default)]
    pub name: ::std::string::String,
    /// Optional. The policy tags attached to this field, used for field-level access control. If not
    /// set, defaults to empty policy_tags.
    #[builder(setter(into))]
    #[serde(default)]
    pub policy_tags: PolicyTags,
    /// Optional. Precision (maximum number of total digits in base 10) and scale (maximum number of
    /// digits in the fractional part in base 10) constraints for values of this field for NUMERIC or
    /// BIGNUMERIC. It is invalid to set precision or scale if type ≠ "NUMERIC" and ≠ "BIGNUMERIC".
    /// If precision and scale are not specified, no value range constraint is imposed on this field
    /// insofar as values are permitted by the type. Values of this NUMERIC or BIGNUMERIC field must be
    /// in this range when: * Precision (P) and scale (S) are specified: [-10P-S + 10-S, 10P-S - 10-S] *
    /// Precision (P) is specified but not scale (and thus scale is interpreted to be equal to zero):
    /// [-10P + 1, 10P - 1]. Acceptable values for precision and scale if both are specified: * If type
    /// = "NUMERIC": 1 ≤ precision - scale ≤ 29 and 0 ≤ scale ≤ 9. * If type = "BIGNUMERIC": 1
    /// ≤ precision - scale ≤ 38 and 0 ≤ scale ≤ 38. Acceptable values for precision if only
    /// precision is specified but not scale (and thus scale is interpreted to be equal to zero): * If
    /// type = "NUMERIC": 1 ≤ precision ≤ 29. * If type = "BIGNUMERIC": 1 ≤ precision ≤ 38. If
    /// scale is specified but not precision, then it is invalid.
    #[builder(setter(into))]
    #[serde(with = "with::int64::option")]
    #[serde(default)]
    pub precision: ::std::option::Option<i64>,
    /// Represents the type of a field element.
    #[builder(setter(into))]
    #[serde(default)]
    pub range_element_type: RangeElementType,
    /// Optional. Specifies the rounding mode to be used when storing values of NUMERIC and BIGNUMERIC
    /// type.
    #[builder(setter(into))]
    pub rounding_mode: RoundingMode,
    /// Optional. See documentation for precision.
    #[builder(setter(into))]
    #[serde(with = "with::int64::option")]
    #[serde(default)]
    pub scale: ::std::option::Option<i64>,
    /// Required. The field data type. Possible values include: * STRING * BYTES * INTEGER (or INT64) *
    /// FLOAT (or FLOAT64) * BOOLEAN (or BOOL) * TIMESTAMP * DATE * TIME * DATETIME * GEOGRAPHY *
    /// NUMERIC * BIGNUMERIC * JSON * RECORD (or STRUCT) Use of RECORD/STRUCT indicates that the field
    /// contains a nested schema.
    #[builder(setter(into))]
    #[serde(rename = "type")]
    #[serde(default)]
    pub ty: ::std::string::String,
}

/// Information about a logical view.
#[derive(::serde::Deserialize, ::serde::Serialize, ::typed_builder::TypedBuilder, Debug, PartialEq, Clone)]
#[derive(Default)]
#[serde(rename_all = "camelCase")]
pub struct View {
    /// Specifices the privacy policy for the view.
    #[builder(setter(into))]
    #[serde(default)]
    pub privacy_policy: PrivacyPolicy,
    /// True if view is defined in legacy SQL dialect, false if in GoogleSQL.
    #[builder(setter(into))]
    #[serde(default)]
    pub use_legacy_sql: bool,
}

#[derive(::serde::Deserialize, ::serde::Serialize, ::typed_builder::TypedBuilder, Debug, PartialEq, Clone)]
#[derive(Default)]
#[serde(rename_all = "camelCase")]
pub struct Tables {
    /// Clustering specification for this table, if configured.
    #[builder(setter(into))]
    #[serde(default)]
    pub clustering: Clustering,
    /// Output only. The time when this table was created, in milliseconds since the epoch.
    #[builder(setter(into))]
    #[serde(with = "with::int64")]
    #[serde(default)]
    pub creation_time: i64,
    /// The time when this table expires, in milliseconds since the epoch. If not present, the table
    /// will persist indefinitely. Expired tables will be deleted and their storage reclaimed.
    #[builder(setter(into))]
    #[serde(with = "with::int64")]
    #[serde(default)]
    pub expiration_time: i64,
    /// The user-friendly name for this table.
    #[builder(setter(into))]
    #[serde(default)]
    pub friendly_name: ::std::string::String,
    /// An opaque ID of the table.
    #[builder(setter(into))]
    #[serde(default)]
    pub id: ::std::string::String,
    /// The resource type.
    #[builder(setter(into))]
    #[serde(default)]
    pub kind: ::std::string::String,
    /// The labels associated with this table. You can use these to organize and group your tables.
    #[builder(setter(into))]
    #[serde(default)]
    pub labels: Labels,
    /// The range partitioning for this table.
    #[builder(setter(into))]
    #[serde(default)]
    pub range_partitioning: RangePartitioning,
    /// Optional. If set to true, queries including this table must specify a partition filter. This
    /// filter is used for partition elimination.
    #[builder(setter(into))]
    #[serde(default)]
    pub require_partition_filter: ::std::option::Option<bool>,
    /// A reference uniquely identifying table.
    #[builder(setter(into))]
    #[serde(default)]
    pub table_reference: TableReference,
    /// The time-based partitioning for this table.
    #[builder(setter(into))]
    #[serde(default)]
    pub time_partitioning: TimePartitioning,
    /// The type of table.
    #[builder(setter(into))]
    #[serde(rename = "type")]
    #[serde(default)]
    pub ty: ::std::string::String,
    /// Information about a logical view.
    #[builder(setter(into))]
    #[serde(default)]
    pub view: View,
}

/// Partial projection of the metadata for a given table in a list response.
#[derive(::serde::Deserialize, ::serde::Serialize, ::typed_builder::TypedBuilder, Debug, PartialEq, Clone)]
#[derive(Default)]
#[serde(rename_all = "camelCase")]
pub struct TableList {
    /// A hash of this page of results.
    #[builder(setter(into))]
    #[serde(default)]
    pub etag: ::std::string::String,
    /// The type of list.
    #[builder(setter(into))]
    #[serde(default)]
    pub kind: ::std::string::String,
    /// A token to request the next page of results.
    #[builder(setter(into))]
    #[serde(default)]
    pub next_page_token: ::std::string::String,
    /// Tables in the requested dataset.
    #[builder(setter(into))]
    #[serde(default)]
    pub tables: ::std::vec::Vec<Tables>,
    /// The total number of tables in the dataset.
    #[builder(setter(into))]
    #[serde(default)]
    pub total_items: i64,
}

/// Reason for not using metadata caching for the table.
#[derive(::serde::Deserialize, ::serde::Serialize, Debug, PartialEq, Clone)]
pub enum UnusedReason {
        /// Metadata cache was outside the table's maxStaleness.
    #[serde(rename = "EXCEEDED_MAX_STALENESS")]
    ExceededMaxStaleness,
        /// Metadata caching feature is not enabled. [Update BigLake tables]
        /// (/bigquery/docs/create-cloud-storage-table-biglake#update-biglake-tables) to enable the
        /// metadata caching.
    #[serde(rename = "METADATA_CACHING_NOT_ENABLED")]
    MetadataCachingNotEnabled,
        /// Other unknown reason.
    #[serde(rename = "OTHER_REASON")]
    OtherReason,
}

/// Table level detail on the usage of metadata caching. Only set for Metadata caching eligible
/// tables referenced in the query.
#[derive(::serde::Deserialize, ::serde::Serialize, ::typed_builder::TypedBuilder, Debug, PartialEq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct TableMetadataCacheUsage {
    /// Free form human-readable reason metadata caching was unused for the job.
    #[builder(setter(into))]
    #[serde(default)]
    pub explanation: ::std::string::String,
    /// Metadata caching eligible table referenced in the query.
    #[builder(setter(into))]
    #[serde(default)]
    pub table_reference: TableReference,
    /// [Table type](/bigquery/docs/reference/rest/v2/tables#Table.FIELDS.type).
    #[builder(setter(into))]
    #[serde(default)]
    pub table_type: ::std::string::String,
    /// Reason for not using metadata caching for the table.
    #[builder(setter(into))]
    pub unused_reason: UnusedReason,
}

#[derive(::serde::Deserialize, ::serde::Serialize, ::typed_builder::TypedBuilder, Debug, PartialEq, Clone)]
#[derive(Default)]
#[serde(rename_all = "camelCase")]
pub struct TableReference {
    /// Required. The ID of the dataset containing this table.
    #[builder(setter(into))]
    #[serde(default)]
    pub dataset_id: ::std::string::String,
    /// Required. The ID of the project containing this table.
    #[builder(setter(into))]
    #[serde(default)]
    pub project_id: ::std::string::String,
    /// Required. The ID of the table. The ID can contain Unicode characters in category L (letter), M
    /// (mark), N (number), Pc (connector, including underscore), Pd (dash), and Zs (space). For more
    /// information, see [General
    /// Category](https://wikipedia.org/wiki/Unicode_character_property#General_Category). The maximum
    /// length is 1,024 characters. Certain operations allow suffixing of the table ID with a partition
    /// decorator, such as `sample_table$20190123`.
    #[builder(setter(into))]
    #[serde(default)]
    pub table_id: ::std::string::String,
}

/// Optional. Output only. Replication status of configured replication.
#[derive(::serde::Deserialize, ::serde::Serialize, Debug, PartialEq, Clone)]
pub enum ReplicationStatus {
        /// Replication is Active with no errors.
    #[serde(rename = "ACTIVE")]
    Active,
        /// Source object is deleted.
    #[serde(rename = "SOURCE_DELETED")]
    SourceDeleted,
        /// Source revoked replication permissions.
    #[serde(rename = "PERMISSION_DENIED")]
    PermissionDenied,
        /// Source configuration doesn’t allow replication.
    #[serde(rename = "UNSUPPORTED_CONFIGURATION")]
    UnsupportedConfiguration,
}

/// Replication info of a table created using `AS REPLICA` DDL like: `CREATE MATERIALIZED VIEW mv1
/// AS REPLICA OF src_mv`
#[derive(::serde::Deserialize, ::serde::Serialize, ::typed_builder::TypedBuilder, Debug, PartialEq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct TableReplicationInfo {
    /// Optional. Output only. If source is a materialized view, this field signifies the last refresh
    /// time of the source.
    #[builder(setter(into))]
    #[serde(with = "with::int64::option")]
    #[serde(default)]
    pub replicated_source_last_refresh_time: ::std::option::Option<i64>,
    /// Optional. Output only. Replication error that will permanently stopped table replication.
    #[builder(setter(into))]
    #[serde(default)]
    pub replication_error: ErrorProto,
    /// Required. Specifies the interval at which the source table is polled for updates.
    #[builder(setter(into))]
    #[serde(with = "with::int64")]
    #[serde(default)]
    pub replication_interval_ms: i64,
    /// Optional. Output only. Replication status of configured replication.
    #[builder(setter(into))]
    pub replication_status: ReplicationStatus,
    /// Required. Source table reference that is replicated.
    #[builder(setter(into))]
    #[serde(default)]
    pub source_table: TableReference,
}

#[derive(::serde::Deserialize, ::serde::Serialize, ::typed_builder::TypedBuilder, Debug, PartialEq, Clone)]
#[derive(Default)]
#[serde(rename_all = "camelCase")]
pub struct TableRow {
    /// Represents a single row in the result set, consisting of one or more fields.
    #[builder(setter(into))]
    #[serde(default)]
    pub f: ::std::vec::Vec<TableCell>,
}

/// Schema of a table
#[derive(::serde::Deserialize, ::serde::Serialize, ::typed_builder::TypedBuilder, Debug, PartialEq, Clone)]
#[derive(Default)]
#[serde(rename_all = "camelCase")]
pub struct TableSchema {
    /// Describes the fields in a table.
    #[builder(setter(into))]
    #[serde(default)]
    pub fields: ::std::vec::Vec<TableFieldSchema>,
}

/// Request message for `TestIamPermissions` method.
#[derive(::serde::Deserialize, ::serde::Serialize, ::typed_builder::TypedBuilder, Debug, PartialEq, Clone)]
#[derive(Default)]
#[serde(rename_all = "camelCase")]
pub struct TestIamPermissionsRequest {
    /// The set of permissions to check for the `resource`. Permissions with wildcards (such as `*` or
    /// `storage.*`) are not allowed. For more information see [IAM
    /// Overview](https://cloud.google.com/iam/docs/overview#permissions).
    #[builder(setter(into))]
    #[serde(default)]
    pub permissions: ::std::vec::Vec<::std::string::String>,
}

/// Response message for `TestIamPermissions` method.
#[derive(::serde::Deserialize, ::serde::Serialize, ::typed_builder::TypedBuilder, Debug, PartialEq, Clone)]
#[derive(Default)]
#[serde(rename_all = "camelCase")]
pub struct TestIamPermissionsResponse {
    /// A subset of `TestPermissionsRequest.permissions` that the caller is allowed.
    #[builder(setter(into))]
    #[serde(default)]
    pub permissions: ::std::vec::Vec<::std::string::String>,
}

#[derive(::serde::Deserialize, ::serde::Serialize, ::typed_builder::TypedBuilder, Debug, PartialEq, Clone)]
#[derive(Default)]
#[serde(rename_all = "camelCase")]
pub struct TimePartitioning {
    /// Optional. Number of milliseconds for which to keep the storage for a partition. A wrapper is
    /// used here because 0 is an invalid value.
    #[builder(setter(into))]
    #[serde(with = "with::int64::option")]
    #[serde(default)]
    pub expiration_ms: ::std::option::Option<i64>,
    /// Optional. If not set, the table is partitioned by pseudo column '_PARTITIONTIME'; if set, the
    /// table is partitioned by this field. The field must be a top-level TIMESTAMP or DATE field. Its
    /// mode must be NULLABLE or REQUIRED. A wrapper is used here because an empty string is an invalid
    /// value.
    #[builder(setter(into))]
    #[serde(default)]
    pub field: ::std::option::Option<::std::string::String>,
    /// If set to true, queries over this table require a partition filter that can be used for
    /// partition elimination to be specified. This field is deprecated; please set the field with the
    /// same name on the table itself instead. This field needs a wrapper because we want to output the
    /// default value, false, if the user explicitly set it.
    #[builder(setter(into))]
    #[serde(default)]
    pub require_partition_filter: bool,
    /// Required. The supported types are DAY, HOUR, MONTH, and YEAR, which will generate one partition
    /// per day, hour, month, and year, respectively.
    #[builder(setter(into))]
    #[serde(rename = "type")]
    #[serde(default)]
    pub ty: ::std::string::String,
}

/// Information about a single training query run for the model.
#[derive(::serde::Deserialize, ::serde::Serialize, ::typed_builder::TypedBuilder, Debug, PartialEq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct TrainingRun {
    /// Output only. Global explanation contains the explanation of top features on the class level.
    /// Applies to classification models only.
    #[builder(setter(into))]
    #[serde(default)]
    pub class_level_global_explanations: ::std::vec::Vec<GlobalExplanation>,
    /// Output only. Data split result of the training run. Only set when the input data is actually
    /// split.
    #[builder(setter(into))]
    pub data_split_result: DataSplitResult,
    /// Output only. The evaluation metrics over training/eval data that were computed at the end of
    /// training.
    #[builder(setter(into))]
    pub evaluation_metrics: EvaluationMetrics,
    /// Output only. Global explanation contains the explanation of top features on the model level.
    /// Applies to both regression and classification models.
    #[builder(setter(into))]
    #[serde(default)]
    pub model_level_global_explanation: GlobalExplanation,
    /// Output only. Output of each iteration run, results.size() <= max_iterations.
    #[builder(setter(into))]
    #[serde(default)]
    pub results: ::std::vec::Vec<IterationResult>,
    /// Output only. The start time of this training run.
    #[builder(setter(into))]
    #[serde(default)]
    pub start_time: ::std::string::String,
    /// Output only. Options that were used for this training run, includes user specified and default
    /// options that were used.
    #[builder(setter(into))]
    #[serde(default)]
    pub training_options: TrainingOptions,
    /// Output only. The start time of this training run, in milliseconds since epoch.
    #[builder(setter(into))]
    #[serde(with = "with::int64")]
    #[serde(default)]
    pub training_start_time: i64,
    /// The model id in the [Vertex AI Model
    /// Registry](https://cloud.google.com/vertex-ai/docs/model-registry/introduction) for this training
    /// run.
    #[builder(setter(into))]
    #[serde(default)]
    pub vertex_ai_model_id: ::std::string::String,
    /// Output only. The model version in the [Vertex AI Model
    /// Registry](https://cloud.google.com/vertex-ai/docs/model-registry/introduction) for this training
    /// run.
    #[builder(setter(into))]
    #[serde(default)]
    pub vertex_ai_model_version: ::std::string::String,
}

/// [Alpha] Information of a multi-statement transaction.
#[derive(::serde::Deserialize, ::serde::Serialize, ::typed_builder::TypedBuilder, Debug, PartialEq, Clone)]
#[derive(Default)]
#[serde(rename_all = "camelCase")]
pub struct TransactionInfo {
    /// Output only. [Alpha] Id of the transaction.
    #[builder(setter(into))]
    #[serde(default)]
    pub transaction_id: ::std::string::String,
}

/// Information about a single transform column.
#[derive(::serde::Deserialize, ::serde::Serialize, ::typed_builder::TypedBuilder, Debug, PartialEq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct TransformColumn {
    /// Output only. Name of the column.
    #[builder(setter(into))]
    #[serde(default)]
    pub name: ::std::string::String,
    /// Output only. The SQL expression used in the column transform.
    #[builder(setter(into))]
    #[serde(default)]
    pub transform_sql: ::std::string::String,
    /// Output only. Data type of the column after the transform.
    #[builder(setter(into))]
    #[serde(rename = "type")]
    pub ty: StandardSqlDataType,
}

/// Request format for undeleting a dataset.
#[derive(::serde::Deserialize, ::serde::Serialize, ::typed_builder::TypedBuilder, Debug, PartialEq, Clone)]
#[derive(Default)]
#[serde(rename_all = "camelCase")]
pub struct UndeleteDatasetRequest {
    /// Optional. The exact time when the dataset was deleted. If not specified, it will undelete the
    /// most recently deleted version.
    #[builder(setter(into))]
    #[serde(default)]
    pub deletion_time: ::std::option::Option<::std::string::String>,
}

/// This is used for defining User Defined Function (UDF) resources only when using legacy SQL.
/// Users of GoogleSQL should leverage either DDL (e.g. CREATE [TEMPORARY] FUNCTION ... ) or the
/// Routines API to define UDF resources. For additional information on migrating, see:
/// https://cloud.google.com/bigquery/docs/reference/standard-sql/migrating-from-legacy-sql#differences_in_user-defined_javascript_functions
#[derive(::serde::Deserialize, ::serde::Serialize, ::typed_builder::TypedBuilder, Debug, PartialEq, Clone)]
#[derive(Default)]
#[serde(rename_all = "camelCase")]
pub struct UserDefinedFunctionResource {
    /// [Pick one] An inline resource that contains code for a user-defined function (UDF). Providing a
    /// inline code resource is equivalent to providing a URI for a file containing the same code.
    #[builder(setter(into))]
    #[serde(default)]
    pub inline_code: ::std::string::String,
    /// [Pick one] A code resource to load from a Google Cloud Storage URI (gs://bucket/path).
    #[builder(setter(into))]
    #[serde(default)]
    pub resource_uri: ::std::string::String,
}

/// Statistics for a vector search query. Populated as part of JobStatistics2.
#[derive(::serde::Deserialize, ::serde::Serialize, ::typed_builder::TypedBuilder, Debug, PartialEq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct VectorSearchStatistics {
    /// When `indexUsageMode` is `UNUSED` or `PARTIALLY_USED`, this field explains why indexes were not
    /// used in all or part of the vector search query. If `indexUsageMode` is `FULLY_USED`, this field
    /// is not populated.
    #[builder(setter(into))]
    #[serde(default)]
    pub index_unused_reasons: ::std::vec::Vec<IndexUnusedReason>,
    /// Specifies the index usage mode for the query.
    #[builder(setter(into))]
    pub index_usage_mode: IndexUsageMode,
}

/// Describes the definition of a logical view.
#[derive(::serde::Deserialize, ::serde::Serialize, ::typed_builder::TypedBuilder, Debug, PartialEq, Clone)]
#[derive(Default)]
#[serde(rename_all = "camelCase")]
pub struct ViewDefinition {
    /// Optional. Specifices the privacy policy for the view.
    #[builder(setter(into))]
    #[serde(default)]
    pub privacy_policy: PrivacyPolicy,
    /// Required. A query that BigQuery executes when the view is referenced.
    #[builder(setter(into))]
    #[serde(default)]
    pub query: ::std::string::String,
    /// True if the column names are explicitly specified. For example by using the 'CREATE VIEW v(c1,
    /// c2) AS ...' syntax. Can only be set for GoogleSQL views.
    #[builder(setter(into))]
    #[serde(default)]
    pub use_explicit_column_names: bool,
    /// Specifies whether to use BigQuery's legacy SQL for this view. The default value is true. If set
    /// to false, the view will use BigQuery's GoogleSQL:
    /// https://cloud.google.com/bigquery/sql-reference/ Queries and views that reference this view must
    /// use the same flag value. A wrapper is used here because the default value is True.
    #[builder(setter(into))]
    #[serde(default)]
    pub use_legacy_sql: bool,
    /// Describes user-defined function resources used in the query.
    #[builder(setter(into))]
    #[serde(default)]
    pub user_defined_function_resources: ::std::vec::Vec<UserDefinedFunctionResource>,
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

