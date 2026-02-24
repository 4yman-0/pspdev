use crate::kernel;

// This is a type alias for the enabled `restore-state-*` feature.
// For example, it is `bool` if you enable `restore-state-bool`.
use critical_section::RawRestoreState;

struct CriticalSection;
critical_section::set_impl!(CriticalSection);

unsafe impl critical_section::Impl for CriticalSection {
    unsafe fn acquire() -> RawRestoreState {
        let state = kernel::suspend_interrupts();
        state.0
    }

    unsafe fn release(token: RawRestoreState) {
        kernel::resume_interrupts(kernel::InterruptCtrlState(token));
    }
}
