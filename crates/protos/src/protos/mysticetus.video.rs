/// A normalized point.
///
/// ## Invariants:
/// - '0.0 <= x <= 1.0'
/// - '0.0 <= y <= 1.0'
///
/// Values outside of this range will be clamped to '[0, 1]'.
#[derive(serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
#[derive(Copy, PartialOrd, Clone, PartialEq, ::prost::Message)]
pub struct Point {
    #[prost(float, required, tag = "1")]
    pub x: f32,
    #[prost(float, required, tag = "2")]
    pub y: f32,
}
/// A normalized size.
///
/// ## Invariants:
/// - '0 <= width <= 1'
/// - '0 <= height <= 1'
#[derive(serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
#[derive(Copy, PartialOrd, Clone, PartialEq, ::prost::Message)]
pub struct Size {
    #[prost(float, required, tag = "1")]
    pub width: f32,
    #[prost(float, required, tag = "2")]
    pub height: f32,
}
/// The dimensions of a video, in pixels.
///
/// ## Invariants:
/// - '0 <= width <= video_size.width'
/// - '0 <= height <= video_size.height'
#[derive(serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
#[derive(Copy, Eq, PartialOrd, Ord, Hash, Clone, PartialEq, ::prost::Message)]
pub struct VideoSize {
    #[prost(uint32, required, tag = "1")]
    pub width: u32,
    #[prost(uint32, required, tag = "2")]
    pub height: u32,
}
/// A normalized bezier curve.
///
/// By convention, 'start' should be to the left of 'end'
/// (i.e 'start.x' < 'end.x').
///
/// Control points should be ordered so that 'control_points\[n\].x' increases
/// as 'n' increases.
#[derive(serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
#[derive(PartialOrd, Clone, PartialEq, ::prost::Message)]
pub struct BezierCurve {
    /// The start point.
    #[prost(message, required, tag = "1")]
    pub start: Point,
    /// The end points.
    #[prost(message, required, tag = "2")]
    pub end: Point,
    /// The control points that define the curve. An empty set of control points
    /// is just a straight line from 'start' to 'end'.
    #[prost(message, repeated, tag = "3")]
    #[serde(default)]
    pub control_points: ::prost::alloc::vec::Vec<Point>,
}
/// A normalized bounding box.
///
/// ## Reference points
/// - '{ x, y }':
/// Represents the upper left corner.
///
/// - '{ x + width, y + height }':
/// Represents the lower right corner.
///
/// ## Invariants:
/// - '0.0 < x < 1.0'
/// - '0.0 < y < 1.0'
/// - '0.0 < x + width < 1.0'
/// - '0.0 < y + height < 1.0'
/// - 'x < x + width'
/// - 'y < y + height'
#[derive(serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
#[derive(Copy, PartialOrd, Clone, PartialEq, ::prost::Message)]
pub struct BoundingBox {
    /// The normalized top left corner.
    #[prost(message, required, tag = "1")]
    pub point: Point,
    /// The normalized size of the bounding box.
    #[prost(message, required, tag = "2")]
    pub size: Size,
}
/// A single annotation on a frame.
#[derive(serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Annotation {
    /// The unique Id for this annotation. Should be a uuid/other highly random
    /// string.
    #[prost(string, required, tag = "1")]
    pub id: ::prost::alloc::string::String,
    /// The kind of annotation
    #[prost(oneof = "annotation::Kind", tags = "2, 3, 4")]
    #[serde(flatten)]
    pub kind: ::core::option::Option<annotation::Kind>,
}
/// Nested message and enum types in `Annotation`.
pub mod annotation {
    /// The kind of annotation
    #[derive(serde::Deserialize, serde::Serialize)]
    #[serde(rename_all = "camelCase")]
    #[serde(tag = "kind")]
    #[derive(Clone, PartialEq, ::prost::Oneof)]
    pub enum Kind {
        /// An animal detection annotation
        #[prost(message, tag = "2")]
        AnimalDetection(super::AnimalDetection),
        /// A horizon annotation
        #[prost(message, tag = "3")]
        Horizon(super::BezierCurve),
        /// A vessel Annotation
        #[prost(message, tag = "4")]
        Vessel(super::Vessel),
    }
}
/// An annotation describing an animal detection.
#[derive(serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct AnimalDetection {
    /// The bounding box containing this sighting/detection.
    #[prost(message, required, tag = "1")]
    pub bounding_box: BoundingBox,
    /// The detection type/label.
    #[prost(enumeration = "DetectionType", required, tag = "2")]
    pub detection_type: i32,
    /// The optional Mysticetus `_rowId` that describes the initial sighting of
    /// this animal.
    ///
    /// Isn't currently populated, reserving the field now that way it's easy to
    /// start adding when we have UI to associate videos with mysticetus data.
    ///
    /// Even when this is populated, this still might be missing if the
    /// PSO's didn't catch/enter the detection at the time.
    #[prost(string, optional, tag = "3")]
    pub initial_row_id: ::core::option::Option<::prost::alloc::string::String>,
    /// The species of animal detected.
    #[prost(oneof = "animal_detection::Species", tags = "4, 5, 6")]
    pub species: ::core::option::Option<animal_detection::Species>,
}
/// Nested message and enum types in `AnimalDetection`.
pub mod animal_detection {
    /// The species of animal detected.
    #[derive(serde::Deserialize, serde::Serialize)]
    #[serde(rename_all = "camelCase")]
    #[derive(Clone, PartialEq, ::prost::Oneof)]
    pub enum Species {
        /// A well known species from the defined mysticetus list.
        #[prost(enumeration = "super::super::common::Species", tag = "4")]
        WellKnown(i32),
        /// If the well known species list did not contain the desired species,
        /// this provides a fallback that can then be adjusted later on manually.
        #[prost(string, tag = "5")]
        OtherSpecies(::prost::alloc::string::String),
        /// A bird sighting. If this variant is set, it's invalid for
        /// 'detection_type' to be anything __other__ than `DetectionType.BODY`.
        #[prost(enumeration = "super::BirdSpecies", tag = "6")]
        Bird(i32),
    }
}
/// A detected Vessel.
#[derive(serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Vessel {
    /// The bounding box around the detected vessel.
    #[prost(message, required, tag = "1")]
    pub bounding_box: BoundingBox,
}
/// The type of detection.
#[derive(serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
#[repr(i32)]
pub enum DetectionType {
    /// An unknown/unspecified detection type.
    Unspecified = 0,
    /// A blow from an animal.
    Blow = 1,
    /// A visible sighting of the animal itself. If the detection is
    /// representing a bird, this is the only valid value.
    Body = 2,
    /// A splash from an animal.
    Splash = 3,
    Footprint = 4,
}
impl DetectionType {
    /// String value of the enum field names used in the ProtoBuf definition.
    ///
    /// The values are not transformed in any way and thus are considered stable
    /// (if the ProtoBuf definition does not change) and safe for programmatic use.
    pub fn as_str_name(&self) -> &'static str {
        match self {
            DetectionType::Unspecified => "UNSPECIFIED",
            DetectionType::Blow => "BLOW",
            DetectionType::Body => "BODY",
            DetectionType::Splash => "SPLASH",
            DetectionType::Footprint => "FOOTPRINT",
        }
    }
}
#[derive(serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
#[repr(i32)]
pub enum BirdSpecies {
    Unknown = 0,
}
impl BirdSpecies {
    /// String value of the enum field names used in the ProtoBuf definition.
    ///
    /// The values are not transformed in any way and thus are considered stable
    /// (if the ProtoBuf definition does not change) and safe for programmatic use.
    pub fn as_str_name(&self) -> &'static str {
        match self {
            BirdSpecies::Unknown => "UNKNOWN",
        }
    }
}
/// Annotations + info on a single frame.
#[derive(serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct FrameInfo {
    /// The frame slug/ID, generated from the GCS md5 hash.
    #[prost(string, required, tag = "1")]
    pub frame_slug: ::prost::alloc::string::String,
    /// The fully qualified GCS path to the image file for this frame.
    #[prost(string, required, tag = "2")]
    pub frame_path: ::prost::alloc::string::String,
    /// The slug representing the video that this frame belongs to.
    #[prost(string, required, tag = "3")]
    pub video_slug: ::prost::alloc::string::String,
    /// A list of unique UID's from any labeler who added annotations.
    #[prost(string, repeated, tag = "4")]
    #[serde(default)]
    pub annotated_by: ::prost::alloc::vec::Vec<::prost::alloc::string::String>,
    /// The annotations applicable to this frame. The key in the map should be
    /// exactly equal to the 'Annotation.id' field on the corresponding value.
    #[prost(map = "string, message", tag = "5")]
    pub annotations: ::std::collections::HashMap<::prost::alloc::string::String, Annotation>,
    /// The offset from the start of the video that this frame is from.
    #[prost(message, required, tag = "6")]
    #[serde(with = "crate::impls::duration")]
    pub offset: super::super::google::protobuf::Duration,
    /// If the video start timestamp is known, this is computed
    /// by adding 'offset', giving us the absolute timestamp of this frame.
    #[prost(message, optional, tag = "7")]
    #[serde(with = "crate::impls::timestamp::opt")]
    pub timestamp: ::core::option::Option<super::super::google::protobuf::Timestamp>,
}
/// Info about a single video throughout the labeling process.
/// Most of these fields are computed serverside when the document is generated,
/// and otherwise managed serverside.
///
/// The only time a downstream client should update fields is when
/// appending 'detection_windows'.
#[derive(serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct VideoInfo {
    /// The fully qualified GCS path to the video file.
    ///
    /// First component of the path must be the bucket name.
    #[prost(string, required, tag = "1")]
    pub video_path: ::prost::alloc::string::String,
    /// The fully qualified GCS path to the directory containing the
    /// extracted frames. Missing if no timestamp ranges have been marked yet.
    #[prost(string, optional, tag = "2")]
    pub frames_path: ::core::option::Option<::prost::alloc::string::String>,
    /// The Mysticetus github repo that may have data associated with this video.
    ///
    /// The old 'projectId' field is being phased out in favor of this.
    #[prost(string, optional, tag = "3")]
    pub repo_name: ::core::option::Option<::prost::alloc::string::String>,
    /// The optional Station ID within the Mysticetus data repo. Used to narrow
    /// down the specific files to pull data from.
    #[prost(string, optional, tag = "4")]
    pub station_id: ::core::option::Option<::prost::alloc::string::String>,
    /// The video slug. Computed serverside from the md5 hash associated with the
    /// GCS blob object. If the slug itself ends in '-test', the 'test'
    /// property can be assumed, even if not set.
    #[prost(string, required, tag = "5")]
    pub slug: ::prost::alloc::string::String,
    /// The type of camera that was used to record this video.
    #[prost(enumeration = "CameraType", required, tag = "6")]
    pub camera_type: i32,
    /// The status of where the video is at in the labeling process.
    #[prost(enumeration = "VideoLabelingStatus", required, tag = "7")]
    pub status: i32,
    /// The size of the video, in pixels.
    #[prost(message, required, tag = "8")]
    pub video_size: VideoSize,
    /// The duration of the video.
    #[prost(message, required, tag = "9")]
    #[serde(with = "crate::impls::duration")]
    pub video_duration: super::super::google::protobuf::Duration,
    /// The absolute timestamp that corresponds to time the video starts.
    #[prost(message, optional, tag = "10")]
    #[serde(with = "crate::impls::timestamp::opt")]
    pub video_start: ::core::option::Option<super::super::google::protobuf::Timestamp>,
    /// The windows of time in the video where detections were seen.
    ///
    /// This field is normalized serverside, so an update will come back once
    /// this occurs if a listener is being used.
    ///
    /// The normalization involves the following steps, in order:
    ///   - Modifies each window to have an overall minimum duration of X (TBD) seconds, adding
    ///     time evenly to both start and end. If adding evenly results in the window starting
    ///     before 00:00:00 or after 'video_duration' the window is shifted until it's within
    ///     range. If the video duration itself is shorter than our minimum window duration, we
    ///     just use the entire video.
    ///   - Sorted by the window start offset.
    ///   - Merges windows with overlapping ranges.
    #[prost(message, repeated, tag = "11")]
    #[serde(default)]
    pub detection_windows: ::prost::alloc::vec::Vec<VideoWindow>,
    /// Information about who did what in the steps of labeling this video.
    #[prost(message, required, tag = "12")]
    pub labeler_info: LabelerInfo,
    /// An optional test flag, indicating this video info represents admin only
    /// test data, while still pointing at a production video.
    #[prost(bool, optional, tag = "15")]
    pub test: ::core::option::Option<bool>,
}
/// Info about who did what steps in labeling a video and its frames.
#[derive(serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
#[derive(Eq, PartialOrd, Hash, Clone, PartialEq, ::prost::Message)]
pub struct LabelerInfo {
    /// The UID of all users that labeled frames for this video.
    #[prost(string, repeated, tag = "1")]
    #[serde(default)]
    pub labeled_frames: ::prost::alloc::vec::Vec<::prost::alloc::string::String>,
    /// The UID of users that rejected this video as having no detections.
    #[prost(string, repeated, tag = "2")]
    #[serde(default)]
    pub rejected_video: ::prost::alloc::vec::Vec<::prost::alloc::string::String>,
    /// The UID of all users that marked detection windows within the video.
    #[prost(string, repeated, tag = "3")]
    #[serde(default)]
    pub marked_windows: ::prost::alloc::vec::Vec<::prost::alloc::string::String>,
}
/// A window of time in a video.
///
/// ## Invariants:
/// '00:00:00 <= start < end'
#[derive(serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
#[derive(Copy, Eq, PartialOrd, Ord, Hash, Clone, PartialEq, ::prost::Message)]
pub struct VideoWindow {
    /// The start of the timestamp range, relative to the start of the video.
    #[prost(message, required, tag = "1")]
    #[serde(with = "crate::impls::duration")]
    pub start: super::super::google::protobuf::Duration,
    /// The end of the timestamp range, relative to the start of the video.
    #[prost(message, required, tag = "2")]
    #[serde(with = "crate::impls::duration")]
    pub end: super::super::google::protobuf::Duration,
}
/// The camera type.
#[derive(serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
#[repr(i32)]
pub enum CameraType {
    /// A camera type that hasn't been marked yet.
    None = 0,
    /// Infrared/black and white video.
    Infrared = 1,
    /// Normal color video.
    Visible = 2,
}
impl CameraType {
    /// String value of the enum field names used in the ProtoBuf definition.
    ///
    /// The values are not transformed in any way and thus are considered stable
    /// (if the ProtoBuf definition does not change) and safe for programmatic use.
    pub fn as_str_name(&self) -> &'static str {
        match self {
            CameraType::None => "NONE",
            CameraType::Infrared => "INFRARED",
            CameraType::Visible => "VISIBLE",
        }
    }
}
/// Where a video is at in the process of being tagged.
#[derive(serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
#[repr(i32)]
pub enum VideoLabelingStatus {
    /// A fully untagged video.
    Untagged = 0,
    /// A video with the detection windows marked.
    DetectionWindowsMarked = 1,
    /// A video with the detection windows marked, after the frames from said
    /// windows are generated.
    FramesGenerated = 2,
    /// A video with all of it's frames tagged/annotated.
    Complete = 3,
    /// A video with no usable data.
    Rejected = 4,
}
impl VideoLabelingStatus {
    /// String value of the enum field names used in the ProtoBuf definition.
    ///
    /// The values are not transformed in any way and thus are considered stable
    /// (if the ProtoBuf definition does not change) and safe for programmatic use.
    pub fn as_str_name(&self) -> &'static str {
        match self {
            VideoLabelingStatus::Untagged => "UNTAGGED",
            VideoLabelingStatus::DetectionWindowsMarked => "DETECTION_WINDOWS_MARKED",
            VideoLabelingStatus::FramesGenerated => "FRAMES_GENERATED",
            VideoLabelingStatus::Complete => "COMPLETE",
            VideoLabelingStatus::Rejected => "REJECTED",
        }
    }
}
