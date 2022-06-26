use crate::{mem, thread::sync};
use core::alloc::GlobalAlloc;
use core::ops::{Index, IndexMut};

const STACK_FRAME_SIZE: usize = 17;
pub type Runner = &'static dyn Fn() -> ();
type StackFrame = [usize; STACK_FRAME_SIZE];

/// Stores offsets of certains registers in `StackFrame`
///
enum StackFrameLayout {  // Warning: must be synchronized with `sync.s`. Note that the currently used layout must be in sync w/ task.s
	R0 = 9,
	Sp = 0,
	Pc = 15,
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

/// Stores context of a task
///
#[derive(Clone)]
pub struct Task {
	runner: Runner,
	stack_begin: *mut u8,
	stack_frame: *mut StackFrame,
	id: usize,
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

			(&mut *task.stack_frame)[StackFrameLayout::Pc] = Task::runner_wrap as usize;
			(&mut *task.stack_frame)[StackFrameLayout::Sp] = task.stack_begin as usize + stack_size - 1;

			Ok(task)
		}
	}

	/// Enqueues the task for context switching
	///
	pub fn start(&mut self) -> Result<(), TaskError> {
		unsafe {
			let _critical = sync::Critical::new();
			queue::add(self)?;
			(*self.stack_frame)[StackFrameLayout::R0] = self as *mut Task as usize;

			Ok(())
		}
	}
}

const TASKS_MAX: usize = 2; // TODO: obsolete.

/// A pair of references to tasks.
///
type ContextSwap<'a> = (&'a Task, &'a Task);  // (previous, next)

type TaskId = usize;
const TASK_ID_INVALID: TaskId = 0xffffffff;

struct ContextQueue<const N: usize> {
	queue: [*const Task; N],
	current: TaskId,
}

/// Fixed-size registry of tasks.
///
impl<const N: usize> ContextQueue<N> {
	pub const fn new() -> ContextQueue<N> {
		ContextQueue::<N> {
			queue: [core::ptr::null(); N],
			current: TASK_ID_INVALID,
		}
	}

	/// Makes an attempt to register the task in the queue.
	///
	pub fn register_task(&mut self, task: &mut Task) -> Result<(), TaskError> {
		match self.find(core::ptr::null()) {
			Ok(id) => {
				task.id = id;
				self.queue[task.id as usize] = task;

				Ok(())
			},
			Err(_) => {
				Err(TaskError::MaxNtasks(N))
			}
		}
	}

	/// Searches for the task and removes it from the queue
	///
	pub fn unregister_task(&mut self, task: &mut Task) -> Result<(), TaskError> {
		if N > task.id && task.id >= 0 {
			if self.queue[task.id as usize] == task {
				task.id = TASK_ID_INVALID;
				self.queue[task.id as usize] = core::ptr::null();

				return Ok(())
			}
		}

		Err(TaskError::NotFound)
	}

	fn find(&self, task: *const Task) -> Result<TaskId, TaskError> {
		for i in 0 .. N {
			if self.queue[i as usize] == task {
				return Ok(i)
			}
		}

		Err(TaskError::NotFound)
	}

	/// Checks whether there is a currently running task
	///
	pub fn check_has_running(&self) -> bool {
		self.current >= 0
	}
}

/// Encapsulated sheduling algorithm selecting a next task from the queue of pending ones.
///
trait Scheduler {
	/// Runs over a queue and selects which task to run next.
	///
	/// In the case when there are no running (pending) tasks, the sheduler is expected to return `TaskError::NotFound`.
	/// For the case of only one task being active at a moment, the sheduler should return this very task as both
	/// "previous" and the "next" one.
	///
	/// As an effect, the `ContextQueue<N>` object's "current" field will be modified (set to the index of a next
	/// selected task).
	///
	fn switch_next<const N: usize>(context_queue: &mut ContextQueue<N>) -> Result<ContextSwap, TaskError>;
}

struct RoundRobin();

/// Implements "Round Robin" scheduling algorithm
///
impl Scheduler for RoundRobin {
	fn switch_next<const N: usize>(context_queue: &mut ContextQueue<N>) -> Result<ContextSwap, TaskError> {
		if context_queue.check_has_running() {
			let current: &Task = unsafe{&*(context_queue.queue[context_queue.current as usize])};
			let mut next = current;

			for i in context_queue.current as usize + 1 .. context_queue.current as usize + N + 1 {
				if !context_queue.queue[i % N].is_null() {
					next = unsafe {&*context_queue.queue[i % N]};
				}
			}

			return Ok((current, next))
		}

		Err(TaskError::NotFound)
	}
}

mod queue {
	use super::{Task, StackFrame, TaskError, TASKS_MAX};


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

		for t in &mut QUEUE {
			if t.is_null() {
				*t = task as *const Task;

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
		for id in (STATE.current_id + 1)..(STATE.current_id + TASKS_MAX + 1) {
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
	#[no_mangle]
	unsafe extern "C" fn stack_frame_swap_next(chunk_a: *mut u8, chunk_b: *mut u8) {
		// A part of the context is stored automatically in a current SP (either MSP or PSP) before an interrupt, while
		// the other one - in MSP.
		const CHUNK_A_SIZE: usize = 9;
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
				let stack_frame_ptr = task_next.stack_frame.cast::<u8>();
				stack_frame_ptr.copy_to_nonoverlapping(chunk_a, CHUNK_A_SIZE);
				stack_frame_ptr.add(CHUNK_A_SIZE).copy_to_nonoverlapping(chunk_b, CHUNK_B_SIZE);
			}
		}
	}
}

impl core::ops::Drop for Task {
	fn drop(&mut self) {
		let _critical = sync::Critical::new();
		unsafe {queue::remove(&self)};
	}
}
