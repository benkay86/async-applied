use futures::stream::StreamExt;

// tokio::main macro automatically sets up the tokio runtime.
#[tokio::main]
async fn main() -> Result<(), util::BoxError> {
    // Initialize a progress bar with 10 steps.
    let steps = 10;
    let pb = indicatif::ProgressBar::new(steps);
    pb.set_style(
        indicatif::ProgressStyle::default_bar()
        .template("for_each                      {bar:10} {pos}/{len}")
    );
    
    // Stream that yields values no more than once every second.
    // FYI, the value yielded by the stream is a tokio::time::Instant, which is
    // a timestamp, but we do not use it in this example. 
    let interval = tokio::time::interval(tokio::time::Duration::from_secs(1));
    
    // Iterate over the first 10 values in the stream.
    // Increment the progress bar by one for each item.
    interval.take(steps as usize).for_each(|_| async { pb.inc(1) } ).await;
    
    // Finish the progress bar.
    pb.finish();
    
    // Same as above, but using for_each_concurrent.
    // The first parameter is a limit, which limits the number of futures that
    // can be run concurrently.  In this case the limit is 1, making the
    // behavior the same as if we had simply called for_each.
    let pb = indicatif::ProgressBar::new(steps);
    pb.set_style(
        indicatif::ProgressStyle::default_bar()
        .template("for_each_concurrent,  limit 1 {bar:10} {pos}/{len}")
    );
    let interval = tokio::time::interval(tokio::time::Duration::from_secs(1));
    interval.take(steps as usize).for_each_concurrent(
        Some(1),
        |_| async { pb.inc(1) }
    ).await;
    pb.finish();
    
    // Now with the limit set to None, which will attempt to run all the futures
    // concurrently.  The point here is to show that indicatif can handle
    // concurrency.  However, due to the nature of tokio::time::Interval, the
    // progress bar will not actually advance any faster. 
    let pb = indicatif::ProgressBar::new(steps);
    pb.set_style(
        indicatif::ProgressStyle::default_bar()
        .template("for_each_concurrent, no limit {bar:10} {pos}/{len}")
    );
    let interval = tokio::time::interval(tokio::time::Duration::from_secs(1));
    interval.take(steps as usize).for_each_concurrent(
        None,
        |_| async { pb.inc(1) }
    ).await;
    pb.finish();
    
    Ok(())
}
