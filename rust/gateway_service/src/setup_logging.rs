use tracing_subscriber::filter::LevelFilter;

pub fn setup_logging() {
    #[cfg(not(feature = "local"))]
    {
        // required to enable CloudWatch error logging by the runtime
        tracing_subscriber::fmt()
            .with_max_level(LevelFilter::INFO)
            // disable printing the name of the module in every log line.
            .with_target(false)
            // this needs to be set to false, otherwise ANSI color codes will
            // show up in a confusing manner in CloudWatch logs.
            .with_ansi(false)
            // disabling time is handy because CloudWatch will add the ingestion time.
            .without_time()
            .init();
    }
}
