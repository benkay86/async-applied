// Downloads a picture of Ferris, the Rust mascot, from the Internet.
// Demonstrates basic use of reqwest for async http(s) requests and downloading.
// Demonstrates the futures <--> tokio compatibility layer for AsyncRead.

// Compatibility trait lets us call `compat()` on a futures::io::AsyncRead
// to convert it into a tokio::io::AsyncRead.
use tokio_util::compat::FuturesAsyncReadCompatExt;

// Lets us call into_async_read() to convert a futures::stream::Stream into a
// futures::io::AsyncRead.
use futures::stream::TryStreamExt;

// tokio::main macro automatically sets up the tokio runtime.
#[tokio::main]
async fn main() -> Result<(), util::BoxError> {
    // Attempt to download ferris..
    let download = reqwest::get("https://rustacean.net/assets/rustacean-orig-noshadow.png")
        .await? // await server response
        .error_for_status()?; // generate an error if server didn't respond OK
    
    // Convert the body of the response into a futures::io::Stream.
    let download = download.bytes_stream();
    
    // Convert the stream into an futures::io::AsyncRead.
    // We must first convert the reqwest::Error into an futures::io::Error.
    let download = download
        .map_err(|e| futures::io::Error::new(futures::io::ErrorKind::Other, e))
        .into_async_read();
    
    // Convert the futures::io::AsyncRead into a tokio::io::AsyncRead.
    let mut download = download.compat();
    
    // Create an output file into which we will save ferris.
    let mut outfile = tokio::fs::File::create("ferris.png").await?;
    
    // Invoke tokio::io::copy to actually perform the download.
    tokio::io::copy(&mut download, &mut outfile).await?;
    
    Ok(())
}
