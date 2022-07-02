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

/// Implements index-based access to stack frame using `StackFrameLayout` enum
///
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

/// Enumeration for error codes
///
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
	stack_frame: StackFrame,  // Saved registers
	id: usize,
}

/// Stores a pointer to an allocated stack and values of registers.
///
impl Task {
	#[no_mangle]
	extern "C" fn runner_wrap(task: &mut Task) {
		(task.runner)();

		unsafe {
			let _critical = sync::Critical::new();
			CONTEXT_QUEUE.unregister_task(task);
		}

		loop {}  // Trap until the task gets dequeued by the scheduler
	}

	fn new() -> Task {
		let task = Task {
			runner: &|| (),
			stack_begin: core::ptr::null_mut(),
			stack_frame: [0; STACK_FRAME_SIZE],
			id: 0
		};

		task
	}

	/// Checks whether memory for the task has been allocated successfully
	///
	pub fn is_alloc(&self) -> bool {
		!self.stack_begin.is_null()
	}

	/// Tries to allocate memory required for the task
	///
	pub fn from_stack_size(runner: Runner, stack_size: usize) -> Result<Task, TaskError> {
		let _critical = sync::Critical::new();

		unsafe {
			let mut task = Task::new();
			task.stack_begin = mem::ALLOCATOR.alloc(core::alloc::Layout::from_size_align(stack_size, 4).unwrap());

			if !task.is_alloc() {
				return Err(TaskError::Alloc)
			}

			task.stack_frame[StackFrameLayout::Pc] = Task::runner_wrap as usize;
			task.stack_frame[StackFrameLayout::Sp] = task.stack_begin as usize + stack_size - 1;

			Ok(task)
		}
	}

	/// Enqueues the task for context switching
	///
	pub fn start(&mut self) -> Result<(), TaskError> {
		unsafe {
			let _critical = sync::Critical::new();
			CONTEXT_QUEUE.register_task(self);
			self.stack_frame[StackFrameLayout::R0] = self as *mut Task as usize;

			Ok(())
		}
	}
}

impl core::ops::Drop for Task {
	fn drop(&mut self) {
		let _critical = sync::Critical::new();
		unsafe {
			mem::ALLOCATOR.dealloc(self.stack_begin.cast::<u8>(), core::alloc::Layout::new::<usize>());
			CONTEXT_QUEUE.unregister_task(self)
		};
	}
}

const TASKS_MAX: usize = 2; // TODO: obsolete.

/// A pair of references to tasks.
///
type ContextSwap<'a> = (TaskId, TaskId);  // (previous, next)

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
				self.queue[task.id as usize] = core::ptr::null();

				if self.current == task.id {
					self.current = TASK_ID_INVALID;
				}

				task.id = TASK_ID_INVALID;

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
		self.current != TASK_ID_INVALID
	}
}

static mut CONTEXT_QUEUE: ContextQueue::<2> = ContextQueue::<2>::new();

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
	fn select_next<const N: usize>(context_queue: &ContextQueue<N>) -> TaskId;
}

struct RoundRobin();

/// Implements "Round Robin" scheduling algorithm
///
impl Scheduler for RoundRobin {
	fn select_next<const N: usize>(context_queue: &ContextQueue<N>) -> TaskId {
		let base = match context_queue.current {
			TASK_ID_INVALID => 0 as usize,
			task_id_current => task_id_current as usize,
		};

		// Search for id. of a next pending task starting from the base (or from the beginning, if there were no
		// currently pending task)
		for i in base + 1 .. base + N + 1 {
			if !context_queue.queue[i % N].is_null() {
				return i as TaskId
			}
		}

		TASK_ID_INVALID
	}
}

/// Part of the task-switching ISR.
///
#[no_mangle]
unsafe extern "C" fn stack_frame_swap_next(chunk_a: *mut u8, chunk_b: *mut u8) {
}
