/// A date. Follows the same range restrictions as placed on
/// 'google.protobuf.Timestamp', in that valid dates must fall between
/// '01/01/-9999' and '12/31/9999'.
#[derive(serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
#[derive(Copy, Eq, PartialOrd, Ord, Hash, Clone, PartialEq, ::prost::Message)]
pub struct Date {
    /// Day of the month. Must be a valid day for the specified month.
    #[prost(uint32, required, tag = "1")]
    pub day: u32,
    /// The month this date falls in.
    #[prost(enumeration = "date::Month", required, tag = "2")]
    pub month: i32,
    /// The year this date falls in. Must be in the range `-9999..=9999` to
    /// conform to the range of dates allowed in the well known protobuf type
    /// \[`google.protobuf.Timestamp`\].
    #[prost(int32, required, tag = "3")]
    pub year: i32,
}
/// Nested message and enum types in `Date`.
pub mod date {
    /// Months of the year. Since protobuf enums require an index 0 variant,
    /// each month is shifted by '-1' from the human friendly numeric version,
    /// i.e January is 0 instead of 1.
    #[derive(serde::Deserialize, serde::Serialize)]
    #[serde(rename_all = "camelCase")]
    #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
    #[repr(i32)]
    pub enum Month {
        January = 0,
        February = 1,
        March = 2,
        April = 3,
        May = 4,
        June = 5,
        July = 6,
        August = 7,
        September = 8,
        October = 9,
        November = 10,
        December = 11,
    }
    impl Month {
        /// String value of the enum field names used in the ProtoBuf definition.
        ///
        /// The values are not transformed in any way and thus are considered stable
        /// (if the ProtoBuf definition does not change) and safe for programmatic use.
        pub fn as_str_name(&self) -> &'static str {
            match self {
                Month::January => "JANUARY",
                Month::February => "FEBRUARY",
                Month::March => "MARCH",
                Month::April => "APRIL",
                Month::May => "MAY",
                Month::June => "JUNE",
                Month::July => "JULY",
                Month::August => "AUGUST",
                Month::September => "SEPTEMBER",
                Month::October => "OCTOBER",
                Month::November => "NOVEMBER",
                Month::December => "DECEMBER",
            }
        }
    }
}
/// Defines file types at different stages of PSO/PM-led data QA/QC.
/// Ordered by "priority" (i.e Signed off files should always be used if found,
/// and edits > saves, etc).
#[derive(serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
#[repr(i32)]
pub enum FileType {
    /// A one-off file saved in the 'Other/' directory, ususally manually by a
    /// PSO/PM.
    Other = 0,
    /// A template file, found in the 'Templates/' directory.
    Template = 1,
    /// An intermediate save file, found in the 'Saves/' directory.
    Save = 2,
    /// The final save for a day, also found in 'Saves/', but with a '-Final'
    /// filename component. There may be multiple of these.
    FinalSave = 3,
    /// An edited save file, found in the 'Edits/' directory.
    Edit = 4,
    /// An edited file, also found in 'Edits/', but with a '-Final'
    /// filename component. There may be multiple of these.
    FinalEdit = 5,
    /// A Signed off file, found in the 'SignOffs/' directory. Will contain one or
    /// more sets of initials in square brackets (i.e '\[MR\]').
    SignedOff = 6,
}
impl FileType {
    /// String value of the enum field names used in the ProtoBuf definition.
    ///
    /// The values are not transformed in any way and thus are considered stable
    /// (if the ProtoBuf definition does not change) and safe for programmatic use.
    pub fn as_str_name(&self) -> &'static str {
        match self {
            FileType::Other => "OTHER",
            FileType::Template => "TEMPLATE",
            FileType::Save => "SAVE",
            FileType::FinalSave => "FINAL_SAVE",
            FileType::Edit => "EDIT",
            FileType::FinalEdit => "FINAL_EDIT",
            FileType::SignedOff => "SIGNED_OFF",
        }
    }
}
/// Generic message to specify progress.
///
/// 'total' can be 0.
///
/// To represent a generic 0-100% progress, set 'total' to 100 (or a multiple
/// of 100 to have a higher precision).
///
/// Invariants:
/// - 'completed' must be less than or equal to 'total'.
#[derive(serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Progress {
    /// The amount of progress completed.
    #[prost(uint32, required, tag = "1")]
    pub completed: u32,
    /// The total amount of progress
    #[prost(uint32, required, tag = "2")]
    pub total: u32,
}
/// Well defined species names, pulled from the main Mysticetus client list
/// definition.
#[derive(serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
#[repr(i32)]
pub enum Species {
    Unknown = 0,
    AmazonianManatee = 1,
    AndrewsBeakedWhale = 2,
    AntarcticFurSeal = 3,
    AntarcticMinkeWhale = 4,
    ArnouxsBeakedWhale = 5,
    AtlanticHumpbackDolphin = 6,
    AtlanticSpottedDolphin = 7,
    AtlanticWhitesidedDolphin = 8,
    AustralianFurSeal = 9,
    AustralianSeaLion = 10,
    BahamondesBeakedWhale = 11,
    BaikalSealOrNerpa = 12,
    BairdsBeakedWhale = 13,
    BeardedSeal = 14,
    BelugaWhale = 15,
    BlainvillesBeakedWhale = 16,
    BlueWhale = 17,
    Boto = 18,
    BowheadWhale = 19,
    BrydesWhale = 20,
    BurmeistersPorpoise = 21,
    CaliforniaSeaLion = 22,
    CaspianSeal = 23,
    ChileanDolphin = 24,
    ClymeneDolphin = 25,
    CommersonsDolphin = 26,
    CommonBottlenoseDolphin = 27,
    CommonDolphin = 28,
    CommonMinkeWhale = 29,
    CrabeaterSeal = 30,
    CuviersBeakedWhale = 31,
    DallsPorpoise = 32,
    Dugong = 33,
    DuskyDolphin = 34,
    DwarfSpermWhale = 35,
    FalseKillerWhale = 36,
    FinWhale = 37,
    FinlessPorpoise = 38,
    FlatbackSeaTurtle = 39,
    FloridaManatee = 40,
    Franciscana = 41,
    FrasersDolphin = 42,
    GalapagosSeaLion = 43,
    GervaisBeakedWhale = 44,
    GinkgotoothedBeakedWhale = 45,
    GraySeal = 46,
    GrayWhale = 47,
    GraysBeakedWhale = 48,
    GreenSeaTurtle = 49,
    GuadalupeFurSeal = 50,
    HarborPorpoise = 51,
    HarborSeal = 52,
    HarpSeal = 53,
    HawaiianMonkSeal = 54,
    HawksbillSeaTurtle = 55,
    HeavisidesDolphin = 56,
    HectorsBeakedWhale = 57,
    HectorsDolphin = 58,
    HoodedSeal = 59,
    HourglassDolphin = 60,
    HubbsBeakedWhale = 61,
    HumpbackWhale = 62,
    IndopacificBottlenoseDolphin = 63,
    IndopacificHumpbackDolphin = 64,
    IrrawaddyDolphin = 65,
    JuanFernandezFurSeal = 66,
    KempsRidleySeaTurtle = 67,
    KillerWhale = 68,
    LeatherbackSeaTurtle = 69,
    LeopardSeal = 70,
    LoggerheadSeaTurtle = 71,
    LongbeakedCommonDolphin = 72,
    LongfinnedPilotWhale = 73,
    LongmansBeakedWhale = 74,
    MarineOtter = 75,
    MediterraneanMonkSeal = 76,
    MelonheadedWhale = 77,
    Narwhal = 78,
    NewZealandFurSeal = 79,
    NewZealandSeaLion = 80,
    NorthAtlanticRightWhale = 81,
    NorthPacificRightWhale = 82,
    NorthernBottlenoseWhale = 83,
    NorthernElephantSeal = 84,
    NorthernFurSeal = 85,
    NorthernRightWhaleDolphin = 86,
    OliveRidleySeaTurtle = 87,
    PacificWhitesidedDolphin = 88,
    PantropicalSpottedDolphin = 89,
    PealesDolphin = 90,
    PolarBear = 91,
    PygmyBeakedWhale = 92,
    PygmyKillerWhale = 93,
    PygmyRightWhale = 94,
    PygmySpermWhale = 95,
    RibbonSeal = 96,
    RicesWhale = 97,
    RingedSeal = 98,
    RissosDolphin = 99,
    RossSeal = 100,
    RoughtoothedDolphin = 101,
    SeaOtter = 102,
    SeiWhale = 103,
    ShepherdsBeakedWhale = 104,
    ShortfinnedPilotWhale = 105,
    SouthAfricanFurSeal = 106,
    SouthAmericanFurSeal = 107,
    SouthAmericanSeaLion = 108,
    SouthAsianRiverDolphin = 109,
    SouthernBottlenoseWhale = 110,
    SouthernElephantSeal = 111,
    SouthernRightWhale = 112,
    SouthernRightWhaleDolphin = 113,
    SowerbysBeakedWhale = 114,
    SpectacledPorpoise = 115,
    SpermWhale = 116,
    SpinnerDolphin = 117,
    SpottedSeal = 118,
    StejnegersBeakedWhale = 119,
    StellerSeaLion = 120,
    StraptoothedWhale = 121,
    StripedDolphin = 122,
    SubantarcticFurSeal = 123,
    TruesBeakedWhale = 124,
    Tucuxi = 125,
    UnidentifiableBaleenWhale = 126,
    UnidentifiableBeakedWhale = 127,
    UnidentifiableCetacean = 128,
    UnidentifiableDolphin = 129,
    UnidentifiableFurSeal = 130,
    UnidentifiableKogiaWhale = 131,
    UnidentifiablePilotWhale = 132,
    UnidentifiablePorpoise = 133,
    UnidentifiableRightWhale = 134,
    UnidentifiableSeaLion = 135,
    UnidentifiableSeal = 136,
    UnidentifiableShelledSeaTurtle = 137,
    UnidentifiableWhale = 138,
    Vaquita = 139,
    Walrus = 140,
    WeddellSeal = 141,
    WestAfricanManatee = 142,
    WestIndianManatee = 143,
    WhaleShark = 144,
    WhitebeakedDolphin = 145,
    YangtzeFinlessPorpoise = 146,
    YangtzeRiverDolphin = 147,
}
impl Species {
    /// String value of the enum field names used in the ProtoBuf definition.
    ///
    /// The values are not transformed in any way and thus are considered stable
    /// (if the ProtoBuf definition does not change) and safe for programmatic use.
    pub fn as_str_name(&self) -> &'static str {
        match self {
            Species::Unknown => "UNKNOWN",
            Species::AmazonianManatee => "AMAZONIAN_MANATEE",
            Species::AndrewsBeakedWhale => "ANDREWS_BEAKED_WHALE",
            Species::AntarcticFurSeal => "ANTARCTIC_FUR_SEAL",
            Species::AntarcticMinkeWhale => "ANTARCTIC_MINKE_WHALE",
            Species::ArnouxsBeakedWhale => "ARNOUXS_BEAKED_WHALE",
            Species::AtlanticHumpbackDolphin => "ATLANTIC_HUMPBACK_DOLPHIN",
            Species::AtlanticSpottedDolphin => "ATLANTIC_SPOTTED_DOLPHIN",
            Species::AtlanticWhitesidedDolphin => "ATLANTIC_WHITESIDED_DOLPHIN",
            Species::AustralianFurSeal => "AUSTRALIAN_FUR_SEAL",
            Species::AustralianSeaLion => "AUSTRALIAN_SEA_LION",
            Species::BahamondesBeakedWhale => "BAHAMONDES_BEAKED_WHALE",
            Species::BaikalSealOrNerpa => "BAIKAL_SEAL_OR_NERPA",
            Species::BairdsBeakedWhale => "BAIRDS_BEAKED_WHALE",
            Species::BeardedSeal => "BEARDED_SEAL",
            Species::BelugaWhale => "BELUGA_WHALE",
            Species::BlainvillesBeakedWhale => "BLAINVILLES_BEAKED_WHALE",
            Species::BlueWhale => "BLUE_WHALE",
            Species::Boto => "BOTO",
            Species::BowheadWhale => "BOWHEAD_WHALE",
            Species::BrydesWhale => "BRYDES_WHALE",
            Species::BurmeistersPorpoise => "BURMEISTERS_PORPOISE",
            Species::CaliforniaSeaLion => "CALIFORNIA_SEA_LION",
            Species::CaspianSeal => "CASPIAN_SEAL",
            Species::ChileanDolphin => "CHILEAN_DOLPHIN",
            Species::ClymeneDolphin => "CLYMENE_DOLPHIN",
            Species::CommersonsDolphin => "COMMERSONS_DOLPHIN",
            Species::CommonBottlenoseDolphin => "COMMON_BOTTLENOSE_DOLPHIN",
            Species::CommonDolphin => "COMMON_DOLPHIN",
            Species::CommonMinkeWhale => "COMMON_MINKE_WHALE",
            Species::CrabeaterSeal => "CRABEATER_SEAL",
            Species::CuviersBeakedWhale => "CUVIERS_BEAKED_WHALE",
            Species::DallsPorpoise => "DALLS_PORPOISE",
            Species::Dugong => "DUGONG",
            Species::DuskyDolphin => "DUSKY_DOLPHIN",
            Species::DwarfSpermWhale => "DWARF_SPERM_WHALE",
            Species::FalseKillerWhale => "FALSE_KILLER_WHALE",
            Species::FinWhale => "FIN_WHALE",
            Species::FinlessPorpoise => "FINLESS_PORPOISE",
            Species::FlatbackSeaTurtle => "FLATBACK_SEA_TURTLE",
            Species::FloridaManatee => "FLORIDA_MANATEE",
            Species::Franciscana => "FRANCISCANA",
            Species::FrasersDolphin => "FRASERS_DOLPHIN",
            Species::GalapagosSeaLion => "GALAPAGOS_SEA_LION",
            Species::GervaisBeakedWhale => "GERVAIS_BEAKED_WHALE",
            Species::GinkgotoothedBeakedWhale => "GINKGOTOOTHED_BEAKED_WHALE",
            Species::GraySeal => "GRAY_SEAL",
            Species::GrayWhale => "GRAY_WHALE",
            Species::GraysBeakedWhale => "GRAYS_BEAKED_WHALE",
            Species::GreenSeaTurtle => "GREEN_SEA_TURTLE",
            Species::GuadalupeFurSeal => "GUADALUPE_FUR_SEAL",
            Species::HarborPorpoise => "HARBOR_PORPOISE",
            Species::HarborSeal => "HARBOR_SEAL",
            Species::HarpSeal => "HARP_SEAL",
            Species::HawaiianMonkSeal => "HAWAIIAN_MONK_SEAL",
            Species::HawksbillSeaTurtle => "HAWKSBILL_SEA_TURTLE",
            Species::HeavisidesDolphin => "HEAVISIDES_DOLPHIN",
            Species::HectorsBeakedWhale => "HECTORS_BEAKED_WHALE",
            Species::HectorsDolphin => "HECTORS_DOLPHIN",
            Species::HoodedSeal => "HOODED_SEAL",
            Species::HourglassDolphin => "HOURGLASS_DOLPHIN",
            Species::HubbsBeakedWhale => "HUBBS_BEAKED_WHALE",
            Species::HumpbackWhale => "HUMPBACK_WHALE",
            Species::IndopacificBottlenoseDolphin => "INDOPACIFIC_BOTTLENOSE_DOLPHIN",
            Species::IndopacificHumpbackDolphin => "INDOPACIFIC_HUMPBACK_DOLPHIN",
            Species::IrrawaddyDolphin => "IRRAWADDY_DOLPHIN",
            Species::JuanFernandezFurSeal => "JUAN_FERNANDEZ_FUR_SEAL",
            Species::KempsRidleySeaTurtle => "KEMPS_RIDLEY_SEA_TURTLE",
            Species::KillerWhale => "KILLER_WHALE",
            Species::LeatherbackSeaTurtle => "LEATHERBACK_SEA_TURTLE",
            Species::LeopardSeal => "LEOPARD_SEAL",
            Species::LoggerheadSeaTurtle => "LOGGERHEAD_SEA_TURTLE",
            Species::LongbeakedCommonDolphin => "LONGBEAKED_COMMON_DOLPHIN",
            Species::LongfinnedPilotWhale => "LONGFINNED_PILOT_WHALE",
            Species::LongmansBeakedWhale => "LONGMANS_BEAKED_WHALE",
            Species::MarineOtter => "MARINE_OTTER",
            Species::MediterraneanMonkSeal => "MEDITERRANEAN_MONK_SEAL",
            Species::MelonheadedWhale => "MELONHEADED_WHALE",
            Species::Narwhal => "NARWHAL",
            Species::NewZealandFurSeal => "NEW_ZEALAND_FUR_SEAL",
            Species::NewZealandSeaLion => "NEW_ZEALAND_SEA_LION",
            Species::NorthAtlanticRightWhale => "NORTH_ATLANTIC_RIGHT_WHALE",
            Species::NorthPacificRightWhale => "NORTH_PACIFIC_RIGHT_WHALE",
            Species::NorthernBottlenoseWhale => "NORTHERN_BOTTLENOSE_WHALE",
            Species::NorthernElephantSeal => "NORTHERN_ELEPHANT_SEAL",
            Species::NorthernFurSeal => "NORTHERN_FUR_SEAL",
            Species::NorthernRightWhaleDolphin => "NORTHERN_RIGHT_WHALE_DOLPHIN",
            Species::OliveRidleySeaTurtle => "OLIVE_RIDLEY_SEA_TURTLE",
            Species::PacificWhitesidedDolphin => "PACIFIC_WHITESIDED_DOLPHIN",
            Species::PantropicalSpottedDolphin => "PANTROPICAL_SPOTTED_DOLPHIN",
            Species::PealesDolphin => "PEALES_DOLPHIN",
            Species::PolarBear => "POLAR_BEAR",
            Species::PygmyBeakedWhale => "PYGMY_BEAKED_WHALE",
            Species::PygmyKillerWhale => "PYGMY_KILLER_WHALE",
            Species::PygmyRightWhale => "PYGMY_RIGHT_WHALE",
            Species::PygmySpermWhale => "PYGMY_SPERM_WHALE",
            Species::RibbonSeal => "RIBBON_SEAL",
            Species::RicesWhale => "RICES_WHALE",
            Species::RingedSeal => "RINGED_SEAL",
            Species::RissosDolphin => "RISSOS_DOLPHIN",
            Species::RossSeal => "ROSS_SEAL",
            Species::RoughtoothedDolphin => "ROUGHTOOTHED_DOLPHIN",
            Species::SeaOtter => "SEA_OTTER",
            Species::SeiWhale => "SEI_WHALE",
            Species::ShepherdsBeakedWhale => "SHEPHERDS_BEAKED_WHALE",
            Species::ShortfinnedPilotWhale => "SHORTFINNED_PILOT_WHALE",
            Species::SouthAfricanFurSeal => "SOUTH_AFRICAN_FUR_SEAL",
            Species::SouthAmericanFurSeal => "SOUTH_AMERICAN_FUR_SEAL",
            Species::SouthAmericanSeaLion => "SOUTH_AMERICAN_SEA_LION",
            Species::SouthAsianRiverDolphin => "SOUTH_ASIAN_RIVER_DOLPHIN",
            Species::SouthernBottlenoseWhale => "SOUTHERN_BOTTLENOSE_WHALE",
            Species::SouthernElephantSeal => "SOUTHERN_ELEPHANT_SEAL",
            Species::SouthernRightWhale => "SOUTHERN_RIGHT_WHALE",
            Species::SouthernRightWhaleDolphin => "SOUTHERN_RIGHT_WHALE_DOLPHIN",
            Species::SowerbysBeakedWhale => "SOWERBYS_BEAKED_WHALE",
            Species::SpectacledPorpoise => "SPECTACLED_PORPOISE",
            Species::SpermWhale => "SPERM_WHALE",
            Species::SpinnerDolphin => "SPINNER_DOLPHIN",
            Species::SpottedSeal => "SPOTTED_SEAL",
            Species::StejnegersBeakedWhale => "STEJNEGERS_BEAKED_WHALE",
            Species::StellerSeaLion => "STELLER_SEA_LION",
            Species::StraptoothedWhale => "STRAPTOOTHED_WHALE",
            Species::StripedDolphin => "STRIPED_DOLPHIN",
            Species::SubantarcticFurSeal => "SUBANTARCTIC_FUR_SEAL",
            Species::TruesBeakedWhale => "TRUES_BEAKED_WHALE",
            Species::Tucuxi => "TUCUXI",
            Species::UnidentifiableBaleenWhale => "UNIDENTIFIABLE_BALEEN_WHALE",
            Species::UnidentifiableBeakedWhale => "UNIDENTIFIABLE_BEAKED_WHALE",
            Species::UnidentifiableCetacean => "UNIDENTIFIABLE_CETACEAN",
            Species::UnidentifiableDolphin => "UNIDENTIFIABLE_DOLPHIN",
            Species::UnidentifiableFurSeal => "UNIDENTIFIABLE_FUR_SEAL",
            Species::UnidentifiableKogiaWhale => "UNIDENTIFIABLE_KOGIA_WHALE",
            Species::UnidentifiablePilotWhale => "UNIDENTIFIABLE_PILOT_WHALE",
            Species::UnidentifiablePorpoise => "UNIDENTIFIABLE_PORPOISE",
            Species::UnidentifiableRightWhale => "UNIDENTIFIABLE_RIGHT_WHALE",
            Species::UnidentifiableSeaLion => "UNIDENTIFIABLE_SEA_LION",
            Species::UnidentifiableSeal => "UNIDENTIFIABLE_SEAL",
            Species::UnidentifiableShelledSeaTurtle => "UNIDENTIFIABLE_SHELLED_SEA_TURTLE",
            Species::UnidentifiableWhale => "UNIDENTIFIABLE_WHALE",
            Species::Vaquita => "VAQUITA",
            Species::Walrus => "WALRUS",
            Species::WeddellSeal => "WEDDELL_SEAL",
            Species::WestAfricanManatee => "WEST_AFRICAN_MANATEE",
            Species::WestIndianManatee => "WEST_INDIAN_MANATEE",
            Species::WhaleShark => "WHALE_SHARK",
            Species::WhitebeakedDolphin => "WHITEBEAKED_DOLPHIN",
            Species::YangtzeFinlessPorpoise => "YANGTZE_FINLESS_PORPOISE",
            Species::YangtzeRiverDolphin => "YANGTZE_RIVER_DOLPHIN",
        }
    }
}
/// A station, that can be on a vehicle (or not), and contains 1 or more
/// station ids.
#[derive(serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
#[derive(Eq, PartialOrd, Ord, Hash, Clone, PartialEq, ::prost::Message)]
pub struct Station {
    /// The optional vehicle name
    #[prost(string, optional, tag = "1")]
    pub vehicle: ::core::option::Option<::prost::alloc::string::String>,
    /// The list of station ids. Zero station ids is considered an illegal state.
    #[prost(string, repeated, tag = "2")]
    pub station_ids: ::prost::alloc::vec::Vec<::prost::alloc::string::String>,
}
