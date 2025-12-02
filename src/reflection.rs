use std::any::{Any, TypeId};
use std::collections::HashMap;

/// Trait for runtime type reflection
pub trait Reflect: Any + Send + Sync {
    /// Get TypeId of the concrete type
    fn type_id(&self) -> TypeId {
        TypeId::of::<Self>()
    }

    /// Get type name
    fn type_name(&self) -> &'static str {
        std::any::type_name::<Self>()
    }

    /// Downcast to Any
    fn as_any(&self) -> &dyn Any;

    /// Downcast to mutable Any
    fn as_any_mut(&mut self) -> &mut dyn Any;

    /// Apply value from another reflected object
    fn apply(&mut self, value: &dyn Reflect);

    /// Clone into a boxed Reflect
    fn reflect_clone(&self) -> Box<dyn Reflect>;

    // Field access for structs
    fn field_count(&self) -> usize {
        0
    }
    fn field_at(&self, _index: usize) -> Option<&dyn Reflect> {
        None
    }
    fn field_at_mut(&mut self, _index: usize) -> Option<&mut dyn Reflect> {
        None
    }
    fn field_name(&self, _index: usize) -> Option<&str> {
        None
    }
    fn field_by_name(&self, _name: &str) -> Option<&dyn Reflect> {
        None
    }
    fn field_by_name_mut(&mut self, _name: &str) -> Option<&mut dyn Reflect> {
        None
    }
}

/// Dynamic value storage for reflection
#[derive(Clone, Debug)]
pub enum ReflectValue {
    Bool(bool),
    I32(i32),
    U32(u32),
    F32(f32),
    F64(f64),
    String(String),
    Usize(usize),
}

/// Registry for reflected types
#[derive(Default)]
pub struct TypeRegistry {
    registrations: HashMap<TypeId, TypeRegistration>,
}

impl TypeRegistry {
    /// Create new registry
    pub fn new() -> Self {
        Self::default()
    }

    /// Register a type
    pub fn register<T: Reflect + Default + Clone>(&mut self) {
        self.register_with_fields::<T>(vec![]);
    }

    /// Register a type with field names
    pub fn register_with_fields<T: Reflect + Default + Clone>(
        &mut self,
        field_names: Vec<&'static str>,
    ) {
        let registration = TypeRegistration::new::<T>(field_names);
        self.registrations.insert(TypeId::of::<T>(), registration);
    }

    /// Get registration by TypeId
    pub fn get(&self, type_id: TypeId) -> Option<&TypeRegistration> {
        self.registrations.get(&type_id)
    }
}

/// Type registration data
pub struct TypeRegistration {
    pub type_name: &'static str,
    pub type_id: TypeId,
    pub default_fn: fn() -> Box<dyn Reflect>,
    pub field_names: Vec<&'static str>,
}

impl TypeRegistration {
    pub fn new<T: Reflect + Default + Clone>(field_names: Vec<&'static str>) -> Self {
        Self {
            type_name: std::any::type_name::<T>(),
            type_id: TypeId::of::<T>(),
            default_fn: || Box::new(T::default()),
            field_names,
        }
    }
}

// Implement Reflect for common primitives
macro_rules! impl_reflect_primitive {
    ($($t:ty),*) => {
        $(
            impl Reflect for $t {
                fn as_any(&self) -> &dyn Any {
                    self
                }

                fn as_any_mut(&mut self) -> &mut dyn Any {
                    self
                }

                fn apply(&mut self, value: &dyn Reflect) {
                    if let Some(v) = value.as_any().downcast_ref::<$t>() {
                        *self = v.clone();
                    }
                }

                fn reflect_clone(&self) -> Box<dyn Reflect> {
                    Box::new(self.clone())
                }
            }
        )*
    };
}

impl_reflect_primitive!(i32, u32, f32, f64, bool, String, usize);

/// Macro to implement Reflect for structs
#[macro_export]
macro_rules! impl_reflect {
    ($t:ty) => {
        impl $crate::reflection::Reflect for $t {
            fn as_any(&self) -> &dyn std::any::Any {
                self
            }

            fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
                self
            }

            fn apply(&mut self, value: &dyn $crate::reflection::Reflect) {
                if let Some(v) = value.as_any().downcast_ref::<$t>() {
                    *self = v.clone();
                }
            }

            fn reflect_clone(&self) -> Box<dyn $crate::reflection::Reflect> {
                Box::new(self.clone())
            }
        }
    };

    // Variant with field names for struct reflection
    ($t:ty, fields: [$($field:ident),* $(,)?]) => {
        impl $crate::reflection::Reflect for $t {
            fn as_any(&self) -> &dyn std::any::Any {
                self
            }

            fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
                self
            }

            fn apply(&mut self, value: &dyn $crate::reflection::Reflect) {
                if let Some(v) = value.as_any().downcast_ref::<$t>() {
                    *self = v.clone();
                }
            }

            fn reflect_clone(&self) -> Box<dyn $crate::reflection::Reflect> {
                Box::new(self.clone())
            }

            fn field_count(&self) -> usize {
                let mut count = 0;
                $(
                    let _ = stringify!($field);
                    count += 1;
                )*
                count
            }

            fn field_name(&self, index: usize) -> Option<&str> {
                let names = &[$(stringify!($field)),*];
                names.get(index).copied()
            }
        }
    };
}
