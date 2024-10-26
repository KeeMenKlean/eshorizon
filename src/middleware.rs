// Trait for CommandHandler.
pub trait CommandHandler {
    fn handle_command(&self, command: &str);
}

// Wrapper struct for CommandHandler middleware closures.
pub struct CommandHandlerMiddlewareStruct<F>
where
    F: Fn(&str),
{
    func: F,
}

impl<F> CommandHandlerMiddlewareStruct<F>
where
    F: Fn(&str),
{
    pub fn new(func: F) -> Self {
        CommandHandlerMiddlewareStruct { func }
    }
}

impl<F> CommandHandler for CommandHandlerMiddlewareStruct<F>
where
    F: Fn(&str),
{
    fn handle_command(&self, command: &str) {
        (self.func)(command);
    }
}

// Function to chain multiple middlewares around a CommandHandler.
pub fn use_command_handler_middleware(
    handler: Box<dyn CommandHandler>,
    middlewares: Vec<Box<dyn Fn(Box<dyn CommandHandler>) -> Box<dyn CommandHandler>>>,
) -> Box<dyn CommandHandler> {
    let mut h = handler;
    for middleware in middlewares.into_iter().rev() {
        h = middleware(h);
    }
    h
}

// Trait for EventHandler.
pub trait EventHandler {
    fn handle_event(&self, event: &str);
}

// Wrapper struct for EventHandler middleware closures.
pub struct EventHandlerMiddlewareStruct<F>
where
    F: Fn(&str),
{
    func: F,
}

impl<F> EventHandlerMiddlewareStruct<F>
where
    F: Fn(&str),
{
    pub fn new(func: F) -> Self {
        EventHandlerMiddlewareStruct { func }
    }
}

impl<F> EventHandler for EventHandlerMiddlewareStruct<F>
where
    F: Fn(&str),
{
    fn handle_event(&self, event: &str) {
        (self.func)(event);
    }
}

// Function to chain multiple middlewares around an EventHandler.
pub fn use_event_handler_middleware(
    handler: Box<dyn EventHandler>,
    middlewares: Vec<Box<dyn Fn(Box<dyn EventHandler>) -> Box<dyn EventHandler>>>,
) -> Box<dyn EventHandler> {
    let mut h = handler;
    for middleware in middlewares.into_iter().rev() {
        h = middleware(h);
    }
    h
}

// Unit tests to verify middleware logic.
#[cfg(test)]
mod tests {
    use super::*;

    struct TestCommandHandler;

    impl CommandHandler for TestCommandHandler {
        fn handle_command(&self, command: &str) {
            println!("Handling command: {}", command);
        }
    }

    struct TestEventHandler;

    impl EventHandler for TestEventHandler {
        fn handle_event(&self, event: &str) {
            println!("Handling event: {}", event);
        }
    }

    #[test]
    fn test_command_handler_middleware() {
        let handler: Box<dyn CommandHandler> = Box::new(TestCommandHandler);

        let middleware1: Box<dyn Fn(Box<dyn CommandHandler>) -> Box<dyn CommandHandler>> =
            Box::new(|next| {
                Box::new(CommandHandlerMiddlewareStruct::new(move |command: &str| {
                    println!("Middleware 1 before");
                    next.handle_command(command);
                    println!("Middleware 1 after");
                }))
            });

        let middleware2: Box<dyn Fn(Box<dyn CommandHandler>) -> Box<dyn CommandHandler>> =
            Box::new(|next| {
                Box::new(CommandHandlerMiddlewareStruct::new(move |command: &str| {
                    println!("Middleware 2 before");
                    next.handle_command(command);
                    println!("Middleware 2 after");
                }))
            });

        let wrapped_handler = use_command_handler_middleware(handler, vec![middleware1, middleware2]);

        wrapped_handler.handle_command("TestCommand");
    }

    #[test]
    fn test_event_handler_middleware() {
        let handler: Box<dyn EventHandler> = Box::new(TestEventHandler);

        let middleware1: Box<dyn Fn(Box<dyn EventHandler>) -> Box<dyn EventHandler>> =
            Box::new(|next| {
                Box::new(EventHandlerMiddlewareStruct::new(move |event: &str| {
                    println!("Middleware 1 before");
                    next.handle_event(event);
                    println!("Middleware 1 after");
                }))
            });

        let middleware2: Box<dyn Fn(Box<dyn EventHandler>) -> Box<dyn EventHandler>> =
            Box::new(|next| {
                Box::new(EventHandlerMiddlewareStruct::new(move |event: &str| {
                    println!("Middleware 2 before");
                    next.handle_event(event);
                    println!("Middleware 2 after");
                }))
            });

        let wrapped_handler = use_event_handler_middleware(handler, vec![middleware1, middleware2]);

        wrapped_handler.handle_event("TestEvent");
    }
}