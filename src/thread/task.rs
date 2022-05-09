use crate::{mem, thread::sync};
use core::alloc::GlobalAlloc;

pub type Runner = &'static dyn Fn() -> ();
type StackFrame = [u32; 19];

pub enum TaskError {
	Alloc,  // Could not allocate the memory
	MaxNtasks(usize),  // The max. allowed number of tasks has been exceeded
	NotFound,
}

#[derive(Clone)]
pub struct Task {
	runner: Runner,
	stack_begin: *mut u8,
	stack_frame: *mut StackFrame,
}

extern "C" {
	fn task_state_save(mem_into: *const u8);
	fn task_state_load(mem_from: *const u8);
}

mod registry {
	use super::{Task, StackFrame, TaskError};

	const TASKS_MAX: usize = 2;
	static mut REGISTRY: [*const Task; TASKS_MAX] = [
		0 as *const Task,
		0 as *const Task,
	];

	struct State {
		current_id: usize,
		free: usize,
	}

	static mut STATE: State = State {current_id: 0, free: TASKS_MAX};

	pub fn add(task: &Task) -> Result<(), TaskError> {
		unsafe {
			for mut t in REGISTRY {
				if t.is_null() {
					t = task;

					return Ok(())
				}
			}
		}

		Err(TaskError::MaxNtasks(TASKS_MAX))
	}

	fn find(task: *const Task) -> Result<*mut *const Task, TaskError> {
		unsafe {
			for mut t in REGISTRY {
				if task == t {
					return Ok(&mut t)
				}
			}
		}

		Err(TaskError::NotFound)
	}

	pub fn remove(task: &Task) -> Result<(), TaskError> {
		let mut registry_entry = find(task)?;

		unsafe {
			*registry_entry = 0 as *const Task;
		}

		Ok(())
	}

	pub fn get_next_round_robin<'a>() -> Result<&'a Task, TaskError> {
		unsafe {
			for id in (STATE.current_id + 1)..(STATE.current_id + TASKS_MAX) {
				let task = REGISTRY[id % TASKS_MAX];

				if !task.is_null() {
					STATE.current_id = id % TASKS_MAX;

					return Ok(&*task);
				}
			}
		}

		Err(TaskError::NotFound)
	}
}

impl Task {
	fn runner_wrap(id: usize) {
	}

	fn new() -> Task {
		return Task {runner: &|| (), stack_begin: 0 as *mut u8, stack_frame: 0 as *mut StackFrame}
	}

	/// Checks whether memory for the task has been allocated successfully
	///
	pub fn is_alloc(&self) -> bool {
		!(self.stack_begin.is_null() || self.stack_frame.is_null())
	}

	/// Tries to allocate memory required for the task
	///
	pub fn from_stack_size(runner: Runner, stack_size: usize) -> Result<Task, TaskError> {
		let _critical = sync::Critical::new();

		unsafe {
			let mut task = Task::new();
			task.stack_begin = mem::ALLOCATOR.alloc(core::alloc::Layout::from_size_align(stack_size, 4).unwrap());
			task.stack_frame = mem::ALLOCATOR.alloc(core::alloc::Layout::from_size_align(core::mem::size_of::<StackFrame>(), 4).unwrap()) as *mut StackFrame;

			if !task.is_alloc() {
				return Err(TaskError::Alloc)
			}

			Ok(task)
		}
	}

	/// Enqueues the task for context switching
	///
	pub fn start(&self) {
		(self.runner)();
	}
}
