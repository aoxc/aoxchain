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
                Ok(c) => c,
                Err(err) => {
                    let mut map = jobs.lock().await;
                    if let Some(rec) = map.get_mut(&task_job_id) {
                        append_output(
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

            if let Some(out) = child.stdout.take() {
                let txc = tx.clone();
                let jobs_c = jobs.clone();
                let id = task_job_id.clone();
                tokio::spawn(async move {
                    let mut lines = BufReader::new(out).lines();
                    while let Ok(Some(line)) = lines.next_line().await {
                        let _ = txc.send(line.clone());
                        let mut guard = jobs_c.lock().await;
                        if let Some(rec) = guard.get_mut(&id) {
                            rec.status.output.push_str(&line);
                            rec.status.output.push('\n');
                            if rec.status.output.len() > output_limit {
                                rec.status.output.truncate(output_limit);
                                rec.status.output.push_str("\n[output truncated]\n");
                                break;
                            }
                        }
                    }
                });
            }

            if let Some(err) = child.stderr.take() {
                let txc = tx.clone();
                let jobs_c = jobs.clone();
                let id = task_job_id.clone();
                tokio::spawn(async move {
                    let mut lines = BufReader::new(err).lines();
                    while let Ok(Some(line)) = lines.next_line().await {
                        let _ = txc.send(format!("[stderr] {line}"));
                        let mut guard = jobs_c.lock().await;
                        if let Some(rec) = guard.get_mut(&id) {
                            let _ = append_output(
                                &mut rec.status.output,
                                &format!("[stderr] {line}\n"),
                                output_limit,
                            );
                        }
                    }
                });
            }

            let timeout = sleep(timeout_duration);
            tokio::pin!(timeout);
            let exit = tokio::select! {
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
                rec.status.exit_code = exit;
                rec.status.finished_at = Some(Utc::now());
                rec.status.output.push_str(&format!(
                    "\n[metrics] wall_time_ms={}",
                    now.elapsed().as_millis()
                ));
            }
            prune_jobs_with_limit(&mut guard, max_job_records);
            let _ = tx.send(String::from("[process finished]"));
        });

        Ok(LaunchResult { job_id })
    }

    pub async fn get_job(&self, id: &str) -> Option<crate::domain::JobStatus> {
        self.jobs.lock().await.get(id).map(|j| j.status.clone())
    }

    pub async fn subscribe(&self, id: &str) -> Option<broadcast::Receiver<String>> {
        self.jobs.lock().await.get(id).map(|r| r.tx.subscribe())
    }

    fn prune_jobs(&self, jobs: &mut HashMap<String, JobRecord>) {
        while jobs.len() > self.settings.max_job_records {
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
}

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

fn env_or_default(name: &str, default: usize) -> usize {
    std::env::var(name)
        .ok()
        .and_then(|v| v.parse::<usize>().ok())
        .filter(|v| *v > 0)
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
