use std::collections::HashMap;
use std::time::Duration;
use uuid::Uuid;
use anyhow::{anyhow, Result};
use esp_idf_svc::timer::{EspTaskTimerService, EspTimer};


pub struct TaskService {
    timer_service: EspTaskTimerService,
    timers: HashMap<Uuid, EspTimer>,
}

impl TaskService {
    pub fn new() -> Result<TaskService> {

        let timer_service = EspTaskTimerService::new()?;

        Ok(
            TaskService {
                timer_service,
                timers: HashMap::new(),
            }
        )
    }

    pub fn schedule(&mut self, callback: impl FnMut() + Send + 'static, every: Duration) -> Result<Uuid> {
        let timer = self.timer_service.timer(callback)?;
        timer.every(every)?;

        let uuid = Uuid::new_v4();
        self.timers.insert(uuid, timer);

        Ok(uuid)
    }

    pub fn cancel(&self, uuid: &Uuid) -> Result<()> {
        let timer = self.timers.get(uuid)
            .ok_or(anyhow!("Task not found"))?;

        timer.cancel()?;

        Ok(())
    }

}