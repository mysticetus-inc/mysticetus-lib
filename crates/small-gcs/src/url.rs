use percent_encoding::AsciiSet;

const BASE_URL: &str = "https://storage.googleapis.com/storage/v1/b/";
const BASE_UPLOAD_URL: &str = "https://storage.googleapis.com/upload/storage/v1/b/";

const REWRITE_SEP: &str = "/rewriteTo/b/";

static ENCODE_SET: &AsciiSet = &percent_encoding::NON_ALPHANUMERIC
    .remove(b'*')
    .remove(b'-')
    .remove(b'.')
    .remove(b'_');

pub struct Encode<'a>(pub &'a str);

impl serde::Serialize for Encode<'_> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.collect_str(&percent_encoding::utf8_percent_encode(self.0, ENCODE_SET))
    }
}

pub fn percent_encode_into(src: &str, dst: &mut String) {
    for chunk in percent_encoding::utf8_percent_encode(src, ENCODE_SET) {
        dst.push_str(chunk);
    }
}

#[derive(Debug, Clone, Copy)]
pub struct UrlBuilder<'a, const IS_UPLOAD: bool> {
    bucket: &'a str,
    name: Option<&'a str>,
    rewrite_dst: Option<RewriteDst<'a>>,
}

#[derive(Debug, Clone, Copy)]
struct RewriteDst<'a> {
    bucket: &'a str,
    name: &'a str,
}

impl<'a> UrlBuilder<'a, false> {
    pub fn new(bucket: &'a str) -> Self {
        Self {
            bucket,
            name: None,
            rewrite_dst: None,
        }
    }

    #[inline]
    pub const fn upload(self) -> UrlBuilder<'a, true> {
        UrlBuilder {
            bucket: self.bucket,
            name: self.name,
            rewrite_dst: None,
        }
    }
}

impl<'a, const IS_UPLOAD: bool> UrlBuilder<'a, IS_UPLOAD> {
    pub fn name(mut self, name: &'a str) -> Self {
        self.name = Some(name);
        self
    }

    fn len_needed(&self) -> usize {
        let base_len = if IS_UPLOAD {
            BASE_UPLOAD_URL.len()
        } else {
            BASE_URL.len()
        };

        let rewrite_len = match self.rewrite_dst {
            None => 0,
            Some(rewrite_dst) => {
                // trailing 3 is from the  '/o/' separator between the target bucket + target name
                REWRITE_SEP.len() + rewrite_dst.bucket.len() + rewrite_dst.name.len() + 3
            }
        };

        base_len
            + self.bucket.len()
            + 3 // '/o/' path sep
            + self.name.map(str::len).unwrap_or(0)
            + rewrite_len
    }

    // inner function that assumes 'dst' is already cleared and has reserved
    // enough capacity to fit the string without resizing.
    fn format_into_inner(&mut self, dst: &mut String) {
        assert!(dst.is_empty());

        if IS_UPLOAD {
            dst.push_str(BASE_UPLOAD_URL);
        } else {
            dst.push_str(BASE_URL);
        }

        dst.push_str(self.bucket);
        dst.push_str("/o");

        if let Some(name) = self.name {
            dst.push_str("/");
            percent_encode_into(name, dst);
        }

        if let Some(RewriteDst { bucket, name }) = self.rewrite_dst {
            assert!(self.name.is_some(), "cant rewrite without a source object");

            dst.push_str(REWRITE_SEP);
            dst.push_str(bucket);
            dst.push_str("/o/");
            dst.push_str(name);
        }
    }

    pub fn format_into(&mut self, dst: &mut String) {
        dst.clear();
        dst.reserve(self.len_needed());

        self.format_into_inner(dst);
    }

    pub fn format(&mut self) -> String {
        let capacity = self.len_needed();
        let mut dst = String::with_capacity(capacity);
        self.format_into_inner(&mut dst);

        debug_assert_eq!(dst.len(), capacity);

        dst
    }
}

impl<'a> UrlBuilder<'a, false> {
    pub fn rewrite(mut self, bucket: &'a str, name: &'a str) -> Self {
        self.rewrite_dst = Some(RewriteDst { bucket, name });
        self
    }
}
