use std::path::PathBuf;
use crate::domain::project::value_objects::ProjectId;

pub struct PathResolver {
    base_dir: PathBuf,
}

impl PathResolver {
    pub fn new() -> Result<Self, String> {
        let home = dirs::home_dir().ok_or("Cannot determine home directory")?;
        let base_dir = home.join(".lebahcode");
        std::fs::create_dir_all(base_dir.join("projects")).map_err(|e| e.to_string())?;
        Ok(Self { base_dir })
    }

    pub fn global_config_path(&self) -> PathBuf {
        self.base_dir.join("config.json")
    }

    pub fn project_dir(&self, project_id: &ProjectId) -> PathBuf {
        self.base_dir.join("projects").join(project_id.as_str())
    }

    pub fn tasks_path(&self, project_id: &ProjectId) -> PathBuf {
        self.project_dir(project_id).join("tasks.json")
    }

    pub fn project_config_path(&self, project_id: &ProjectId) -> PathBuf {
        self.project_dir(project_id).join("project_config.json")
    }

    pub fn outputs_dir(&self, project_id: &ProjectId) -> PathBuf {
        self.project_dir(project_id).join("outputs")
    }

    pub fn output_file(&self, project_id: &ProjectId, task_id_str: &str) -> PathBuf {
        self.outputs_dir(project_id).join(format!("{}.jsonl", task_id_str))
    }
}
