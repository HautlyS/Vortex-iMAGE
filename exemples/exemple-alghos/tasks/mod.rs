//! Collaborative Task Lists with CRDT sync
//! P2P task management with conflict-free replication
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct TaskList {
    pub id: String,
    pub name: String,
    pub tasks: Vec<Task>,
    pub owner: String,
    pub collaborators: Vec<String>,
    pub created_at: u64,
    pub updated_at: u64,
    pub color: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Task {
    pub id: String,
    pub title: String,
    pub description: Option<String>,
    pub completed: bool,
    pub priority: Priority,
    pub due_date: Option<u64>,
    pub assignee: Option<String>,
    pub created_by: String,
    pub created_at: u64,
    pub updated_at: u64,
    pub subtasks: Vec<SubTask>,
    pub tags: Vec<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct SubTask {
    pub id: String,
    pub title: String,
    pub completed: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum Priority {
    Low,
    Medium,
    High,
    Urgent,
}

impl Default for Priority {
    fn default() -> Self { Self::Medium }
}

/// CRDT operations for task sync
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum TaskOp {
    CreateList { list: TaskList, clock: u64 },
    DeleteList { list_id: String, clock: u64 },
    AddTask { list_id: String, task: Task, clock: u64 },
    UpdateTask { list_id: String, task_id: String, updates: TaskUpdate, clock: u64 },
    DeleteTask { list_id: String, task_id: String, clock: u64 },
    ToggleTask { list_id: String, task_id: String, completed: bool, clock: u64 },
    ReorderTask { list_id: String, task_id: String, new_index: usize, clock: u64 },
    AddSubtask { list_id: String, task_id: String, subtask: SubTask, clock: u64 },
    ToggleSubtask { list_id: String, task_id: String, subtask_id: String, completed: bool, clock: u64 },
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Default)]
pub struct TaskUpdate {
    pub title: Option<String>,
    pub description: Option<Option<String>>,
    pub priority: Option<Priority>,
    pub due_date: Option<Option<u64>>,
    pub assignee: Option<Option<String>>,
    pub tags: Option<Vec<String>>,
}

/// Task manager with CRDT sync
pub struct TaskManager {
    pub lists: HashMap<String, TaskList>,
    pub clock: u64,
    pub author: String,
    pending_ops: Vec<TaskOp>,
}

impl TaskManager {
    pub fn new(author: String) -> Self {
        Self {
            lists: HashMap::new(),
            clock: 0,
            author,
            pending_ops: Vec::new(),
        }
    }

    fn next_clock(&mut self) -> u64 {
        self.clock += 1;
        self.clock
    }

    fn gen_id() -> String {
        use std::time::{SystemTime, UNIX_EPOCH};
        let ts = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis();
        format!("{:x}{:04x}", ts, rand::random::<u16>())
    }

    pub fn create_list(&mut self, name: String, color: String) -> TaskList {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH).unwrap().as_secs();
        
        let list = TaskList {
            id: Self::gen_id(),
            name,
            tasks: Vec::new(),
            owner: self.author.clone(),
            collaborators: Vec::new(),
            created_at: now,
            updated_at: now,
            color,
        };
        
        let clock = self.next_clock();
        self.pending_ops.push(TaskOp::CreateList { list: list.clone(), clock });
        self.lists.insert(list.id.clone(), list.clone());
        list
    }

    pub fn add_task(&mut self, list_id: &str, title: String, priority: Priority) -> Option<Task> {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH).unwrap().as_secs();
        
        let task = Task {
            id: Self::gen_id(),
            title,
            description: None,
            completed: false,
            priority,
            due_date: None,
            assignee: None,
            created_by: self.author.clone(),
            created_at: now,
            updated_at: now,
            subtasks: Vec::new(),
            tags: Vec::new(),
        };
        
        let clock = self.next_clock();
        if let Some(list) = self.lists.get_mut(list_id) {
            self.pending_ops.push(TaskOp::AddTask {
                list_id: list_id.to_string(),
                task: task.clone(),
                clock,
            });
            list.tasks.push(task.clone());
            list.updated_at = now;
            Some(task)
        } else {
            None
        }
    }

    pub fn toggle_task(&mut self, list_id: &str, task_id: &str) -> bool {
        let clock = self.next_clock();
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH).unwrap().as_secs();
        
        if let Some(list) = self.lists.get_mut(list_id) {
            if let Some(task) = list.tasks.iter_mut().find(|t| t.id == task_id) {
                task.completed = !task.completed;
                task.updated_at = now;
                
                self.pending_ops.push(TaskOp::ToggleTask {
                    list_id: list_id.to_string(),
                    task_id: task_id.to_string(),
                    completed: task.completed,
                    clock,
                });
                return true;
            }
        }
        false
    }

    pub fn delete_task(&mut self, list_id: &str, task_id: &str) -> bool {
        if let Some(list) = self.lists.get_mut(list_id) {
            if let Some(pos) = list.tasks.iter().position(|t| t.id == task_id) {
                list.tasks.remove(pos);
                let clock = self.next_clock();
                self.pending_ops.push(TaskOp::DeleteTask {
                    list_id: list_id.to_string(),
                    task_id: task_id.to_string(),
                    clock,
                });
                return true;
            }
        }
        false
    }

    pub fn update_task(&mut self, list_id: &str, task_id: &str, updates: TaskUpdate) -> bool {
        if let Some(list) = self.lists.get_mut(list_id) {
            if let Some(task) = list.tasks.iter_mut().find(|t| t.id == task_id) {
                if let Some(title) = &updates.title { task.title = title.clone(); }
                if let Some(desc) = &updates.description { task.description = desc.clone(); }
                if let Some(priority) = &updates.priority { task.priority = priority.clone(); }
                if let Some(due) = &updates.due_date { task.due_date = *due; }
                if let Some(assignee) = &updates.assignee { task.assignee = assignee.clone(); }
                if let Some(tags) = &updates.tags { task.tags = tags.clone(); }
                
                task.updated_at = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH).unwrap().as_secs();
                
                let clock = self.next_clock();
                self.pending_ops.push(TaskOp::UpdateTask {
                    list_id: list_id.to_string(),
                    task_id: task_id.to_string(),
                    updates,
                    clock,
                });
                return true;
            }
        }
        false
    }

    pub fn add_subtask(&mut self, list_id: &str, task_id: &str, title: String) -> Option<SubTask> {
        let subtask = SubTask {
            id: Self::gen_id(),
            title,
            completed: false,
        };
        
        let clock = self.next_clock();
        if let Some(list) = self.lists.get_mut(list_id) {
            if let Some(task) = list.tasks.iter_mut().find(|t| t.id == task_id) {
                self.pending_ops.push(TaskOp::AddSubtask {
                    list_id: list_id.to_string(),
                    task_id: task_id.to_string(),
                    subtask: subtask.clone(),
                    clock,
                });
                task.subtasks.push(subtask.clone());
                return Some(subtask);
            }
        }
        None
    }

    pub fn toggle_subtask(&mut self, list_id: &str, task_id: &str, subtask_id: &str) -> bool {
        let clock = self.next_clock();
        if let Some(list) = self.lists.get_mut(list_id) {
            if let Some(task) = list.tasks.iter_mut().find(|t| t.id == task_id) {
                if let Some(subtask) = task.subtasks.iter_mut().find(|s| s.id == subtask_id) {
                    subtask.completed = !subtask.completed;
                    self.pending_ops.push(TaskOp::ToggleSubtask {
                        list_id: list_id.to_string(),
                        task_id: task_id.to_string(),
                        subtask_id: subtask_id.to_string(),
                        completed: subtask.completed,
                        clock,
                    });
                    return true;
                }
            }
        }
        false
    }

    /// Apply remote operation (CRDT merge)
    pub fn apply_op(&mut self, op: TaskOp) {
        match op {
            TaskOp::CreateList { list, clock } => {
                if clock > self.clock { self.clock = clock; }
                self.lists.entry(list.id.clone()).or_insert(list);
            }
            TaskOp::DeleteList { list_id, clock } => {
                if clock > self.clock { self.clock = clock; }
                self.lists.remove(&list_id);
            }
            TaskOp::AddTask { list_id, task, clock } => {
                if clock > self.clock { self.clock = clock; }
                if let Some(list) = self.lists.get_mut(&list_id) {
                    if !list.tasks.iter().any(|t| t.id == task.id) {
                        list.tasks.push(task);
                    }
                }
            }
            TaskOp::ToggleTask { list_id, task_id, completed, clock } => {
                if clock > self.clock { self.clock = clock; }
                if let Some(list) = self.lists.get_mut(&list_id) {
                    if let Some(task) = list.tasks.iter_mut().find(|t| t.id == task_id) {
                        task.completed = completed;
                    }
                }
            }
            TaskOp::DeleteTask { list_id, task_id, clock } => {
                if clock > self.clock { self.clock = clock; }
                if let Some(list) = self.lists.get_mut(&list_id) {
                    list.tasks.retain(|t| t.id != task_id);
                }
            }
            TaskOp::UpdateTask { list_id, task_id, updates, clock } => {
                if clock > self.clock { self.clock = clock; }
                if let Some(list) = self.lists.get_mut(&list_id) {
                    if let Some(task) = list.tasks.iter_mut().find(|t| t.id == task_id) {
                        if let Some(title) = updates.title { task.title = title; }
                        if let Some(desc) = updates.description { task.description = desc; }
                        if let Some(priority) = updates.priority { task.priority = priority; }
                        if let Some(due) = updates.due_date { task.due_date = due; }
                        if let Some(assignee) = updates.assignee { task.assignee = assignee; }
                        if let Some(tags) = updates.tags { task.tags = tags; }
                    }
                }
            }
            TaskOp::ReorderTask { list_id, task_id, new_index, clock } => {
                if clock > self.clock { self.clock = clock; }
                if let Some(list) = self.lists.get_mut(&list_id) {
                    if let Some(pos) = list.tasks.iter().position(|t| t.id == task_id) {
                        let task = list.tasks.remove(pos);
                        let idx = new_index.min(list.tasks.len());
                        list.tasks.insert(idx, task);
                    }
                }
            }
            TaskOp::AddSubtask { list_id, task_id, subtask, clock } => {
                if clock > self.clock { self.clock = clock; }
                if let Some(list) = self.lists.get_mut(&list_id) {
                    if let Some(task) = list.tasks.iter_mut().find(|t| t.id == task_id) {
                        if !task.subtasks.iter().any(|s| s.id == subtask.id) {
                            task.subtasks.push(subtask);
                        }
                    }
                }
            }
            TaskOp::ToggleSubtask { list_id, task_id, subtask_id, completed, clock } => {
                if clock > self.clock { self.clock = clock; }
                if let Some(list) = self.lists.get_mut(&list_id) {
                    if let Some(task) = list.tasks.iter_mut().find(|t| t.id == task_id) {
                        if let Some(subtask) = task.subtasks.iter_mut().find(|s| s.id == subtask_id) {
                            subtask.completed = completed;
                        }
                    }
                }
            }
        }
    }

    /// Get pending operations for sync
    pub fn drain_pending(&mut self) -> Vec<TaskOp> {
        std::mem::take(&mut self.pending_ops)
    }

    /// Serialize operations to bytes
    pub fn serialize_ops(ops: &[TaskOp]) -> Vec<u8> {
        bincode::serialize(ops).unwrap_or_default()
    }

    /// Deserialize operations from bytes
    pub fn deserialize_ops(data: &[u8]) -> Vec<TaskOp> {
        bincode::deserialize(data).unwrap_or_default()
    }

    pub fn get_list(&self, id: &str) -> Option<&TaskList> {
        self.lists.get(id)
    }

    pub fn all_lists(&self) -> Vec<&TaskList> {
        self.lists.values().collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_list_and_tasks() {
        let mut mgr = TaskManager::new("user1".into());
        let list = mgr.create_list("My Tasks".into(), "#8b5cf6".into());
        
        assert_eq!(list.name, "My Tasks");
        assert!(mgr.lists.contains_key(&list.id));
        
        let task = mgr.add_task(&list.id, "Test task".into(), Priority::High).unwrap();
        assert_eq!(task.title, "Test task");
        assert!(!task.completed);
        
        mgr.toggle_task(&list.id, &task.id);
        let updated = mgr.get_list(&list.id).unwrap();
        assert!(updated.tasks[0].completed);
    }

    #[test]
    fn test_crdt_merge() {
        let mut mgr1 = TaskManager::new("user1".into());
        let mut mgr2 = TaskManager::new("user2".into());
        
        let list = mgr1.create_list("Shared".into(), "#3b82f6".into());
        
        // Sync list to mgr2
        for op in mgr1.drain_pending() {
            mgr2.apply_op(op);
        }
        
        // Both add tasks concurrently
        mgr1.add_task(&list.id, "Task from user1".into(), Priority::Medium);
        mgr2.add_task(&list.id, "Task from user2".into(), Priority::Low);
        
        // Sync both ways
        for op in mgr1.drain_pending() {
            mgr2.apply_op(op);
        }
        for op in mgr2.drain_pending() {
            mgr1.apply_op(op);
        }
        
        // Both should have 2 tasks
        assert_eq!(mgr1.get_list(&list.id).unwrap().tasks.len(), 2);
        assert_eq!(mgr2.get_list(&list.id).unwrap().tasks.len(), 2);
    }
}
