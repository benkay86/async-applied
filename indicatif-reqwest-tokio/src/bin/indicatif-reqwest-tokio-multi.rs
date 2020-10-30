// Downloads multiple files from https://file-examples.com.
// Demonstrates basic use of reqwest for async http(s) requests and showing an indicatif status bar for the downloads.
// This is a combination of reqwest-tokio and indicatif-tokio

use std::sync::Arc;

use futures::{stream, StreamExt};
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use reqwest::{header, Client, Url};
use tokio::{io::AsyncWriteExt};

async fn download_task(
    download_url: &str,
    multi_bar: Arc<MultiProgress>,
) -> Result<(), util::BoxError> {
    // Parse URL into Url type
    let url = Url::parse(download_url)?;

    // Create a reqwest Client
    let client = Client::new();

    // We need to determine the file size before we download, so we can create a ProgressBar
    // A Header request for the CONTENT_LENGTH header gets us the file size
    let download_size = {
        let resp = client.head(url.as_str()).send().await?;
        if resp.status().is_success() {
            resp.headers() // Gives is the HeaderMap
                .get(header::CONTENT_LENGTH) // Gives us an Option containing the HeaderValue
                .and_then(|ct_len| ct_len.to_str().ok()) // Unwraps the Option as &str
                .and_then(|ct_len| ct_len.parse().ok()) // Parses the Option as u64
                .unwrap_or(0) // Fallback to 0
        } else {
            // We return an Error if something goes wrong here
            return Err(
                format!("Couldn't download URL: {}. Error: {:?}", url, resp.status(),).into(),
            );
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
    let progress_bar = multi_bar.add(ProgressBar::new(download_size));

    // Set Style to the ProgressBar
    progress_bar.set_style(
        ProgressStyle::default_bar()
            .template("[{bar:40.cyan/blue}] {bytes}/{total_bytes} - {msg}")
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
        progress_bar.inc(chunk.len() as u64); // Increase ProgressBar by chunk size
        outfile.write(&chunk).await?; // Write chunk to output file
    }

    // Finish the progress bar to prevent glitches
    progress_bar.finish();

    // Must flush tokio::io::BufWriter manually.
    // It will *not* flush itself automatically when dropped.
    outfile.flush().await?;

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), util::BoxError> {
    // A vector containing all the URLs to download
    let download_links = vec![
        "https://file-examples-com.github.io/uploads/2017/11/file_example_WAV_10MG.wav", // 10MB WAV audio file
        "https://file-examples-com.github.io/uploads/2017/11/file_example_OOG_2MG.ogg", // 2MB OGG audio file
        "https://file-examples-com.github.io/uploads/2017/10/file_example_PNG_3MB.png", // 3MB PNG image
        "https://file-examples-com.github.io/uploads/2017/10/file_example_JPG_1MB.jpg", // 1MB JPG image
    ];

    // Set up a new multi-progress bar.
    // The bar is stored in an `Arc` to facilitate sharing between threads.
    let multibar = std::sync::Arc::new(indicatif::MultiProgress::new());

    // Add an overall progress indicator to the multibar.
    // It has as many steps as the download_links Vector and will increment on completion of each task.
    let main_pb = std::sync::Arc::new(
        multibar
            .clone()
            .add(indicatif::ProgressBar::new(download_links.len() as u64)),
    );
    main_pb.set_style(
        indicatif::ProgressStyle::default_bar().template("{msg} {bar:10} {pos}/{len}"),
    );
    main_pb.set_message("total  ");

    // Make the main progress bar render immediately rather than waiting for the
    // first task to finish.
    main_pb.tick();

    // Convert download_links Vector into stream
    // This is basically a async compatible iterator
    let stream = stream::iter(download_links);

    // Set up a future to iterate over tasks and run up to 2 at a time.
    let tasks = stream
        .enumerate()
        .for_each_concurrent(Some(2), |(_i, download_link)| {
            // Clone multibar and main_pb.  We will move the clones into each task.
            let multibar = multibar.clone();
            let main_pb = main_pb.clone();
            async move {
                let _task = tokio::task::spawn(download_task(download_link, multibar)).await;
                main_pb.inc(1);
            }
        });

    // Set up a future to manage rendering of the multiple progress bars.
    let multibar = {
        // Create a clone of the multibar, which we will move into the task. 
        let multibar = multibar.clone();
        
        // multibar.join() is *not* async and will block until all the progress
        // bars are done, therefore we must spawn it on a separate scheduler
        // on which blocking behavior is allowed.
        tokio::task::spawn_blocking(move || multibar.join())
    };

    // Wait for the tasks to finish.
    tasks.await;

    // Change the message on the overall progress indicator. 
    main_pb.finish_with_message("done");

    // Wait for the progress bars to finish rendering.
    // The first ? unwraps the outer join() in which we are waiting for the
    // future spawned by tokio::task::spawn_blocking to finishe.
    // The second ? unwraps the inner multibar.join().  
    multibar.await??;

    Ok(())
}
