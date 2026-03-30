use crate::errors::HubError;
use chrono::Utc;
use serde::Serialize;
use std::{collections::HashMap, process::Stdio, sync::Arc, time::Duration};
use tokio::{
    io::{AsyncBufReadExt, BufReader},
    process::Command,
    sync::{Mutex, broadcast},
    time::sleep,
};

#[derive(Clone)]
pub struct Runner {
    pub jobs: Arc<Mutex<HashMap<String, JobRecord>>>,
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

impl Runner {
    pub fn new() -> Self {
        Self {
            jobs: Arc::new(Mutex::new(HashMap::new())),
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
        self.jobs.lock().await.insert(
            job_id.clone(),
            JobRecord {
                status,
                tx: tx.clone(),
            },
        );

        let jobs = self.jobs.clone();
        let task_job_id = job_id.clone();
        tokio::spawn(async move {
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
                        rec.status
                            .output
                            .push_str(&format!("Failed to launch process: {err}\n"));
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
                            if rec.status.output.len() > 512 * 1024 {
                                rec.status.output.truncate(512 * 1024);
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
                            rec.status.output.push_str("[stderr] ");
                            rec.status.output.push_str(&line);
                            rec.status.output.push('\n');
                        }
                    }
                });
            }

            let timeout = sleep(Duration::from_secs(300));
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
            }
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
}
