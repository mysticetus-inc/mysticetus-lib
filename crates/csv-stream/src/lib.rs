#![feature(int_from_ascii, array_windows)]
mod error;
mod row;
mod stream;
pub use error::Error;
mod buffer;
mod reader;
pub use row::Row;

#[cfg(test)]
mod test_utils;

pub type StdError = Box<dyn std::error::Error + Send + Sync>;

pub use stream::CsvStream;

#[cfg(test)]
mod tests {
    use futures::StreamExt;

    use crate::test_utils::HorribleStream;

    #[tokio::test]
    async fn test_csv() -> Result<(), crate::Error<std::convert::Infallible>> {
        const FILE: &str = "...";

        let stream = HorribleStream::from_file(FILE).await.unwrap();
        let mut stream = std::pin::pin!(crate::CsvStream::new(stream));

        while let Some(result) = stream.next().await {
            let row = result?;
            println!("{row:#?}");
        }

        Ok(())
    }
}
