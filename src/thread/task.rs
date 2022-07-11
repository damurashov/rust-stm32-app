use crate::{mem, thread::sync, log, log::Logger};
use core::fmt::Write;
use core::alloc::GlobalAlloc;
use core::ops::{Index, IndexMut, Drop};
use core::arch::asm;
use core::convert::{From};
use core::marker::{PhantomData, PhantomPinned};
use core::pin::Pin;

pub type Runner = fn() -> ();
type TaskId = usize;
const TASK_ID_INVALID: TaskId = 0xffffffff;

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
	Alloc(usize),  // Could not allocate the memory
	MaxNtasks(usize),  // The max. allowed number of tasks has been exceeded
	NotFound,
}

pub struct DynAlloc<'a> {
	stack: *mut u8,
	stack_size: usize,
	_a: PhantomData<&'a mut usize>,
}

impl<'a> DynAlloc<'a> {
	pub fn from_usize(mut stack_size: usize) -> Result<DynAlloc<'a>, TaskError> {
		unsafe {
			stack_size = stack_size.next_multiple_of(core::mem::size_of::<usize>());
			let allocated = mem::ALLOCATOR.alloc(core::alloc::Layout::from_size_align(stack_size, 4).unwrap());

			if allocated.is_null() {
				Err(TaskError::Alloc(stack_size))
			} else {
				Ok(DynAlloc {
					stack: allocated,
					stack_size,
					_a: PhantomData,
				})
			}
		}
	}
}

impl<'a> Drop for DynAlloc<'a> {
	fn drop(&mut self) {
		unsafe {
			mem::ALLOCATOR.dealloc(self.stack, core::alloc::Layout::new::<usize>());
		}
	}
}

pub struct StaticAlloc<'a, const N: usize>
	where [(); N / core::mem::size_of::<usize>()]: {

	stack: [usize; N / core::mem::size_of::<usize>()],
	_a: PhantomData<&'a mut u8>,
	_b: PhantomPinned,
}

impl<'a, const N: usize> StaticAlloc<'a, N>
	where [(); N / core::mem::size_of::<usize>()]: {

	const STACK_SIZE: usize = N / core::mem::size_of::<usize>();

	pub fn new() -> Self {
		Self {
			stack: [0; N / core::mem::size_of::<usize>()],
			_a: PhantomData,
			_b: PhantomPinned,
		}
	}
}

#[derive(Clone, Copy)]
enum Context {
	Initialized(StackFrame),
	Uninitialized,
}

struct ContextQueue<const N: usize> {
	context_queue: [Context; N],
	current: TaskId,
}

impl<const N: usize> ContextQueue<N> {
	const fn new() -> Self {
		Self {
			context_queue: [Context::Uninitialized; N],
			current: TASK_ID_INVALID,
		}
	}

	fn alloc(&mut self) -> Result<(TaskId, &mut StackFrame), TaskError> {
		for i in 0..N {
			if let Context::Uninitialized = self.context_queue[i] {
				self.context_queue[i] = Context::Initialized([0; StackFrameLayout::Size as usize]);

				if let Context::Initialized(ref mut stack_frame) = self.context_queue[i] {
					return Ok((i, stack_frame))
				}
			}
		}

		Err(TaskError::MaxNtasks(N))
	}

	fn dealloc(&mut self, task_id: TaskId) {
		self.context_queue[task_id] = Context::Uninitialized;

		if self.current == task_id {
			self.current = TASK_ID_INVALID;
		}
	}
}

static mut CONTEXT_QUEUE: ContextQueue<2> = ContextQueue::<2>::new();

pub struct Stack<'a>(&'a mut usize, usize);  // Begin of memory chunk, length (multiple of type)

impl Stack<'_> {
	/// Stack size in bytes
	///
	fn size(&self) -> usize {
		core::mem::size_of_val(self.0) * self.1
	}

	fn addr_start(&self) -> usize {
		(self.0 as *const usize).to_bits()
	}
}

impl<'a, const N: usize> From<&'a mut StaticAlloc<'a, N>> for Stack<'a>
	where [(); N / core::mem::size_of::<usize>()]: {

	fn from(alloc: &'a mut StaticAlloc<'a, N>) -> Self {
		log!("Converting {:?}", core::ptr::addr_of!(alloc.stack));
		log!("Stack size {}", StaticAlloc::<'a, N>::STACK_SIZE * core::mem::size_of::<usize>());
		Self (
			unsafe {alloc.stack.as_mut_slice().as_mut_ptr().as_mut().unwrap()},
			StaticAlloc::<'a, N>::STACK_SIZE,
		)
	}
}

impl <'a> From<DynAlloc<'a>> for Stack<'a> {
	fn from(alloc: DynAlloc<'a>) -> Self {
		unsafe {
			let begin = <*mut usize>::from_bits(alloc.stack.to_bits().next_multiple_of(core::mem::size_of::<usize>()));
			let size = alloc.stack.offset(alloc.stack_size as isize).offset_from(begin.cast()) as usize
				/ core::mem::size_of::<usize>();
			Self(&mut *begin, size)
		}
	}
}

pub struct Task<'a> {
	runner: Runner,
	stack: Stack<'a>,
	id: TaskId,
}

/// Stores a pointer to an allocated stack and values of registers.
///
impl<'a> Task<'a> {
	/// New instance from runner and stack
	pub fn from_rs(runner: Runner, stack: Stack<'a>) -> Self {
		log!("Creating new task w/ stack at {:#x?} stack size {}", stack.addr_start(), stack.size());
		Self {
			runner,
			stack,
			id: TASK_ID_INVALID,
		}
	}

	pub fn start(&mut self) -> Result<(), TaskError> {
		let _critical = sync::Critical::new();
		let (_, stack_frame) = unsafe {CONTEXT_QUEUE.alloc()}?;

		stack_frame[StackFrameLayout::Pc] = runner_wrap as usize;
		stack_frame[StackFrameLayout::Sp] = self.stack.addr_start() + self.stack.size();
		stack_frame[StackFrameLayout::R0] = (self as *mut Self).to_bits();

		Ok(())
	}

	pub fn stop(&self) {
		let _critical = sync::Critical::new();

		if self.id != TASK_ID_INVALID {
			unsafe {CONTEXT_QUEUE.dealloc(self.id)};
		}
	}
}

impl<'a> Drop for Task<'a> {
	fn drop(&mut self) {
		self.stop();
	}
}


#[no_mangle]
unsafe extern "C" fn runner_wrap(task_addr: usize) {
	let task = (task_addr as *mut Task).as_ref().unwrap();
	log!("Starting task id={:#x}", task.id);
	(task.runner)();

	{
		let _critical = sync::Critical::new();
		task.stop();
	}

	loop {}  // Trap until the task gets dequeued by the scheduler
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
			if let Context::Initialized(_) = context_queue.context_queue[i % N] {
				return i % N as TaskId;
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
		} else if let Context::Initialized(stack_frame) = &CONTEXT_QUEUE.context_queue[CONTEXT_QUEUE.current as usize] {
			(stack_frame as *const StackFrame).to_bits()
		} else {
			0
		}
	};

	let next = {
		let id = RoundRobin::select_next(&CONTEXT_QUEUE);

		if TASK_ID_INVALID == id {
			0
		} else if let Context::Initialized(stack_frame) = &CONTEXT_QUEUE.context_queue[id as usize] {
			(stack_frame as *const StackFrame).to_bits()
		} else {
			0
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
