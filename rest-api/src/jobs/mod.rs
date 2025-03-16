pub struct IntervalJob {
    pub cron_tab: String,
}

pub struct JobManager {
    jobs: Vec<IntervalJob>
}

impl JobManager {
    pub async fn new() -> Self {
        Self { jobs: vec![] }
    }

    pub fn add(&mut self, job: IntervalJob) {
        self.jobs.push(job);
    }

    pub fn start(&self) {
    }
}
