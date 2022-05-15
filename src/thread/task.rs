use crate::{mem, thread::sync};
use core::alloc::GlobalAlloc;

pub type Runner = &'static dyn Fn() -> ();
type StackFrame = [u32; 16];

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

	pub unsafe fn add(task: &Task) -> Result<(), TaskError> {
		for mut t in REGISTRY {
			if t.is_null() {
				t = task;

				return Ok(())
			}
		}

		Err(TaskError::MaxNtasks(TASKS_MAX))
	}

	unsafe fn find(task: *const Task) -> Result<*mut *const Task, TaskError> {
		for mut t in REGISTRY {
			if task == t {
				return Ok(&mut t)
			}
		}

		Err(TaskError::NotFound)
	}

	pub unsafe fn remove(task: &Task) -> Result<(), TaskError> {
		let mut registry_entry = find(task)?;

		*registry_entry = 0 as *const Task;

		Ok(())
	}

	pub unsafe fn get_next_round_robin<'a>() -> Result<&'a Task, TaskError> {
		for id in (STATE.current_id + 1)..(STATE.current_id + TASKS_MAX) {
			let task = REGISTRY[id % TASKS_MAX];

			if !task.is_null() {
				STATE.current_id = id % TASKS_MAX;

				return Ok(&*task);
			}
		}

		Err(TaskError::NotFound)
	}

	pub unsafe fn get_current<'a>() -> Result<&'a Task, TaskError> {
		let task = REGISTRY[STATE.current_id % TASKS_MAX];

		match task.is_null() {
			true => Ok(&*task),
			false => Err(TaskError::NotFound),
		}
	}

	/// Part of the task-switching ISR.
	///
	/// Saves `chunk_a` (manually-saved 8 4-byte words) and `chunk_b` (automatically saved by Cortex-M0 before switching
	/// to the ISR) into the current task's context storage, and loads the context of a next task.
	///
	/// 2 chunks are used intead of 1, because Cortex-M0 does not save all the registers in one place, and it is easier
	/// to store the rest of them separately due to limitations of the instructions available.
	///
	#[no_mangle]
	unsafe extern "C" fn stack_frame_swap_next(chunk_a: *mut u8, chunk_b: *mut u8) {
		const CHUNK_A_SIZE: usize = 8;
		const CHUNK_B_SIZE: usize = 8;

		match get_next_round_robin() {
			Err(_) => {},  // No other task is pending. Swap is not required.
			Ok(task_next) => {

				// Save the state
				match get_current() {
					Err(_) => {},  // Most likely, the ISR has been called from the main loop which we do not allocate memory for.
					Ok(task) => {
						let stack_frame_ptr = task.stack_frame.cast::<u8>();
						chunk_a.copy_to_nonoverlapping(stack_frame_ptr, CHUNK_A_SIZE);
						chunk_b.copy_to_nonoverlapping(stack_frame_ptr.add(CHUNK_A_SIZE), CHUNK_B_SIZE);
					}
				};

				// Load the state
				let stack_frame_ptr = task_next.stack_begin.cast::<u8>();
				stack_frame_ptr.copy_to_nonoverlapping(chunk_a, CHUNK_A_SIZE);
				stack_frame_ptr.add(CHUNK_A_SIZE).copy_to_nonoverlapping(chunk_b, CHUNK_B_SIZE);
			}
		}
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
			task.stack_frame = mem::ALLOCATOR.alloc(core::alloc::Layout::new::<StackFrame>()) as *mut StackFrame;

			if !task.is_alloc() {
				return Err(TaskError::Alloc)
			}

			registry::add(&task);

			Ok(task)
		}
	}

	/// Enqueues the task for context switching
	///
	pub fn start(&self) {
		(self.runner)();
	}
}

impl core::ops::Drop for Task {
	fn drop(&mut self) {
		let _critical = sync::Critical::new();
		unsafe {registry::remove(&self)};
	}
}
