pub(crate) fn append_to_path<I>(url: &reqwest::Url, parts: I) -> reqwest::Url
where
    I: IntoIterator,
    I::Item: AsRef<str>,
{
    let mut dst_url = url.clone();
    dst_url
        .path_segments_mut()
        .expect("can be a base")
        .extend(parts);
    dst_url
}

// used for `#[serde(skip_serializing_if = "is_false")]` attrs
#[inline]
pub(crate) fn is_false(b: &bool) -> bool {
    !*b
}
