use std::num::NonZeroU8;
use std::sync::atomic::{AtomicU8, Ordering};

use tracing_core::Interest;

static MAX_SEVERITY: AtomicU8 = AtomicU8::new(match Severity::STATIC_MAX_LEVEL {
    Some(sev) => sev as u8,
    None => Severity::Notice as u8,
});

/// Google logging severity. Used to override [tracing::Level] in events
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum Severity {
    /// (100) Debug or trace information.
    ///
    /// Maps from [tracing::Level::TRACE]
    Debug = 1,
    ///	(200) Routine information, such as ongoing status or performance.
    ///
    /// Maps from [tracing::Level::DEBUG]
    Info = 2,
    /// (300) Normal but significant events, such as start up, shut down, or a configuration change.
    ///
    /// Maps from [tracing::Level::INFO]
    #[default]
    Notice = 3,
    /// (400) Warning events might cause problems.
    ///
    /// Maps from [tracing::Level::WARN]
    Warning = 4,
    /// (500) Error events are likely to cause problems.
    ///
    /// Maps from [tracing::Level::ERROR]
    Error = 5,
    /// (600) Critical events cause more severe problems or outages.
    Critical = 6,
    /// (700) A person must take an action immediately.
    ///
    /// Maps from [tracing::Level::ERROR], when `alert = true` is specified
    Alert = 7,
    /// (800) One or more systems are unusable.
    Emergency = 8,
}

impl Severity {
    pub const STATIC_MAX_LEVEL: Option<Self> =
        Self::from_tracing_filter_unknown_alert(tracing::level_filters::STATIC_MAX_LEVEL);

    pub const KEY: &'static str = "severity";

    pub const ALL: [Self; 8] = [
        Self::Debug,
        Self::Info,
        Self::Notice,
        Self::Warning,
        Self::Error,
        Self::Critical,
        Self::Alert,
        Self::Emergency,
    ];

    pub const fn should_alert(&self) -> bool {
        match self {
            Self::Critical | Self::Alert | Self::Emergency => true,
            _ => false,
        }
    }

    #[inline]
    pub const fn const_cmp(&self, rhs: &Self) -> std::cmp::Ordering {
        let self_int = *self as u8;
        let rhs_int = *rhs as u8;

        if self_int > rhs_int {
            std::cmp::Ordering::Greater
        } else if self_int < rhs_int {
            std::cmp::Ordering::Less
        } else {
            std::cmp::Ordering::Equal
        }
    }

    #[inline]
    pub const fn worst_opt(this: Option<Self>, other: Option<Self>) -> Option<Self> {
        match (this, other) {
            (Some(this), Some(other)) => Some(this.worst(other)),
            (Some(one), None) | (None, Some(one)) => Some(one),
            (None, None) => None,
        }
    }

    #[inline]
    pub const fn worst(self, other: Self) -> Self {
        if self.as_int() < other.as_int() {
            other
        } else {
            self
        }
    }

    pub fn interest(meta: &tracing::Metadata<'_>) -> Interest {
        match Self::STATIC_MAX_LEVEL {
            None => Interest::always(),
            Some(max) => {
                let alert = meta.fields().field("alert").is_some();
                let meta_sev = Self::from_tracing(meta.level().clone(), alert);

                if max < meta_sev {
                    Interest::always()
                } else {
                    Interest::sometimes()
                }
            }
        }
    }

    #[inline]
    pub fn max_level(order: Ordering) -> Option<Self> {
        Self::from_int(MAX_SEVERITY.load(order) as isize)
    }

    #[inline]
    pub fn tracing_max_level() -> Option<Self> {
        Self::from_tracing_filter_unknown_alert(tracing::level_filters::LevelFilter::current())
    }

    #[inline]
    pub fn set_max_level(&self, order: Ordering) {
        MAX_SEVERITY.store(*self as u8, order);
    }

    pub const fn into_tracing(&self) -> tracing::Level {
        match self {
            Self::Debug => tracing::Level::TRACE,
            Self::Info => tracing::Level::DEBUG,
            Self::Notice => tracing::Level::INFO,
            Self::Warning => tracing::Level::WARN,
            Self::Error | Self::Alert | Self::Critical | Self::Emergency => tracing::Level::ERROR,
        }
    }

    pub const fn from_tracing(tracing: tracing::Level, alert: bool) -> Self {
        match tracing {
            tracing::Level::TRACE => Self::Debug,
            tracing::Level::DEBUG => Self::Info,
            tracing::Level::INFO => Self::Notice,
            tracing::Level::WARN => Self::Warning,
            tracing::Level::ERROR if !alert => Self::Error,
            tracing::Level::ERROR => Self::Alert,
        }
    }

    pub const fn from_tracing_unknown_alert(tracing: tracing::Level) -> Option<Self> {
        match tracing {
            tracing::Level::TRACE => Some(Self::Debug),
            tracing::Level::DEBUG => Some(Self::Info),
            tracing::Level::INFO => Some(Self::Notice),
            tracing::Level::WARN => Some(Self::Warning),
            tracing::Level::ERROR => None,
        }
    }

    pub const fn from_tracing_filter_unknown_alert(
        tracing: tracing::level_filters::LevelFilter,
    ) -> Option<Self> {
        match tracing.into_level() {
            Some(level) => Self::from_tracing_unknown_alert(level),
            None => None,
        }
    }

    pub const fn from_tracing_filter(
        tracing: tracing::level_filters::LevelFilter,
        alert: bool,
    ) -> Option<Self> {
        match tracing.into_level() {
            Some(level) => Some(Self::from_tracing(level, alert)),
            None => None,
        }
    }

    pub const fn from_int(i: isize) -> Option<Self> {
        match i {
            1 | 100 => Some(Self::Debug),
            2 | 200 => Some(Self::Info),
            3 | 300 => Some(Self::Notice),
            4 | 400 => Some(Self::Warning),
            5 | 500 => Some(Self::Error),
            6 | 600 => Some(Self::Critical),
            7 | 700 => Some(Self::Alert),
            8 | 800 => Some(Self::Emergency),
            _ => None,
        }
    }

    pub const fn as_non_zero(&self) -> NonZeroU8 {
        NonZeroU8::new(*self as u8).expect("no variant is 0")
    }

    pub const fn as_int(&self) -> u16 {
        *self as u16 * 100
    }

    #[inline]
    pub const fn as_upper_str(&self) -> &'static str {
        match self {
            Self::Debug => "DEBUG",
            Self::Alert => "ALERT",
            Self::Info => "INFO",
            Self::Notice => "NOTICE",
            Self::Warning => "WARNING",
            Self::Error => "ERROR",
            Self::Critical => "CRITICAL",
            Self::Emergency => "EMERGENCY",
        }
    }

    #[inline]
    pub fn from_upper_str(s: &str) -> Option<Self> {
        match s {
            "DEBUG" => Some(Self::Debug),
            "ALERT" => Some(Self::Alert),
            "INFO" => Some(Self::Info),
            "NOTICE" => Some(Self::Notice),
            "WARNING" => Some(Self::Warning),
            "ERROR" => Some(Self::Error),
            "CRITICAL" => Some(Self::Critical),
            "EMERGENCY" => Some(Self::Emergency),
            _ => None,
        }
    }

    #[inline]
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Debug => "Debug",
            Self::Alert => "Alert",
            Self::Info => "Info",
            Self::Notice => "Notice",
            Self::Warning => "Warning",
            Self::Error => "Error",
            Self::Critical => "Critical",
            Self::Emergency => "Emergency",
        }
    }

    #[inline]
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "Debug" => Some(Self::Debug),
            "Alert" => Some(Self::Alert),
            "Info" => Some(Self::Info),
            "Notice" => Some(Self::Notice),
            "Warning" => Some(Self::Warning),
            "Error" => Some(Self::Error),
            "Critical" => Some(Self::Critical),
            "Emergency" => Some(Self::Emergency),
            _ => None,
        }
    }

    #[cfg(feature = "valuable")]
    pub fn from_value(value: &valuable::Value<'_>) -> Option<Self> {
        let valuable::Value::Enumerable(enumer) = value else {
            return None;
        };

        if enumer.definition().name() != Self::KEY {
            return None;
        }

        let variant = enumer.variant();

        if !matches!(variant.fields(), valuable::Fields::Unnamed(0)) {
            return None;
        }

        Self::from_str(variant.name())
    }

    pub fn record(&self, span: &tracing::Span) {
        #[cfg(feature = "valuable")]
        {
            span.record(Self::KEY, valuable::Valuable::as_value(self));
        }
        #[cfg(not(feature = "valuable"))]
        {
            span.record(Self::KEY, self.as_upper_str());
        }
    }
}

impl PartialEq<tracing::Level> for Severity {
    fn eq(&self, other: &tracing::Level) -> bool {
        match self.partial_cmp(other) {
            Some(ord) => ord.is_eq(),
            None => false,
        }
    }
}

impl PartialOrd<tracing::Level> for Severity {
    fn partial_cmp(&self, other: &tracing::Level) -> Option<std::cmp::Ordering> {
        // if the tracing level isn't Error, we can convert without needing to know
        // if we should alert.
        if *other != tracing::Level::ERROR {
            return Some(self.cmp(&Self::from_tracing(other.clone(), false)));
        }

        match self {
            // these variants must always be specified manually (i.e converting
            // from tracing::Level never becomes these variants), so they're always
            // greater.
            Self::Critical | Self::Emergency => Some(std::cmp::Ordering::Greater),
            // without knowing if we should alert, we can't be sure how we compare
            // if 'self' is either of these variants (when tracing::Level == Error)
            Self::Alert | Self::Error => None,
            // otherwise if the tracing::Level is Error and we aren't we're clearly
            // less than it.
            Self::Debug | Self::Notice | Self::Info | Self::Warning => {
                Some(std::cmp::Ordering::Less)
            }
        }
    }
}

impl PartialOrd for Severity {
    #[inline]
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Severity {
    #[inline]
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.const_cmp(other)
    }
}

impl serde::Serialize for Severity {
    #[inline]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(self.as_upper_str())
    }
}

#[cfg(feature = "valuable")]
impl valuable::Valuable for Severity {
    fn as_value(&self) -> valuable::Value<'_> {
        valuable::Value::Enumerable(self)
    }

    fn visit(&self, visit: &mut dyn valuable::Visit) {
        visit.visit_value(self.as_value());
    }
}

#[cfg(feature = "valuable")]
macro_rules! variant {
    ($variant:ident) => {{ valuable::VariantDef::new(Severity::$variant.as_str(), valuable::Fields::Unnamed(0)) }};
    (& $variant:ident) => {{
        const VAR: valuable::VariantDef<'static> = variant!($variant);
        &VAR
    }};
}

#[cfg(feature = "valuable")]
impl valuable::Enumerable for Severity {
    fn variant(&self) -> valuable::Variant<'_> {
        valuable::Variant::Static(match self {
            Self::Debug => variant!(&Debug),
            Self::Alert => variant!(&Alert),
            Self::Info => variant!(&Info),
            Self::Notice => variant!(&Notice),
            Self::Warning => variant!(&Warning),
            Self::Error => variant!(&Error),
            Self::Critical => variant!(&Critical),
            Self::Emergency => variant!(&Emergency),
        })
    }

    fn definition(&self) -> valuable::EnumDef<'_> {
        const VARIANTS: [valuable::VariantDef<'static>; Severity::ALL.len()] = [
            variant!(Debug),
            variant!(Info),
            variant!(Notice),
            variant!(Warning),
            variant!(Error),
            variant!(Critical),
            variant!(Alert),
            variant!(Emergency),
        ];

        valuable::EnumDef::new_static(Self::KEY, &VARIANTS)
    }
}
