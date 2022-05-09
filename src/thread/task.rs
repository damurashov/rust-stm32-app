use crate::{mem, thread::sync};
use core::alloc::GlobalAlloc;

pub type Runner = &'static dyn Fn() -> ();
type StackFrame = [u32; 19];

pub enum TaskError {
	Alloc(usize),  // Could not allocate the memory
	MaxNtasks(usize)  // The max. allowed number of tasks has been exceeded
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

	pub fn add(task: &Task) -> Result<(), TaskError> {
		unsafe {
			for mut t in REGISTRY {
				if 0 == t as usize {
					t = task;

					return Ok(());
				}
			}
		}

		Err(TaskError::MaxNtasks(TASKS_MAX))
	}
}

impl Task {
	fn runner_wrap(id: usize) {
	}

	/// Tries to allocate memory required for the task
	///
	pub fn from_stack_size(runner: Runner, stack_size: usize) -> Result<Task, TaskError> {
		let _critical = sync::Critical::new();

		unsafe {
			let mut stack_begin = mem::ALLOCATOR.alloc(core::alloc::Layout::from_size_align(stack_size, 4).unwrap());

			if stack_begin == 0 as *mut u8 {
				return Err(TaskError::Alloc(stack_size));
			}

			let mut stack_frame = mem::ALLOCATOR.alloc(core::alloc::Layout::from_size_align(core::mem::size_of::<StackFrame>(), 4).unwrap()) as *mut StackFrame;

			if stack_frame == 0 as *mut StackFrame {
				mem::ALLOCATOR.dealloc(stack_begin, core::alloc::Layout::from_size_align(stack_size, 4).unwrap());
				return Err(TaskError::Alloc(core::mem::size_of::<StackFrame>()));
			}

			Ok(Task {runner, stack_begin, stack_frame})
		}
	}

	/// Enqueues the task for context switching
	///
	pub fn start(&self) {
		(self.runner)();
	}
}
