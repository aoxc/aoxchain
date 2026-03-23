use std::{
    env,
    ffi::OsString,
    fs,
    path::{Path, PathBuf},
    sync::{Mutex, MutexGuard, OnceLock},
    time::{SystemTime, UNIX_EPOCH},
};

fn env_lock() -> &'static Mutex<()> {
    static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    LOCK.get_or_init(|| Mutex::new(()))
}

fn unique_test_home(label: &str) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time should be after unix epoch")
        .as_nanos();
    env::temp_dir().join(format!("aoxcmd-{label}-{nanos}"))
}

pub(crate) struct TestHome {
    _guard: MutexGuard<'static, ()>,
    previous_home: Option<OsString>,
    path: PathBuf,
}

impl TestHome {
    pub(crate) fn new(label: &str) -> Self {
        let guard = env_lock().lock().expect("test env mutex must lock");
        let path = unique_test_home(label);
        let previous_home = env::var_os("AOXC_HOME");
        env::set_var("AOXC_HOME", &path);

        Self {
            _guard: guard,
            previous_home,
            path,
        }
    }

    #[allow(dead_code)]
    pub(crate) fn path(&self) -> &Path {
        &self.path
    }
}

impl Drop for TestHome {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.path);
        match &self.previous_home {
            Some(value) => env::set_var("AOXC_HOME", value),
            None => env::remove_var("AOXC_HOME"),
        }
    }
}
