extern "C" {
    fn critical_enter();
    fn critical_exit();
}

#[macro_export]
macro_rules! critical {
    ($code:block) => {
        unsafe {
            critical_enter();
        }

        $code

        unsafe {
            critical_exit();
        }
    }
}
