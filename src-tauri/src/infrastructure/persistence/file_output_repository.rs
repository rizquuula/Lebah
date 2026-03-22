use std::io::Write;
use std::sync::Arc;

use crate::domain::errors::DomainError;
use crate::domain::project::value_objects::ProjectId;
use crate::domain::repositories::OutputRepository;
use crate::domain::task::value_objects::TaskId;
use crate::infrastructure::persistence::path_resolver::PathResolver;

pub struct FileOutputRepository {
    resolver: Arc<PathResolver>,
}

impl FileOutputRepository {
    pub fn new(resolver: Arc<PathResolver>) -> Self {
        Self { resolver }
    }
}

impl OutputRepository for FileOutputRepository {
    fn append(
        &self,
        project_id: &ProjectId,
        task_id: &TaskId,
        line: &str,
    ) -> Result<(), DomainError> {
        let dir = self.resolver.outputs_dir(project_id);
        std::fs::create_dir_all(&dir).map_err(|e| DomainError::InvalidValue(e.to_string()))?;
        let path = self.resolver.output_file(project_id, &task_id.0);
        let mut file = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&path)
            .map_err(|e| DomainError::InvalidValue(e.to_string()))?;
        writeln!(file, "{}", line).map_err(|e| DomainError::InvalidValue(e.to_string()))
    }

    fn load_all(&self, project_id: &ProjectId, task_id: &TaskId) -> Vec<String> {
        let path = self.resolver.output_file(project_id, &task_id.0);
        std::fs::read_to_string(&path)
            .unwrap_or_default()
            .lines()
            .filter(|l| !l.is_empty())
            .map(|l| l.to_string())
            .collect()
    }

    fn clear(&self, project_id: &ProjectId, task_id: &TaskId) -> Result<(), DomainError> {
        let path = self.resolver.output_file(project_id, &task_id.0);
        if path.exists() {
            std::fs::remove_file(&path).map_err(|e| DomainError::InvalidValue(e.to_string()))?;
        }
        Ok(())
    }
}
