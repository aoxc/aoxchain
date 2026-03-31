use crate::errors::HubError;
use chrono::Utc;
use serde::Serialize;
use std::{
    collections::HashMap,
    process::Stdio,
    sync::Arc,
    time::{Duration, Instant},
};
use tokio::{
    io::{AsyncBufReadExt, BufReader},
    process::Command,
    sync::{Mutex, Semaphore, broadcast},
    time::sleep,
};

#[derive(Clone)]
pub struct Runner {
    pub jobs: Arc<Mutex<HashMap<String, JobRecord>>>,
    settings: RunnerSettings,
    execution_slots: Arc<Semaphore>,
}

#[derive(Clone)]
pub struct JobRecord {
    pub status: crate::domain::JobStatus,
    pub tx: broadcast::Sender<String>,
}

#[derive(Serialize)]
pub struct LaunchResult {
    pub job_id: String,
}

#[derive(Clone, Copy)]
struct RunnerSettings {
    max_concurrent_jobs: usize,
    max_job_records: usize,
    command_timeout: Duration,
    max_output_bytes: usize,
}

impl Runner {
    pub fn new() -> Self {
        let settings = RunnerSettings::default();
        Self {
            jobs: Arc::new(Mutex::new(HashMap::new())),
            execution_slots: Arc::new(Semaphore::new(settings.max_concurrent_jobs)),
            settings,
        }
    }

    pub async fn launch(
        &self,
        job_id: String,
        command_id: String,
        program: String,
        args: Vec<String>,
        env: Vec<(String, String)>,
        workdir: String,
    ) -> Result<LaunchResult, HubError> {
        let permit = self
            .execution_slots
            .clone()
            .try_acquire_owned()
            .map_err(|_| {
                HubError::Capacity(format!(
                    "max concurrent jobs ({}) reached",
                    self.settings.max_concurrent_jobs
                ))
            })?;

        let now = Instant::now();
        let (tx, _) = broadcast::channel(256);

        let status = crate::domain::JobStatus {
            id: job_id.clone(),
            command_id,
            started_at: Utc::now(),
            finished_at: None,
            exit_code: None,
            timed_out: false,
            output: String::new(),
        };

        {
            let mut jobs = self.jobs.lock().await;
            jobs.insert(
                job_id.clone(),
                JobRecord {
                    status,
                    tx: tx.clone(),
                },
            );
            self.prune_jobs(&mut jobs);
        }

        let jobs = self.jobs.clone();
        let task_job_id = job_id.clone();
        let output_limit = self.settings.max_output_bytes;
        let timeout_duration = self.settings.command_timeout;
        let max_job_records = self.settings.max_job_records;

        tokio::spawn(async move {
            let _permit = permit;

            let mut child = match Command::new(&program)
                .args(args)
                .envs(env)
                .current_dir(workdir)
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .spawn()
            {
                Ok(child) => child,
                Err(err) => {
                    let mut map = jobs.lock().await;
                    if let Some(rec) = map.get_mut(&task_job_id) {
                        let _ = append_output(
                            &mut rec.status.output,
                            &format!("Failed to launch process: {err}\n"),
                            output_limit,
                        );
                        rec.status.finished_at = Some(Utc::now());
                        rec.status.exit_code = Some(-1);
                    }
                    let _ = tx.send(format!("Failed to launch process: {err}"));
                    return;
                }
            };

            if let Some(stdout) = child.stdout.take() {
                let txc = tx.clone();
                let jobs_c = jobs.clone();
                let id = task_job_id.clone();

                tokio::spawn(async move {
                    let mut lines = BufReader::new(stdout).lines();

                    while let Ok(Some(line)) = lines.next_line().await {
                        let _ = txc.send(line.clone());

                        let mut guard = jobs_c.lock().await;
                        if let Some(rec) = guard.get_mut(&id) {
                            let should_stop = append_output(
                                &mut rec.status.output,
                                &format!("{line}\n"),
                                output_limit,
                            );
                            if should_stop {
                                break;
                            }
                        } else {
                            break;
                        }
                    }
                });
            }

            if let Some(stderr) = child.stderr.take() {
                let txc = tx.clone();
                let jobs_c = jobs.clone();
                let id = task_job_id.clone();

                tokio::spawn(async move {
                    let mut lines = BufReader::new(stderr).lines();

                    while let Ok(Some(line)) = lines.next_line().await {
                        let formatted = format!("[stderr] {line}");
                        let _ = txc.send(formatted.clone());

                        let mut guard = jobs_c.lock().await;
                        if let Some(rec) = guard.get_mut(&id) {
                            let should_stop = append_output(
                                &mut rec.status.output,
                                &format!("{formatted}\n"),
                                output_limit,
                            );
                            if should_stop {
                                break;
                            }
                        } else {
                            break;
                        }
                    }
                });
            }

            let timeout = sleep(timeout_duration);
            tokio::pin!(timeout);

            let exit_code = tokio::select! {
                status = child.wait() => status.ok().and_then(|s| s.code()),
                _ = &mut timeout => {
                    let _ = child.kill().await;
                    let mut guard = jobs.lock().await;
                    if let Some(rec) = guard.get_mut(&task_job_id) {
                        rec.status.timed_out = true;
                    }
                    Some(124)
                }
            };

            let mut guard = jobs.lock().await;
            if let Some(rec) = guard.get_mut(&task_job_id) {
                rec.status.exit_code = exit_code;
                rec.status.finished_at = Some(Utc::now());

                let _ = append_output(
                    &mut rec.status.output,
                    &format!("\n[metrics] wall_time_ms={}\n", now.elapsed().as_millis()),
                    output_limit,
                );
            }

            prune_jobs_with_limit(&mut guard, max_job_records);
            let _ = tx.send(String::from("[process finished]"));
        });

        Ok(LaunchResult { job_id })
    }

    pub async fn get_job(&self, id: &str) -> Option<crate::domain::JobStatus> {
        self.jobs
            .lock()
            .await
            .get(id)
            .map(|record| record.status.clone())
    }

    pub async fn subscribe(&self, id: &str) -> Option<broadcast::Receiver<String>> {
        self.jobs
            .lock()
            .await
            .get(id)
            .map(|record| record.tx.subscribe())
    }

    fn prune_jobs(&self, jobs: &mut HashMap<String, JobRecord>) {
        prune_jobs_with_limit(jobs, self.settings.max_job_records);
    }
}

impl Default for Runner {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for RunnerSettings {
    fn default() -> Self {
        Self {
            max_concurrent_jobs: env_or_default("AOXCHUB_MAX_CONCURRENT_JOBS", 8),
            max_job_records: env_or_default("AOXCHUB_MAX_JOB_RECORDS", 512),
            command_timeout: Duration::from_secs(
                env_or_default("AOXCHUB_COMMAND_TIMEOUT_SECS", 300) as u64,
            ),
            max_output_bytes: env_or_default("AOXCHUB_MAX_OUTPUT_BYTES", 512 * 1024),
        }
    }
}

/// Appends text into the job output buffer while enforcing a strict byte ceiling.
///
/// Security and operational rationale:
/// - Prevents unbounded in-memory growth from verbose child processes.
/// - Ensures output truncation is deterministic and centralized.
/// - Returns `true` when the caller should stop appending additional content.
fn append_output(buffer: &mut String, chunk: &str, max_output_bytes: usize) -> bool {
    if buffer.len() >= max_output_bytes {
        return true;
    }

    let remaining = max_output_bytes.saturating_sub(buffer.len());
    if remaining == 0 {
        return true;
    }

    if chunk.len() <= remaining {
        buffer.push_str(chunk);
        return false;
    }

    let mut safe_cut = 0usize;
    for (idx, _) in chunk.char_indices() {
        if idx <= remaining {
            safe_cut = idx;
        } else {
            break;
        }
    }

    if safe_cut > 0 {
        buffer.push_str(&chunk[..safe_cut]);
    }

    const TRUNCATION_MARKER: &str = "\n[output truncated]\n";
    if buffer.len() < max_output_bytes {
        let marker_remaining = max_output_bytes.saturating_sub(buffer.len());
        if TRUNCATION_MARKER.len() <= marker_remaining {
            buffer.push_str(TRUNCATION_MARKER);
        } else if marker_remaining > 0 {
            let mut marker_safe_cut = 0usize;
            for (idx, _) in TRUNCATION_MARKER.char_indices() {
                if idx <= marker_remaining {
                    marker_safe_cut = idx;
                } else {
                    break;
                }
            }
            if marker_safe_cut > 0 {
                buffer.push_str(&TRUNCATION_MARKER[..marker_safe_cut]);
            }
        }
    }

    true
}

/// Prunes the in-memory job registry down to the configured retention ceiling.
///
/// Retention policy:
/// 1. Prefer removing the oldest completed job first.
/// 2. If no completed jobs exist, remove the oldest started job.
/// 3. Continue until the registry size is compliant.
fn prune_jobs_with_limit(jobs: &mut HashMap<String, JobRecord>, max_job_records: usize) {
    while jobs.len() > max_job_records {
        let candidate = jobs
            .iter()
            .filter_map(|(id, record)| {
                record
                    .status
                    .finished_at
                    .map(|finished_at| (id.clone(), finished_at))
            })
            .min_by_key(|(_, finished_at)| *finished_at)
            .map(|(id, _)| id)
            .or_else(|| {
                jobs.iter()
                    .min_by_key(|(_, record)| record.status.started_at)
                    .map(|(id, _)| id.clone())
            });

        if let Some(job_id) = candidate {
            jobs.remove(&job_id);
        } else {
            break;
        }
    }
}

fn env_or_default(name: &str, default: usize) -> usize {
    std::env::var(name)
        .ok()
        .and_then(|value| value.parse::<usize>().ok())
        .filter(|value| *value > 0)
        .unwrap_or(default)
}

#[cfg(test)]
impl Runner {
    fn with_limits(max_concurrent_jobs: usize, max_job_records: usize) -> Self {
        let settings = RunnerSettings {
            max_concurrent_jobs,
            max_job_records,
            command_timeout: Duration::from_secs(5),
            max_output_bytes: 1024,
        };

        Self {
            jobs: Arc::new(Mutex::new(HashMap::new())),
            execution_slots: Arc::new(Semaphore::new(max_concurrent_jobs)),
            settings,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn rejects_launch_when_concurrency_limit_is_reached() {
        let runner = Runner::with_limits(1, 16);

        let first = runner
            .launch(
                String::from("job-1"),
                String::from("cmd"),
                String::from("bash"),
                vec![String::from("-lc"), String::from("sleep 1")],
                vec![],
                String::from("/tmp"),
            )
            .await;

        assert!(first.is_ok());

        let second = runner
            .launch(
                String::from("job-2"),
                String::from("cmd"),
                String::from("bash"),
                vec![String::from("-lc"), String::from("echo blocked")],
                vec![],
                String::from("/tmp"),
            )
            .await;

        assert!(matches!(second, Err(HubError::Capacity(_))));
    }

    #[tokio::test]
    async fn prunes_finished_jobs_beyond_capacity() {
        let runner = Runner::with_limits(2, 2);

        for n in 0..4 {
            let job_id = format!("job-{n}");
            let _ = runner
                .launch(
                    job_id,
                    String::from("cmd"),
                    String::from("bash"),
                    vec![String::from("-lc"), String::from("echo ok")],
                    vec![],
                    String::from("/tmp"),
                )
                .await;

            tokio::time::sleep(Duration::from_millis(60)).await;
        }

        let jobs = runner.jobs.lock().await;
        assert!(jobs.len() <= 2);
    }
}
