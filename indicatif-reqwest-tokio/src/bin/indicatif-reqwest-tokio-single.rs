// Downloads a 10MB video example file from https://file-examples.com.
// Demonstrates basic use of reqwest for async http(s) requests and showing an indicatif status bar for the download.

// Needed to be able to call flush() on a tokio::io::AsyncWrite.
use indicatif::{ProgressBar, ProgressStyle};
use reqwest::{Client, Url, header};
use tokio::io::AsyncWriteExt;

// tokio::main macro automatically sets up the tokio runtime.
#[tokio::main]
async fn main() -> Result<(), util::BoxError> {
    // Set the URL of the file to download, we us a 10MB example video here
    let download_url_str = "https://file-examples-com.github.io/uploads/2017/04/file_example_MP4_1280_10MG.mp4";
    
    // Parse URL into Url type
    let url = Url::parse(download_url_str)?;

    // Create a reqwest Client
    let client = Client::new();

    // We need to determine the file size before we download, so we can create a ProgressBar
    // A Header request for the CONTENT_LENGTH header gets us the file size
    let download_size = {
        let resp = client.head(url.as_str()).send().await?;
        if resp.status().is_success() {
            resp.headers() // Gives is the HeaderMap
                .get(header::CONTENT_LENGTH) // Gives us an Option containin the HeaderValue
                .and_then(|ct_len| ct_len.to_str().ok()) // Unwraps the Option as &str
                .and_then(|ct_len| ct_len.parse().ok()) // Parses the Option as u64
                .unwrap_or(0) // Fallback to 0
        } else {
            // We return an Error if something goes wrong here
            return Err(format!(
                "Couldn't download URL: {}. Error: {:?}",
                url,
                resp.status(),
            )
            .into());
        }
    };

    // Parse the filename from the given URL
    let filename = url
           .path_segments() // Splits into segments of the URL
           .and_then(|segments| segments.last()) // Retrieves the last segment
           .unwrap_or("video.mp4"); // Fallback to generic filename

    // Here we build the actual Request with a RequestBuilder from the Client
    let request = client.get(url.as_str());

    // Create the ProgressBar with the aquired size from before
    let progress_bar = ProgressBar::new(download_size);

    // Set Style to the ProgressBar
    progress_bar.set_style(
        ProgressStyle::default_bar()
            .template("[{bar:40.cyan/blue}] {bytes}/{total_bytes} {msg}")
            .progress_chars("#>-"),
    );

    // Set the filename as message part of the progress bar
    progress_bar.set_message(&filename);

    // Create the output file
    let mut outfile = tokio::fs::File::create(&filename).await?;

    // Do the actual request to download the file
    let mut download = request.send().await?;

    // Do an asynchronous, buffered copy of the download to the output file.
    // 
    // We use the part from the reqwest-tokio example here on purpose
    // This way, we are able to increase the ProgressBar with every downloaded chunk
    while let Some(chunk) = download.chunk().await? {
        progress_bar.inc(chunk.len() as u64);   // Increase ProgressBar by chunk size
        outfile.write(&chunk).await?;   // Write chunk to output file
    }

    // Finish the progress bar to prevent glitches
    progress_bar.finish();
    
    // Must flush tokio::io::BufWriter manually.
    // It will *not* flush itself automatically when dropped.
    outfile.flush().await?;
    
    Ok(())
}