macro_rules! send_control_errorless {
    ($signal:expr, $($tx:expr),*) => {
        $({
            let _ = $tx.send($signal);
        })*
    }
}

macro_rules! send_control {
    ($signal:expr, $($tx:expr),*) => {
        $({
            $tx.send($signal)?
        })*
    }
}

macro_rules! __exit_await_thread {
    ($($thread:expr),*) => {
        $(
            $thread.join();
        )*
    }
}

