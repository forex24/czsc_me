use std::any::Any;
use std::collections::HashMap;
use std::sync::RwLock;

/// Trait that must be implemented by types that want to use caching
pub trait Cacheable {
    /// Get the cache storage
    fn get_cache(&self) -> &RwLock<HashMap<String, Box<dyn Any + Send + Sync>>>;
}

/// Macro to implement the Cacheable trait
#[macro_export]
macro_rules! impl_cacheable {
    ($type:ty) => {
        impl Cacheable for $type {
            fn get_cache(&self) -> &RwLock<HashMap<String, Box<dyn Any + Send + Sync>>> {
                &self._memoize_cache
            }
        }
    };
}

/// Macro to create a cached method
#[macro_export]
macro_rules! make_cache {
    ($func:ident, $ret_type:ty) => {
        fn $func(&self) -> $ret_type {
            let cache = self.get_cache();
            let func_key = stringify!($func);
            
            // Try to get from cache first
            if let Ok(cache_read) = cache.read() {
                if let Some(cached) = cache_read.get(func_key) {
                    if let Some(result) = cached.downcast_ref::<$ret_type>() {
                        return result.clone();
                    }
                }
            }
            
            // If not in cache, compute and store
            let result = self.$func##_impl();
            if let Ok(mut cache_write) = cache.write() {
                cache_write.insert(func_key.to_string(), Box::new(result.clone()));
            }
            
            result
        }
    };
}

/// Helper macro to define the cache field in a struct
#[macro_export]
macro_rules! define_cache_field {
    () => {
        _memoize_cache: RwLock<HashMap<String, Box<dyn Any + Send + Sync>>> = RwLock::new(HashMap::new())
    };
} 