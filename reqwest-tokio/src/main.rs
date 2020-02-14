// Downloads a picture of Ferris, the Rust mascot, from the Internet.
// Demonstrates basic use of reqwest for async http(s) requests and downloading.

// Needed to be able to call flush() on a tokio::io::AsyncWrite.
use tokio::io::AsyncWriteExt;

// tokio::main macro automatically sets up the tokio runtime.
#[tokio::main]
async fn main() -> Result<(), util::BoxError> {
    // Attempt to download ferris..
    let mut download = reqwest::get("https://rustacean.net/assets/rustacean-orig-noshadow.png")
        .await? // await server response
        .error_for_status()?; // generate an error if server didn't respond OK
    
    // Create an output file into which we will save ferris.
    let mut outfile = tokio::fs::File::create("ferris.png").await?;
    
    // Do an asynchronous, buffered copy of the download to the output file.
    // 
    // Note that in some sense this is a workaround for being unable to use
    // tokio::io::copy as in the reqwest-tokio-compat example, but on the other
    // hand this method has no performance penalty and can actually be
    // preferable in some cases because it gives us more control.
    while let Some(chunk) = download.chunk().await? {
        outfile.write(&chunk).await?;
    }
    
    // Must flush tokio::io::BufWriter manually.
    // It will *not* flush itself automatically when dropped.
    outfile.flush().await?;
    
    Ok(())
}
