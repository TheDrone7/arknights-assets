use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use std::time::Duration;

pub struct DownloadUi {
    multi: MultiProgress,
    main_pb: ProgressBar,
}

impl DownloadUi {
    pub fn new(total_files: u64) -> Self {
        let multi = MultiProgress::new();
        let style =
            ProgressStyle::with_template("[{elapsed_precise}] {bar:40.cyan/blue} {pos:>7}/{len:7}")
                .unwrap();

        let main_pb = multi.add(ProgressBar::new(total_files));
        main_pb.set_style(style);
        main_pb.enable_steady_tick(Duration::from_secs(1));

        Self { multi, main_pb }
    }

    pub fn inc_main(&self) {
        self.main_pb.inc(1);
    }

    pub fn finish(&self) {
        self.main_pb.finish_and_clear();
    }

    pub fn add_download_bar(&self, filename: &str) -> ProgressBar {
        let style = ProgressStyle::with_template("{spinner} {wide_msg}").unwrap();
        let pb = self.multi.add(ProgressBar::new_spinner());
        pb.set_style(style);
        pb.set_message(format!("Downloading {}", filename));
        pb.enable_steady_tick(Duration::from_millis(100));
        pb
    }
}
