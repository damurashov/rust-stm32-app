use crate::{mem, thread::sync};
use core::alloc::GlobalAlloc;
use core::ops::{Index, IndexMut};

pub type Runner = &'static dyn Fn() -> ();
type StackFrame = [usize; 16];

// Warning: must be synchronized with `sync.s`. Note that when changing layout
/// Stores offsets of certains registers in `StackFrame`
///
enum StackFrameLayout {
	R0 = 8,
	Sp = 12,
	Pc = 14,
}

impl Index<StackFrameLayout> for StackFrame {
	type Output = usize;

	fn index(&self, sfl: StackFrameLayout) -> &Self::Output {
		&self[sfl as usize]
	}
}

impl IndexMut<StackFrameLayout> for StackFrame {
	fn index_mut(&mut self, sfl: StackFrameLayout) -> &mut Self::Output {
		&mut self[sfl as usize]
	}
}

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
	id: usize,
}

mod queue {
	use super::{Task, StackFrame, TaskError};

	const TASKS_MAX: usize = 2;
	static mut QUEUE: [*const Task; TASKS_MAX] = [
		0 as *const Task,
		0 as *const Task,
	];

	struct State {
		current_id: usize,
		free: usize,
	}

	static mut STATE: State = State {current_id: 0, free: TASKS_MAX};

	pub unsafe fn add(task: &mut Task) -> Result<(), TaskError> {
		task.id = 0;

		for mut t in QUEUE {
			if t.is_null() {
				t = task;

				return Ok(())
			}

			task.id += 1;
		}

		Err(TaskError::MaxNtasks(TASKS_MAX))
	}

	unsafe fn find(task: *const Task) -> Result<*mut *const Task, TaskError> {
		for mut t in QUEUE {
			if task == t {
				return Ok(&mut t)
			}
		}

		Err(TaskError::NotFound)
	}

	pub unsafe fn remove(task: &Task) -> Result<(), TaskError> {
		let mut queue_entry = find(task)?;

		*queue_entry = 0 as *const Task;

		Ok(())
	}

	unsafe fn get_next_round_robin<'a>() -> Result<&'a Task, TaskError> {
		for id in (STATE.current_id + 1)..(STATE.current_id + TASKS_MAX) {
			let task = QUEUE[id % TASKS_MAX];

			if !task.is_null() {
				STATE.current_id = id % TASKS_MAX;

				return Ok(&*task);
			}
		}

		Err(TaskError::NotFound)
	}

	unsafe fn get_current<'a>() -> Result<&'a Task, TaskError> {
		let task = QUEUE[STATE.current_id % TASKS_MAX];

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
	#[no_mangle]
	extern "C" fn runner_wrap(task: &mut Task) {
		(task.runner)();

		unsafe {
			let _critical = sync::Critical::new();
			queue::remove(task);
		}

		loop {}  // Trap until the task gets dequeued by the scheduler
	}

	fn new() -> Task {
		let mut task = Task {runner: &|| (), stack_begin: 0 as *mut u8, stack_frame: 0 as *mut StackFrame, id: 0};

		for mut t in unsafe{*task.stack_frame} {
			t = 0;
		}

		task
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

			(*task.stack_frame)[StackFrameLayout::Pc] = Task::runner_wrap as usize;
			(*task.stack_frame)[StackFrameLayout::Sp] = task.stack_begin as usize;

			Ok(task)
		}
	}

	/// Enqueues the task for context switching
	///
	pub fn start(&mut self) {
		unsafe {
			let _critical = sync::Critical::new();
			queue::add(self);
			(*self.stack_frame)[StackFrameLayout::R0] = self as *mut Task as usize;
		}
	}
}

impl core::ops::Drop for Task {
	fn drop(&mut self) {
		let _critical = sync::Critical::new();
		unsafe {queue::remove(&self)};
	}
}
