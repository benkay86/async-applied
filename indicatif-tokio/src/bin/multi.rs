use futures::stream::StreamExt;
use rand::Rng;

// tokio::main macro automatically sets up the tokio runtime.
#[tokio::main]
async fn main() -> Result<(), util::BoxError> {
    // Generate a stream of 10 tasks each with 10 steps.
    // Each task progresses at a random rate,
    // with a delay between 500 and 1500 ms.
    let task_steps = 10;
    let task_delay_millis: (u64, u64) = (500, 1500);
    let ntasks = 10;
    let mut rng = rand::thread_rng();
    let tasks = futures::stream::repeat(())
        .map(|_| tokio::time::interval(
            tokio::time::Duration::from_millis(
                rng.gen_range(task_delay_millis.0..
                task_delay_millis.1
            )
        ))
        .take(task_steps)).take(ntasks);
    
    // Set up a new multi-progress bar.
    // The bar is stored in an `Arc` to facilitate sharing between threads.
    let multibar = std::sync::Arc::new(indicatif::MultiProgress::new());
    
    // Add an overall progress indicator to the multibar.
    // It has 10 steps and will increment on completion of each task.
    let main_pb = std::sync::Arc::new(multibar.clone().add(indicatif::ProgressBar::new(ntasks as u64)));
    main_pb.set_style(
        indicatif::ProgressStyle::default_bar()
        .template("{msg} {bar:10} {pos}/{len}")
    );
    main_pb.set_message("total  ");
    
    // Make the main progress bar render immediately rather than waiting for the
    // first task to finish.
    main_pb.tick();
    
    // Set up a future to iterate over tasks and run up to 3 at a time.
    let tasks = tasks.enumerate().for_each_concurrent(Some(3), |(i, interval)| {
        // Clone multibar and main_pb.  We will move the clones into each task.
        let multibar = multibar.clone();
        let main_pb = main_pb.clone();
        async move {
            // Add a new progress indicator to the multibar.
            let task_pb = multibar.add(indicatif::ProgressBar::new(task_steps as u64));
            task_pb.set_style(
                indicatif::ProgressStyle::default_bar()
                .template("task {msg} {bar:10} {pos}/{len}")
            );
            task_pb.set_message(&format!("{:>2}", i+1));
            
            // Increment this task's progress indicator.
            interval.for_each(|_| async { task_pb.inc(1) }).await;
            
            // Increment the overall progress indicator.
            main_pb.inc(1);
            
            // Clear this tasks's progress indicator.
            task_pb.finish_and_clear();
        }
    });
    
    // Set up a future to manage rendering of the multiple progress bars.
    let multibar = {
        // Create a clone of the multibar, which we will move into the task. 
        let multibar = multibar.clone();
        
        // multibar.join() is *not* async and will block until all the progress
        // bars are done, therefore we must spawn it on a separate scheduler
        // on which blocking behavior is allowed.
        tokio::task::spawn_blocking(move || { multibar.join() })
    };
    
    // Wait for the tasks to finish.
    tasks.await;
    
    // Change the message on the overall progress indicator. 
    main_pb.finish_with_message("done");
    
    // Wait for the progress bars to finish rendering.
    // The first ? unwraps the outer join() in which we are waiting for the
    // future spawned by tokio::task::spawn_blocking to finish.
    // The second ? unwraps the inner multibar.join().  
    multibar.await??;
    
    Ok(())
}
