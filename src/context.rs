use std::any::Any;
use std::collections::HashMap;
use std::fmt::Debug;
use std::sync::Mutex as StdMutex;
use std::sync::Arc;

// Define a clonable wrapper for Box<dyn Any + Send + Sync>
#[derive(Debug)]
pub struct CloneableAny(Box<dyn Any + Send + Sync>);

impl Clone for CloneableAny {
    fn clone(&self) -> Self {
        // Attempt to clone the underlying value if it supports the Clone trait.
        if let Some(value) = self.0.downcast_ref::<i32>() {
            CloneableAny(Box::new(value.clone()))
        } else if let Some(value) = self.0.downcast_ref::<String>() {
            CloneableAny(Box::new(value.clone()))
        } else {
            panic!("Unsupported type in CloneableAny");
        }
    }
}

impl CloneableAny {
    pub fn new<T: Any + Clone + Send + Sync>(value: T) -> Self {
        CloneableAny(Box::new(value))
    }
}

// The context is now using CloneableAny instead of Box<dyn Any + Send + Sync>
pub type Context = HashMap<String, CloneableAny>;

// Define the function type for context marshaling and unmarshaling.
type ContextMarshalFunc = Box<dyn Fn(&Context) -> Result<HashMap<String, CloneableAny>, String> + Send + Sync>;
type ContextUnmarshalFunc = Box<dyn Fn(&mut Context, HashMap<String, CloneableAny>) -> Result<(), String> + Send + Sync>;

// Global lists of marshaling and unmarshaling functions, protected by mutex for thread safety.
lazy_static::lazy_static! {
    static ref CONTEXT_MARSHAL_FUNCS: Arc<StdMutex<Vec<ContextMarshalFunc>>> = Arc::new(StdMutex::new(Vec::new()));
    static ref CONTEXT_UNMARSHAL_FUNCS: Arc<StdMutex<Vec<ContextUnmarshalFunc>>> = Arc::new(StdMutex::new(Vec::new()));
}

// Register a context marshaling function.
pub fn register_context_marshaler(f: ContextMarshalFunc) {
    let mut funcs = CONTEXT_MARSHAL_FUNCS.lock().unwrap();
    funcs.push(f);
}

// Register a context unmarshaling function.
pub fn register_context_unmarshaler(f: ContextUnmarshalFunc) {
    let mut funcs = CONTEXT_UNMARSHAL_FUNCS.lock().unwrap();
    funcs.push(f);
}

// Marshal a context into a map.
pub fn marshal_context(ctx: &Context) -> Result<HashMap<String, CloneableAny>, String> {
    let mut result = HashMap::new();
    let funcs = CONTEXT_MARSHAL_FUNCS.lock().unwrap();
    for f in funcs.iter() {
        match f(ctx) {
            Ok(m) => {
                for (k, v) in m {
                    result.insert(k, v);
                }
            }
            Err(e) => return Err(e),
        }
    }
    Ok(result)
}

// Unmarshal a context from a map.
pub fn unmarshal_context(ctx: &mut Context, vals: HashMap<String, CloneableAny>) -> Result<(), String> {
    let funcs = CONTEXT_UNMARSHAL_FUNCS.lock().unwrap();
    for f in funcs.iter() {
        f(ctx, vals.clone())?;
    }
    Ok(())
}

// Copy context from one to another by marshaling and unmarshaling.
pub fn copy_context(from: &Context, to: &mut Context) -> Result<(), String> {
    let marshaled = marshal_context(from)?;
    unmarshal_context(to, marshaled)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    // Test marshaling and unmarshaling a context.
    #[test]
    fn test_marshal_unmarshal_context() {
        let mut ctx: Context = HashMap::new();
        ctx.insert("aggregate_id".to_string(), CloneableAny::new(42));

        let mut new_ctx: Context = HashMap::new();
        copy_context(&ctx, &mut new_ctx).unwrap();

        assert_eq!(
            *new_ctx.get("aggregate_id").unwrap().0.downcast_ref::<i32>().unwrap(),
            42
        );
    }

    // Test that register_context_marshaler works.
    #[test]
    fn test_register_context_marshaler() {
        register_context_marshaler(Box::new(|ctx: &Context| {
            let mut result = HashMap::new();
            if let Some(val) = ctx.get("aggregate_id") {
                result.insert("aggregate_id".to_string(), val.clone());
            }
            Ok(result)
        }));

        let ctx: Context = HashMap::new();
        assert!(marshal_context(&ctx).is_ok());
    }

    // Test that register_context_unmarshaler works.
    #[test]
    fn test_register_context_unmarshaler() {
        let mut ctx: Context = HashMap::new();
        let mut new_ctx: Context = HashMap::new();

        register_context_unmarshaler(Box::new(|ctx: &mut Context, vals: HashMap<String, CloneableAny>| {
            for (k, v) in vals {
                ctx.insert(k, v);
            }
            Ok(())
        }));

        copy_context(&ctx, &mut new_ctx).unwrap();
    }
}