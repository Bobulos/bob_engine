#[macro_export]
macro_rules! register_system {
    // register_system!(engine, MySystem::new())
    ($engine:expr, $system:expr) => {
        $engine.register(Box::new($system))
    };
}