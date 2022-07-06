use crate::{mem, thread::sync};
use core::alloc::GlobalAlloc;
use core::ops::{Index, IndexMut, Drop};
use core::arch::asm;
use core::convert::{From, Into};
use core::default::Default;

pub type Runner = fn() -> ();
type ContextPointer = *const Task;
type ContextRef<'a> = &'a Task;
type TaskId = usize;
const TASK_ID_INVALID: TaskId = 0xffffffff;
static mut CONTEXT_QUEUE: ContextQueue::<2> = ContextQueue::<2>::new();

/// Stores offsets of certains registers in `StackFrame`
///
enum StackFrameLayout {  // Warning: must be synchronized with `sync.s`. Note that the currently used layout must be in accordance w/ the layout expected by task.s

	// Those are automatically pushed into the stack before invoking ISR. By the moment of context switching, the values
	// will have been stored in a mem. pointed by a currently used stack (PSP in our case). Refer to p.26 of
	// stm32f030f4's "Programming manual"
	R0 = 0,
	R1,
	R2,
	R3,
	R12,
	Lr,  // R14
	Pc,  // R15
	Xpsr,

	// Those are pushed into the stack by the context-switching code. By the moment of context switching, the values
	// will have been stored in mem. pointed by MSP (MSP is the one always used by ISRs).
	R4,
	R5,
	R6,
	R7,
	R8,
	R9,
	R10,
	R11,
	Sp,  // R13

	Size,
}

type StackFrame = [usize; StackFrameLayout::Size as usize];

impl Default for StackFrame {
	fn default() -> Self {
		[0; StackFrameLayout::Size as usize]
	}
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

pub struct DynAlloc(*const u8, usize);

/// Stores memory allocation info
///
pub enum StackMemory<'a> {
	Stack(&'a u8),  // Means that memory has been allocated in a parent task's stack
	Heap(DynAlloc),  // Memory has been allocated in heap
}

impl Default for StackMemory<'_> {
	fn default() -> StackMemory<'static> {
		static STUB: [u8; 0] = [];
		(&STUB).into()
	}
}

/// Convert memory pointer into address
impl From<& StackMemory<'_>> for usize {
	fn from(src: &StackMemory<'_>) -> usize {
		match src {
			StackMemory::Stack(r) => (*r as *const u8).to_bits(),
			StackMemory::Heap(r) => unsafe{(r.0 as *const u8).to_bits()},
		}
	}
}

impl<'a, const N: usize> From<&'a[u8;N]> for StackMemory<'a> {
	fn from (src: &'a [u8;N]) -> StackMemory {
		unsafe {StackMemory::Stack(&*(src as *const u8))}
	}
}

impl From<usize> for StackMemory<'_> {
	fn from(stack_size: usize) -> StackMemory<'static> {
		unsafe {
			StackMemory::Heap(DynAlloc(
				mem::ALLOCATOR.alloc(core::alloc::Layout::from_size_align(stack_size, 4).unwrap()),
				stack_size,
			))
		}
	}
}

impl Drop for DynAlloc {
	fn drop(&mut self) {
		if !self.0.is_null() {
			unsafe {
				mem::ALLOCATOR.dealloc(self.0 as *mut u8, core::alloc::Layout::new::<usize>());
			}
		}
	}
}

#[no_mangle]
unsafe extern "C" fn runner_wrap(task_id: TaskId) {
	let task = CONTEXT_QUEUE.queue[task_id as usize];
	((*task).runner)();

	let _critical = sync::Critical::new();
	CONTEXT_QUEUE.unregister_task(task.as_ref().unwrap());

	loop {}  // Trap until the task gets dequeued by the scheduler
}

/// Stores context of a task
///
pub struct Context<'a> {
	runner: Runner,
	stack_memory: StackMemory<'a>,
	stack_frame: StackFrame,  // Saved registers
}

/// Stores context of a task
///
pub enum Task<'a> {
	Unqueued(Context<'a>),
	Queued(TaskId),
}

/// Stores a pointer to an allocated stack and values of registers.
///
impl<'a> Task<'a> {

	fn new() -> Self {
		Self::Unqueued(Context::<'a>{
			runner: core::ptr::null(),
			stack_memory: StackMemory::default(),
			stack_frame: StackFrame::default(),
		})
	}

	/// Constructs new task from `runner` and `stack_memory`
	pub fn from_rs(runner: Runner, stack_memory: StackMemory<'a>) -> Self {
		Self::Unqueued(Context::<'a>{
			runner,
			stack_memory,
			stack_frame: StackFrame::default(),
		})
	}

	pub fn start(mut self) -> Result<Self, TaskError> {
		if let Self::Unqueued(context) = self {
			let _critical = sync::Critical::new();
			context.stack_frame[StackFrameLayout::Pc] = runner_wrap as usize;
			context.stack_frame[StackFrameLayout::Sp] = context.stack_memory.into();

			unsafe {
				let queued_id = CONTEXT_QUEUE.register_task(context)?;
				CONTEXT_QUEUE.queue[queued_id as usize].stack_frame[StackFrameLayout::R0] = queued_id;
			}

			Ok(Self::Queued(queued_id))
		} else {
			Ok(self)
		}
	}
}

impl Drop for Task<'_> {
	fn drop(&mut self) {
		match self {
			Self::Queued(id) => unsafe {
				*self = Self::Unqueued(CONTEXT_QUEUE.unregister_task(id));
			},
			_ => {},
		}
	}
}

struct ContextQueue<const N: usize> {
	queue: [ContextPointer; N],
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
	pub fn register_task(&mut self, task: ContextRef) -> Result<usize, TaskError> {
		match self.find(core::ptr::null()) {
			Ok(id) => {
				self.queue[id as usize] = task;
				Ok(id)
			},
			Err(_) => {
				Err(TaskError::MaxNtasks(N))
			}
		}
	}

	/// Searches for the task and removes it from the queue
	///
	pub fn unregister_task(&mut self, task: ContextRef) -> Result<(), TaskError> {
		let id = self.find(task)?;
		self.queue[id as usize] = core::ptr::null();

		if id == self.current {
			self.current = TASK_ID_INVALID;
		}

		Ok(())
	}

	fn find(&self, task: ContextPointer) -> Result<TaskId, TaskError> {
		for i in 0 .. N {
			if self.queue[i as usize] == task {
				return Ok(i)
			}
		}

		Err(TaskError::NotFound)
	}
}

/// Encapsulated sheduling algorithm selecting a next task from the queue of pending ones.
///
trait Scheduler {
	/// Runs over a queue and selects which task to run next.
	///
	/// In the case when there are no running tasks, the scheduler should return TASK_ID_INVALID.
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

		// Search for id. of a next pending task starting from the base (from the beginning, if there were no
		// currently pending tasks)
		for i in base + 1 .. base + N + 2 {
			if !context_queue.queue[i % N].is_null() {
				return i % N as TaskId
			}
		}

		TASK_ID_INVALID
	}
}

/// Part of the task-switching ISR. Updates the currently run task's id. Returns a pair of stack frame addresses
///
/// # Return options
///
/// (A, A) - no switching is required (there are no pending tasks, or there is only one task running)
/// (<currsfa | 0>, nextsfa) - addresses of the current and the next task's stack frames
///
/// # Return registers layout
/// R0 - currsfa
/// R1 - nextsfa
///
#[no_mangle]
unsafe extern "C" fn task_frame_switch_get_swap() {

	let current = {
		if TASK_ID_INVALID == CONTEXT_QUEUE.current {
			0
		} else {
			let task = CONTEXT_QUEUE.queue[CONTEXT_QUEUE.current as usize];
			(&(*task).stack_frame as *const StackFrame).to_bits()
		}
	};

	let next = {
		let id = RoundRobin::select_next(&CONTEXT_QUEUE);

		if TASK_ID_INVALID == id {
			0
		} else {
			let task = CONTEXT_QUEUE.queue[id as usize];
			(&(*task).stack_frame as *const StackFrame).to_bits()
		}
	};

	asm!(
		"movs r0, {0}",  // Store `CONTEXT_QUEUE.current` in R0
		"movs r1, {1}",  // Store `next` in R1
		in(reg) current,
		in(reg) next,
		// Prevent clobbering of output registers
		out("r0") _,
		out("r1") _
	);
}
