//! Async tabix index and fields.

mod reader;
mod writer;

pub use self::{reader::Reader, writer::Writer};

use std::path::Path;

use tokio::{fs::File, io};

use super::Index;

/// Reads the entire contents of a tabix index.
///
/// This is a convenience function and is equivalent to opening the file at the given path and
/// reading the index.
///
/// # Examples
///
/// ```no_run
/// # use std::io;
/// #
/// # #[tokio::main]
/// # async fn main() -> io::Result<()> {
/// use noodles_tabix as tabix;
/// let index = tabix::r#async::read("sample.vcf.gz.tbi").await?;
/// # Ok(())
/// # }
/// ```
pub async fn read<P>(src: P) -> io::Result<Index>
where
    P: AsRef<Path>,
{
    let mut reader = File::open(src).await.map(Reader::new)?;
    reader.read_index().await
}
